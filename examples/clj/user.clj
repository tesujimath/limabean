(ns user
  (:require [limabean.core.filters :as f]))

(defn fy
  "Example of financial year date filter, from 1st April to 31st March.

  Example usage:
  ```
  (show (journal (fy 25)))
  ```"
  [year]
  (let [year (if (< year 100) (+ 2000 year) year)]
    (f/date>=< year 4 1 (inc year) 4 1)))

(defn magic-money-xf
  "Stateful transducer on directives to add money to each account which is opened."
  ([] (magic-money-xf {}))
  ([{:keys [acc units cur]}]
   (let [acc (or acc "Equity:Magic")
         units (or units 100.00M)
         cur (or cur "NZD")]
     (fn [rf]
       (let [state (volatile! {:acc acc})]
         (fn
           ;; init
           ([] (rf))
           ;; completion
           ([result] (rf result))
           ;; step
           ([result d]
            (if (= (:dct d) :open)
              (do (when (:acc @state)
                    ;; open magic equity account on first open
                    (rf result {:date (:date d), :dct :open, :acc acc})
                    (vreset! state (dissoc @state :acc)))
                  ;; original open
                  (rf result d)
                  ;; magic money transaction
                  (rf result
                      {:date (:date d),
                       :dct :txn,
                       :postings [{:acc acc, :units (- units), :cur cur}
                                  {:acc (:acc d),
                                   :units units,
                                   :cur cur,
                                   :payee "magical benefactor"}]}))
              (rf result d)))))))))
