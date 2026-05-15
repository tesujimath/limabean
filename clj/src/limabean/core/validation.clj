(ns limabean.core.validation)

(defn- txn-balance-xf
  "Validate that transactions balance.

   Only transactions with provenance are checked, since these were created by plugins.
   Others are assumed to be OK, since they were produced from the core booking algorithm."
  [options]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (when (and (= :txn (:dct dct)) (:provenance dct))
         (println "checking balance for txn"
                  (:date dct)
                  (:payee dct)
                  (:narration dct)))
       (rf result dct)))))

(defn post-booking-xf
  "Post-booking validation transducer"
  [options]
  (comp (txn-balance-xf options)))
