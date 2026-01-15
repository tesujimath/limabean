(ns limabean.core.xf
  (:require [limabean.core.inventory :as inv]))

(defn all-of
  "Transducer to filter items selected by all filters"
  [filters]
  (if (seq filters) (filter (apply every-pred filters)) identity))

(defn postings
  "Transducer to extract postings from directives, with date et al from txn"
  []
  (comp (filter #(= :txn (:dct %)))
        (mapcat #(map (fn [p]
                        (merge (select-keys % [:date :payee :narration]) p))
                   (:postings %)))))

(defn register
  "Return a (stateful) transducer to accumulate postings with their total units.
  Only one running total is maintained, unseparated by account."
  []
  (fn [rf]
    (let [state (volatile! (inv/accumulator :none))]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result p]
         (let [acc (:acc p)
               p (dissoc p :cost) ;; register excludes cost
               accumulated (inv/accumulate @state p)
               bal (inv/balance accumulated)]
           (vreset! state accumulated)
           (rf result (assoc p :bal bal))))))))
