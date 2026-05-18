(ns limabean.test.plugins.adjust-postings-after-booking
  (:require [clojure.set :as set]))

(defn booked-xf
  "Booked plugin to adjust posting amounts after booking, to test validation."
  [{:keys [config]}]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (let [dct' (if (and (= :txn (:dct dct))
                           (some (fn [pst] (contains? config (:acc pst)))
                                 (:postings dct)))
                    (update dct
                            :postings
                            (fn [psts]
                              (mapv (fn [pst]
                                      (if-let [adj (get config (:acc pst))]
                                        (update pst :units #(+ adj %))
                                        pst))
                                psts)))
                    dct)]
         (rf result dct'))))))
