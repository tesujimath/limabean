(ns limabean.plugins.bad.throw
  (:require [limabean.plugin-support :refer [plugin-failed-on-directive!]]))

(defn raw-xf
  "A bad plugin which throws an exception when it sees a transaction"
  [_]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (cond (= (:dct dct) :txn)
               (plugin-failed-on-directive! "plugin rejects transactions" dct)
             :else (rf result dct))))))
