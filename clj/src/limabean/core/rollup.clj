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
        (filter (fn [[acc units]] (not (zero? units))))))

(defn account-ancestors
  "Return the ancestors of an account"
  [acc]
  (let [[ancestors _] (reduce (fn [[ancestors combined] acc]
                                (if (empty? ancestors)
                                  [[acc] acc]
                                  (let [combined (str combined ":" acc)]
                                    [(conj ancestors combined) combined])))
                        [[] ""]
                        (str/split acc #":"))]
    (pop ancestors)))

(defn account-depth
  "Return depth of account, being the number of colons"
  [acc]
  (count (filter #(= % \:) acc)))

(defn with-ancestors-units
  "Transducer map catting [acc units] into sequence of [acc units] for account and all ancestors"
  []
  (mapcat (fn [[acc units]] (map #(vector % units) (account-ancestors acc)))))

(defn build
  "Build a rollup in a single currency from an inventory"
  [inv cur]
  (let [acc-units (into {} (account-units cur) (cell/unmark inv))
        rollup-units (reduce (fn [r [acc units]]
                               (assoc r acc (+ units (or (get r acc) 0M))))
                       {}
                       (eduction (with-ancestors-units) acc-units))
        max-depth (let [depths (map account-depth (keys acc-units))]
                    (if (seq depths) (apply max depths) 0))
        rollup (into {}
                     (map #(vector %
                                   (if-let [u (get acc-units %)]
                                     [u max-depth]
                                     [(get rollup-units %) (account-depth %)]))
                       (concat (keys acc-units) (keys rollup-units))))]
    (cell/mark rollup :rollup)))

(defn padded-row
  [acc units col]
  (cell/row (into [(cell acc)] (concat (repeat col cell/EMPTY) [(cell units)]))
            cell/SPACE-MEDIUM))

(defmethod cell :rollup
  [rollup]
  (cell/stack (mapv (fn [acc]
                      (let [[units col] (get rollup acc)]
                        (padded-row acc units col)))
                (sort (keys (cell/unmark rollup))))))
