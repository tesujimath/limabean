(ns limabean.core.registry)

(defn build
  "Accumulate directives into registry"
  [directives options]
  (let [default-booking (or (:booking options) :strict)
        init (transient {:acc-booking (transient {}), :acc-curs (transient {})})
        result (reduce
                 (fn [result d]
                   (case (:dct d)
                     :open (let [booking (or (:booking d) default-booking)]
                             (assoc!
                               result
                               :acc-booking
                               (assoc! (:acc-booking result) (:acc d) booking))
                             result)
                     :txn
                       ;; collect currency for each posting
                       (reduce (fn [result p]
                                 (let [acc-curs (:acc-curs result)
                                       acc (:acc p)
                                       curs (if-let [curs (get acc-curs acc)]
                                              curs
                                              (transient #{}))]
                                   (assoc! result
                                           :acc-curs
                                           (assoc! acc-curs
                                                   acc
                                                   (conj! curs (:cur p))))))
                         result
                         (:postings d))
                     result))
                 init
                 directives)
        acc-booking (persistent! (:acc-booking result))
        acc-cur-sets (into {}
                           (map (fn [[k v]] [k (persistent! v)])
                             (persistent! (:acc-curs result))))
        acc-curs (into {} (map (fn [[k v]] [k (vec (sort v))]) acc-cur-sets))
        curs (vec (sort (into #{} (mapcat (fn [[_ v]] v) acc-cur-sets))))
        accs (vec (sort (keys acc-booking)))]
    {:acc-booking (fn [acc] (get acc-booking acc)),
     :acc-curs (fn [acc] (get acc-curs acc)),
     :accs (fn [] accs),
     :curs (fn [] curs)}))
