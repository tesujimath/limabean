(ns limabean.core.journal
  (:require [limabean.core.inventory :as inventory]
            [limabean.core.cell :refer [cell]]
            [limabean.core.cell :as cell]))

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
               p (dissoc p :cost) ;; journal excludes cost
               accumulated (inventory/accumulate @state p)
               bal (inventory/positions accumulated)]
           (vreset! state accumulated)
           (rf result (cell/mark (assoc p :bal bal) :journal/entry))))))))

(defn build [postings] (into [] (with-bal) postings))

(defmethod cell :journal/entry
  [p]
  (cell/row [(cell (:date p)) (cell (:acc p)) (cell (:payee p))
             (cell (:narration p)) (cell (:units p)) (cell (:cur p))
             (cell (:bal p))]
            cell/SPACE-MEDIUM))
