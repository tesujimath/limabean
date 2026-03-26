(ns limabean
  "Top-level limabean functions for use from the REPL."
  (:require [clojure.java.io :as io]
            [limabean.adapter.edn] ;; edn is for print-method only
            [limabean.adapter.loader :as loader]
            [limabean.adapter.logging :as logging]
            [limabean.adapter.show :as show]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.xf :as xf]
            [limabean.core.journal :as journal]
            [limabean.core.registry :as registry]
            [limabean.core.rollup :as rollup]
            [limabean.adapter.pod :as pod]))

(def ^:dynamic *beans*
  "An aggregate of the elements which were used in deriving the directives for the current beanfile.

  Useful in case of failed plugins, for inspecting partial state."
  nil)
(def ^:dynamic *directives*
  "Vector of all directives form the beanfile after running plugins."
  nil)
(def ^:dynamic *options* "Map of options from the beanfile." nil)
(def ^:dynamic *registry*
  "Map of attributes derived from directives and options, e.g. booking method for account."
  nil)

(defn- assign-limabean-globals
  [beans]
  (let [directives (get beans :directives [])
        options (get beans :options {})]
    (alter-var-root #'*beans* (constantly beans))
    (alter-var-root #'*directives* (constantly directives))
    (alter-var-root #'*options* (constantly options))
    (alter-var-root #'*registry* (constantly (:registry beans)))))

(defn- split-args-and-opts
  "Return a list of args and hashmap of opts, by splitting on the first keyword."
  [args-and-opts]
  (let [[args opts] (split-with (complement keyword?) args-and-opts)]
    (when (odd? (count opts))
      (throw (ex-info "bad usage"
                      {:user-error "odd number of keyword/options"})))
    (when-not (every? keyword? (take-nth 2 opts))
      (throw (ex-info "bad usage"
                      {:user-error "expected alternating keyword/options"})))
    [args (apply hash-map opts)]))

(defn- join-args-and-opts
  "Splice them back together again."
  [args opts]
  (concat args (mapcat identity opts)))

(defn load-beanfile
  [path]
  (when (:pod *beans*) (pod/stop (:pod *beans*)))
  (assign-limabean-globals {})
  (logging/initialize)
  (let [beans (loader/load-beanfile path)]
    (binding [*out* *err*]
      (println "[limabean]" (count (:raw-directives beans))
               "directives loaded from" path)
      (let [bad-plugins (filter :err (:plugins beans))]
        (doseq [plugin bad-plugins]
          (println "ERROR in plugin" (:name plugin) "-" (:err plugin))))
      (assign-limabean-globals beans)
      (if-let [err (:plugin-errors beans)]
        (println err)
        (println "[limabean]"
                 (count *directives*)
                 "directives resulting from running plugins")))
    :ok))

(defn- postings
  [args]
  (let [[filters opts] (split-args-and-opts args)]
    (eduction (comp (xf/postings) (xf/all-of filters))
              (get opts :directives *directives*))))

(defn inventory
  "Build inventory from `*directives*` and `*registry*` after applying filters, if any.

  Custom directives may be passed in after the filters using :directives."
  [& args]
  (inventory/build (postings args) (partial registry/acc-booking *registry*)))

(defn rollup
  "Build a rollup for the primary currency from `*directives*` and `*registry*` after applying filters, if any.

  To build for a different currency, simply filter by that currency, e.g
  ```
  (rollup (f/cur \"CHF\"))
  ```

  Custom directives may be passed in after the filters using :directives."
  [& args]
  (let [inv (apply inventory args)
        primary-cur (first (apply max-key val (inventory/cur-freq inv)))]
    (rollup/build inv primary-cur)))

(defn balances
  "Build balances from `*directives*` and `*options*`, optionally further filtered.

  Custom directives may be passed in after the filters using :directives.
  "
  [& args]
  (let [[filters opts] (split-args-and-opts args)]
    (apply inventory
      (join-args-and-opts (conj filters
                                (f/sub-acc (:name-assets *options*)
                                           (:name-liabilities *options*)))
                          opts))))

(defn journal
  "Build a journal of postings from `*directives*` with running balance.

  Custom directives may be passed in after the filters using :directives."
  [& args]
  (journal/build (postings args)))

(defn show "Convert `x` to a cell and tabulate it." [x] (show/show x))

(defn version
  "Get the library version from pom.properties, else returns \"unknown\"."
  []
  (or
    (let [props (java.util.Properties.)]
      (try
        (with-open
          [in
             (io/input-stream
               (io/resource
                 "META-INF/maven/io.github.tesujimath/limabean/pom.properties"))]
          (.load props in)
          (.getProperty props "version"))
        (catch Exception _ nil)))
    "unknown"))
