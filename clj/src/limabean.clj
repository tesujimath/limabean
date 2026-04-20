(ns limabean
  "Top-level limabean functions for use from the REPL."
  (:require [clojure.java.io :as io]
            [limabean.adapter.loader :as loader]
            [limabean.adapter.logging :as logging]
            [limabean.adapter.print]
            [limabean.adapter.show :as show]
            [limabean.adapter.pod :as pod]
            [limabean.adapter.bean-queries :as bean-queries]))

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
                 (count (:directives beans))
                 "directives resulting from booking and running plugins")))
    :ok))

(defn inventory
  "Build inventory from `*beans*` after applying filters, if any.

  Custom directives may be passed in after the filters using :directives."
  [& args]
  (bean-queries/inventory *beans* args))

(defn rollup
  "Build a rollup for the primary currency from an inventory.

  To build for a different currency, simply filter by that currency, e.g
  ```
  (rollup (inventory (f/cur \"CHF\")))
  ```
  "
  [inv]
  (bean-queries/rollup inv))

(defn balances
  "Build balances from `*beans*`, optionally further filtered.

  Custom directives may be passed in after the filters using :directives.
  "
  [& args]
  (bean-queries/balances *beans* args))

(defn income-statement
  "Build balances from `*beans*`, optionally further filtered.

  Custom directives may be passed in after the filters using :directives.
  "
  [& args]
  (bean-queries/income-statement *beans* args))

(defn journal
  "Build a journal of postings from `*beans*` with running balance.

  Custom directives may be passed in after the filters using :directives."
  [& args]
  (bean-queries/journal *beans* args))

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
