(ns limabean.app
  (:require [limabean.adapter.beanfile :as beanfile]
            [limabean.adapter.show :refer [show]]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.registry :as registry]
            [limabean.core.xf :as xf]
            [limabean.user]
            [taoensso.telemere :as tel]))

(defn balances
  "Print balances of assets and liabilities"
  [{:keys [beanfile]}]
  (let [{:keys [directives options]} (beanfile/book beanfile)
        registry (registry/build directives options)
        _ (tel/log! {:id ::registry, :data registry})
        postings (eduction (comp (xf/postings)
                                 (filter (f/some-sub-acc (:name-assets options)
                                                         (:name-liabilities
                                                           options))))
                           directives)
        inv (inventory/build postings (:acc-booking registry))
        _ (tel/log! {:id ::inventory, :data inv})]
    (show inv)))

(defn repl
  "Run the REPL"
  [{:keys [beanfile]}]
  (clojure.main/repl :init
                     (fn []
                       (require '[limabean.user])
                       (in-ns 'limabean.user)
                       (limabean.user/load-beanfile beanfile))))
