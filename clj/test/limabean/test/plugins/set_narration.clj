(ns limabean.test.plugins.set-narration)

(defn booked-xf
  "Transducer on booked directives to override the narration

  For an explanation of transducers, see https://clojure.org/reference/transducers"
  [{:keys [config options]}]
  (let [narration (or (:narration config) "unspecified in config")]
    (fn [rf]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result dct]
         (rf result
             (cond-> dct
               (= (:type dct) :limabean/txn) (assoc :narration narration))))))))
