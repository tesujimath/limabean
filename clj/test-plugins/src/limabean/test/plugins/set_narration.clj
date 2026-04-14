(ns limabean.test.plugins.set-narration)

(defn raw-xf
  "Transducer on raw directives to override the narration

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
             (cond-> dct (= (:dct dct) :txn) (assoc :narration narration))))))))
