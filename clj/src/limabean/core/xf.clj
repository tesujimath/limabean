(ns limabean.core.xf "Transducers.")

(defn all-of
  "Transducer to filter items selected by all filters"
  [filters]
  (if (seq filters) (filter (apply every-pred filters)) identity))

(defn postings
  "Transducer to extract postings from directives, with date et al from the parent transaction."
  []
  (comp (filter #(= :txn (:dct %)))
        (mapcat #(map (fn [p]
                        (merge (select-keys % [:date :payee :narration]) p))
                   (:postings %)))))
