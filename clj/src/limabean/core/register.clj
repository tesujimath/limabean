(ns limabean.core.register
  (:require [limabean.core.inventory :as inventory]
            [limabean.core.tabulate :refer
             [stack row align-left date->cell decimal->cell positions->cell
              SPACE-MEDIUM tabular tabulate]]))

(defn with-bal
  "Return a (stateful) transducer to add a running total of units to postings.
  Only one running total is maintained, unseparated by account."
  []
  (fn [rf]
    (let [state (volatile! (inventory/accumulator :none))]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result p]
         (let [acc (:acc p)
               p (dissoc p :cost) ;; register excludes cost
               accumulated (inventory/accumulate @state p)
               bal (inventory/balance accumulated)]
           (vreset! state accumulated)
           (rf result (assoc p :bal bal))))))))

(defn build
  [postings]
  (tabular {:postings (into [] (with-bal) postings)} ::register))

(defmethod tabulate ::register
  [reg]
  (stack (mapv (fn [p]
                 (row [(date->cell (:date p)) (align-left (:acc p))
                       (align-left (:payee p)) (align-left (:narration p))
                       (decimal->cell (:units p)) (align-left (:cur p))
                       (positions->cell (:bal p))]
                      SPACE-MEDIUM))
           (:postings reg))))
