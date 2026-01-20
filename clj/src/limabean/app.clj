(ns limabean.app
  (:require [limabean.adapter.beanfile :as beanfile]
            [limabean.adapter.show :refer [show]]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.registry :as registry]
            [limabean.core.xf :as xf]
            [taoensso.telemere :as tel]))

(def reports #{"balances"})
(def default-report "balances")

(defn report
  "Run the named report"
  [{:keys [name beanfile]}]
  (case name
    "balances" (let [{:keys [directives options]} (beanfile/book beanfile)
                     registry (registry/build directives options)
                     _ (tel/log! {:id ::registry, :data registry})
                     postings (eduction (comp (xf/postings)
                                              (filter (f/some-sub-acc
                                                        (:name-assets options)
                                                        (:name-liabilities
                                                          options))))
                                        directives)
                     inv (inventory/build postings (:acc-booking registry))
                     _ (tel/log! {:id ::inventory, :data inv})]
                 (show inv))
    :else (throw (ex-info "unknown report" {:name name})))
  (flush))
