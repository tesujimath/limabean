(ns limabean.test.plugins.duplicate-txns)

(defn raw-xf
  "Transducer on raw directives to synthesize duplicate transactions (without spans)."
  [_]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (let [result' (if (= :txn (:dct dct))
                       (rf result
                           (-> dct
                               (dissoc :span)
                               (update :postings
                                       #(mapv (fn [pst] (dissoc pst :span)) %))
                               (assoc-in [:metadata :dup] nil)))
                       result)]
         (rf result' dct))))))
