(ns limabean.user
  (:require [limabean.adapter.beanfile :as beanfile]
            [limabean.adapter.logging :as logging]
            [limabean.adapter.show :as show]
            [limabean.adapter.tabulate :as tabulate]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.registry :as registry]
            [limabean.core.xf :as xf]
            [limabean.core.journal :as journal]
            [limabean.core.rollup :as rollup]))

(def ^:dynamic *directives* nil)
(def ^:dynamic *options* nil)
(def ^:dynamic *registry* nil)

(defn load-beanfile
  [path]
  (logging/initialize)
  (let [beans (beanfile/book path)]
    (alter-var-root #'*directives* (constantly (:directives beans)))
    (alter-var-root #'*options* (constantly (:options beans)))
    (alter-var-root #'*registry*
                    (constantly (registry/build *directives* *options*)))
    (println "Welcome to limabean," (count *directives*)
             "directives loaded from" path))
  :ok)

(defn- postings
  [filters]
  (eduction (comp (xf/postings) (xf/all-of filters)) *directives*))

(defn inventory
  "Build inventory after applying filters, if any"
  [& filters]
  (inventory/build (postings filters) (:acc-booking *registry*)))

(defn rollup
  "Build a rollup for the primary currency"
  [& filters]
  (let [inv (apply inventory filters)
        primary-cur (first (apply max-key val (inventory/currency-freqs inv)))]
    (rollup/build inv primary-cur)))

(defn balances
  "Build balances, optionally further filtered"
  [& filters]
  (apply inventory
    (conj filters
          (f/some-sub-acc (:name-assets *options*)
                          (:name-liabilities *options*)))))

(defn journal
  "Build a journal of postings with running balance"
  [& filters]
  (journal/build (postings filters)))

(def show show/show)
