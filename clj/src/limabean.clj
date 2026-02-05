(ns limabean
  "Top-level limabean functions for use from the REPL."
  (:require [limabean.adapter.beanfile :as beanfile]
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

(defn load-beanfile
  [path]
  (assign-limabean-globals {})
  (logging/initialize)
  (assign-limabean-globals (beanfile/book path))
  (binding [*out* *err*]
    (println "[limabean]" (count *directives*) "directives loaded from" path))
  :ok)

(defn- postings
  [filters]
  (eduction (comp (xf/postings) (xf/all-of filters)) *directives*))

(defn inventory
  "Build inventory from `*directives*` and `*registry*` after applying filters, if any"
  [& filters]
  (inventory/build (postings filters)
                   (partial registry/acc-booking *registry*)))

(defn rollup
  "Build a rollup for the primary currency from `*directives*` and `*registry*` after applying filters, if any.

  To build for a different currency, simply filter by that currency, e.g
  ```
  (rollup (f/cur \"CHF\"))
  ```"
  [& filters]
  (let [inv (apply inventory filters)
        primary-cur (first (apply max-key val (inventory/cur-freq inv)))]
    (rollup/build inv primary-cur)))

(defn balances
  "Build balances from `*directives*` and `*options*`, optionally further filtered"
  [& filters]
  (apply inventory
    (conj filters
          (f/sub-acc (:name-assets *options*) (:name-liabilities *options*)))))

(defn journal
  "Build a journal of postings from `*directives*` with running balance"
  [& filters]
  (journal/build (postings filters)))

(defn show "Convert `x` to a cell and tabulate it." [x] (show/show x))
