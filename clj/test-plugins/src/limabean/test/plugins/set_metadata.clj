(ns limabean.test.plugins.set-metadata)

(defn raw-xf
  "Test plugin which sets metadata for the specified directive types"
  [{:keys [config]}]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (cond (contains? config (:dct dct))
               (rf result
                   (update dct :metadata #(merge % (get config (:dct dct)))))
             :else (rf result dct))))))
