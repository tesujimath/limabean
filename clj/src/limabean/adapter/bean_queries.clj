(ns limabean.adapter.bean-queries
  (:require [limabean.adapter.print]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.xf :as xf]
            [limabean.core.journal :as journal]
            [limabean.core.registry :as registry]
            [limabean.core.rollup :as rollup]))

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

(defn- postings
  [beans args]
  (let [[filters opts] (split-args-and-opts args)]
    (eduction (comp (xf/postings) (xf/all-of filters))
              (get opts :directives (:directives beans)))))

(defn inventory
  "Build inventory from `beans` after applying filters, if any.

  Custom directives may be passed in after the filters using :directives."
  [beans args]
  (inventory/build (postings beans args)
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
  "Build balances from `beans`, optionally further filtered.

  Custom directives may be passed in after the filters using :directives.
  "
  [beans args]
  (let [[filters opts] (split-args-and-opts args)]
    (inventory beans
               (join-args-and-opts
                 (conj filters
                       (f/sub-acc (:name-assets (:options beans))
                                  (:name-liabilities (:options beans))))
                 opts))))

(defn income-statement
  "Build balances from `beans`, optionally further filtered.

  Custom directives may be passed in after the filters using :directives.
  "
  [beans args]
  (let [[filters opts] (split-args-and-opts args)]
    (inventory beans
               (join-args-and-opts (conj filters
                                         (f/sub-acc
                                           (:name-income (:options beans))
                                           (:name-expenses (:options beans))))
                                   opts))))

(defn journal
  "Build a journal of postings from `beans` with running balance.

  Custom directives may be passed in after the filters using :directives."
  [beans args]
  (journal/build (postings beans args)))
