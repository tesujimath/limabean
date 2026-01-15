(ns limabean.core.cell
  (:require [java-time.api :as jt]
            [clojure.string :as str]))

(def EMPTY {:empty nil})
(def SPACE-MINOR " ")
(def SPACE-MEDIUM "  ")

(defn stack "A stack of cells" [cells] {:stack cells})

(defn row
  "Convert a row to cells with gutter"
  [cells gutter]
  {:row [cells gutter]})

(defn empty-or [x c] (if (nil? x) EMPTY c))

(defn align-left
  "Convert string to left-aligned cell, or nil to empty"
  [s]
  (empty-or s {:aligned [s :left]}))

(defn date->cell "Convert a date to cell" [d] (align-left (str d)))

(defn decimal->cell
  "Convert decimal to cell anchored at the units digit, so will align with e.g. integers"
  [d]
  (let [s (str d)
        dp (or (str/index-of s ".") (count s))]
    {:anchored [s (dec dp)]}))

(defn cost->cell
  "Format a cost into a cell"
  [cost]
  (row [(date->cell (:date cost)) (align-left (:cur cost))
        (decimal->cell (:per-unit cost))
        (if-let [label (:label cost)]
          (align-left label)
          EMPTY) (if (:merge cost) (align-left "*") EMPTY)]
       SPACE-MINOR))

(defn position->cell
  "Format a single position into a cell"
  [pos]
  ;; TODO cost
  (let [units (row [(decimal->cell (:units pos)) (align-left (:cur pos))]
                   SPACE-MINOR)]
    (if-let [cost (:cost pos)]
      (row [units (cost->cell cost)] SPACE-MEDIUM)
      (row [units] SPACE-MEDIUM))))

(defn positions->cell
  "Format arbitrary number of positions into a stack"
  [positions]
  (case (count positions)
    0 EMPTY
    1 (position->cell (first positions))
    (stack (mapv position->cell positions))))

(defn inventory->cell
  "Format an inventory into a cell ready for tabulation"
  [inv]
  (let [accounts (sort (keys inv))]
    (stack (mapv (fn [account]
                   (row [(align-left account)
                         (positions->cell (get inv account))]
                        SPACE-MEDIUM))
             accounts))))

(defn register->cell
  "Format a register into a cell ready for tabulation"
  [reg]
  (stack (mapv (fn [p]
                 (row [(date->cell (:date p)) (align-left (:acc p))
                       (align-left (:payee p)) (align-left (:narration p))
                       (decimal->cell (:units p)) (align-left (:cur p))
                       (positions->cell (:bal p))]
                      SPACE-MEDIUM))
           reg)))
