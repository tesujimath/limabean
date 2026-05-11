(ns limabean.adapter.bean-queries
  (:require [limabean.adapter.print]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.xf :as xf]
            [limabean.core.journal :as journal]
            [limabean.core.registry :as registry]
            [limabean.core.rollup :as rollup]))

(defn- postings
  [beans filters]
  (eduction (comp (xf/postings) (xf/all-of filters)) (:directives beans)))

(defn inventory
  "Build inventory from `beans` after applying filters, if any."
  [beans filters]
  (inventory/build (postings beans filters)
                   (partial registry/acc-booking (:registry beans))))

(defn rollup
  "Build a rollup for the primary currency from an inventory.

  To build for a different currency, simply filter by that currency, e.g
  ```
  (rollup (inventory (f/cur \"CHF\")))
  ```
  "
  [inv]
  (let [primary-cur (first (apply max-key val (inventory/cur-freq inv)))]
    (rollup/build inv primary-cur)))

(defn balances
  "Build balances from `beans`, optionally further filtered."
  [beans filters]
  (inventory beans
             (conj filters
                   (f/sub-acc (:name-assets (:options beans))
                              (:name-liabilities (:options beans))))))

(defn income-statement
  "Build balances from `beans`, optionally further filtered.

  Custom directives may be passed in after the filters using :directives.
  "
  [beans filters]
  (inventory beans
             (conj filters
                   (f/sub-acc (:name-income (:options beans))
                              (:name-expenses (:options beans)))))
  ())

(defn journal
  "Build a journal of postings from `beans` with running balance.

  Custom directives may be passed in after the filters using :directives."
  [beans filters]
  (journal/build (postings beans filters)))
