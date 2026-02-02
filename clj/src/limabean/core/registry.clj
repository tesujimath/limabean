(ns limabean.core.registry
  (:require [clojure.set :as set]
            [taoensso.telemere :as tel]))

(defn build
  "Accumulate directives into registry"
  [directives options]
  (let [default-booking (get options :booking :strict)
        init (transient {:acc-booking (transient {}), :cur-freq (transient {})})
        result
          (reduce (fn [result d]
                    (case (:dct d)
                      :open (if-let [booking (get d :booking)]
                              (assoc!
                                result
                                :acc-booking
                                (assoc! (:acc-booking result) (:acc d) booking))
                              result)
                      :txn
                        ;; bump currency frequency for each posting
                        (reduce (fn [result p]
                                  (let [cur-freq (:cur-freq result)
                                        cur (:cur p)
                                        freq (get cur-freq cur 0)]
                                    (assoc! result
                                            :cur-freq
                                            (assoc! cur-freq cur (inc freq)))))
                          result
                          (:postings d))
                      result))
            init
            directives)
        acc-booking (persistent! (:acc-booking result))
        cur-freq (persistent! (:cur-freq result))
        curs (mapv first (sort-by (comp - second) (into [] cur-freq)))
        accs (into {}
                   (map (fn [acc] [acc
                                   (cond-> nil
                                     (contains? acc-booking acc)
                                       (assoc :booking (get acc-booking acc)))])
                     (keys acc-booking)))]
    {:default-booking default-booking, :accs accs, :curs curs}))

(defn acc-booking
  "Lookup the booking method for an account in the registry"
  [reg acc]
  (get-in reg [:accs acc :booking] (:default-booking reg)))
