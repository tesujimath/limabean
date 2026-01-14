(ns limabean.core.xf)


(defn postings
  "Transducer to extract postings from directives, with date et al from txn"
  []
  (comp (filter #(= :txn (:dct %)))
        (mapcat #(map (fn [p]
                        (merge (select-keys % [:date :payee :narration]) p))
                   (:postings %)))))
