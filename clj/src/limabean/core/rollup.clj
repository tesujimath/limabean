(ns limabean.core.rollup
  (:require [clojure.string :as str]
            [limabean.core.inventory :as inventory]
            [limabean.core.cell :as cell :refer [cell]]))

(defn account-units
  "Transducer of account names and units for `cur`"
  [cur]
  (comp (map (fn [[acc positions]] [acc
                                    (inventory/positions->units-of positions
                                                                   cur)]))
        (filter (fn [[_acc units]] (not (zero? units))))))

(defn account-and-ancestors
  "Return the ancestors of an account"
  [acc]
  (let [[ancestors _] (reduce (fn [[ancestors combined] acc]
                                (if (empty? ancestors)
                                  [[acc] acc]
                                  (let [combined (str combined ":" acc)]
                                    [(conj ancestors combined) combined])))
                        [[] ""]
                        (str/split acc #":"))]
    ancestors))

(defn account-depth
  "Return depth of account, being the number of colons"
  [acc]
  (count (filter #(= % \:) acc)))

(defn with-ancestors-units
  "Transducer map catting [acc units] into sequence of [acc units] for account and all ancestors"
  []
  (mapcat (fn [[acc units]]
            (map #(vector % units) (account-and-ancestors acc)))))

(defn build
  "Build a rollup in a single currency from an inventory"
  [inv cur]
  (let [item-units (into {} (account-units cur) inv)
        total-units (reduce (fn [r [acc units]]
                              (assoc r acc (+ units (get r acc 0M))))
                      {}
                      (eduction (with-ancestors-units) item-units))
        max-depth (let [depths (map account-depth (keys item-units))]
                    (if (seq depths) (apply max depths) 0))
        rollup (into {}
                     (map (fn [acc] [acc
                                     (cell/mark
                                       (cond-> nil
                                         (contains? item-units acc)
                                           (assoc :item
                                             [(get item-units acc) max-depth])
                                         (and (contains? total-units acc)
                                              ;; don't show the rollup if
                                              ;; it's simply the item
                                              ;; itself
                                              (not= (get item-units acc)
                                                    (get total-units acc)))
                                           (assoc :total
                                             [(get total-units acc)
                                              (account-depth acc)]))
                                       :rollup/entry)])
                       (concat (keys item-units) (keys total-units))))]
    rollup))

(defn padded-row
  [acc units col]
  (cell/row (into [(cell acc)] (concat (repeat col cell/EMPTY) [(cell units)]))
            cell/SPACE-MEDIUM))

(defmethod cell :rollup/entry
  [entry]
  (let [{:keys [item total]} entry
        [total-cells total-width] (if total
                                    [(concat (repeat (second total) cell/EMPTY)
                                             [(cell (first total))])
                                     (inc (second total))]
                                    [[] 0])
        item-cells (if item
                     (concat (repeat (- (second item) total-width) cell/EMPTY)
                             [(cell (first item))])
                     [])]
    (cell/row (vec (concat total-cells item-cells)) cell/SPACE-MEDIUM)))
