(ns beancount.plugins.auto_accounts
  (:require [clojure.set :as set]))

(defn raw-xf
  "Legacy auto_accounts plugin"
  [_]
  (fn [rf]
    (let [opened-accounts (volatile! #{})]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result dct]
         (cond (= (:dct dct) :open) (do (vreset! opened-accounts
                                                 (conj @opened-accounts
                                                       (:acc dct)))
                                        (rf result dct))
               (= (:dct dct) :txn)
                 (let [txn-accs (into #{} (map :acc) (:postings dct))
                       new-accs (sort (vec (set/difference txn-accs
                                                           @opened-accounts)))]
                   (when (seq new-accs)
                     (vreset! opened-accounts
                              (apply conj @opened-accounts new-accs))
                     (reduce (fn [result acc]
                               (let [auto-open {:dct :open,
                                                :date (:date dct),
                                                :acc acc,
                                                :metadata {:auto nil}}]
                                 (rf result auto-open)))
                       result
                       new-accs))
                   (rf result dct))
               :else (rf result dct)))))))
