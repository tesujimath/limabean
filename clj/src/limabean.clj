(ns limabean
  "Top-level limabean functions for use from the REPL."
  (:require [clojure.java.io :as io]
            [limabean.adapter.beanfile :as beanfile]
            [limabean.adapter.logging :as logging]
            [limabean.adapter.show :as show]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.registry :as registry]
            [limabean.core.xf :as xf]
            [limabean.core.journal :as journal]
            [limabean.core.rollup :as rollup]))

(def ^:dynamic *directives* "Vector of all directives form the beanfile." nil)
(def ^:dynamic *options* "Map of options from the beanfile." nil)
(def ^:dynamic *registry*
  "Map of attributes derived from directives and options, e.g. booking method for account."
  nil)

(defn- assign-limabean-globals
  [beans]
  (let [directives (get beans :directives [])
        options (get beans :options {})]
    (alter-var-root #'*directives* (constantly directives))
    (alter-var-root #'*options* (constantly options))
    (alter-var-root #'*registry*
                    (constantly (registry/build directives options)))))

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
  (assign-limabean-globals {})
  (logging/initialize)
  (assign-limabean-globals (beanfile/book path))
  (binding [*out* *err*]
    (println "[limabean]" (count *directives*) "directives loaded from" path))
  :ok)

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
