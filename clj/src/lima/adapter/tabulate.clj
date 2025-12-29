(ns lima.adapter.tabulate
  (:require [clojure.edn :as edn]
            [clojure.java.shell :as shell]
            [java-time.api :as jt]
            [cheshire.core :as cheshire]
            [clojure.string :as str]))

(defn tabulate-cell
  "Tabulate a cell using lima-pod"
  [cell]
  (let [cell-json (cheshire/generate-string cell)
        tabulated (shell/sh "lima-pod" "tabulate" :in cell-json)]
    (if (= (tabulated :exit) 0)
      (tabulated :out)
      (do (println "lima-pod error" (tabulated :err))
          (throw (Exception. "lima-pod failed"))))))

(def EMPTY {:empty nil})
(def SPACE-MINOR " ")
(def SPACE-MEDIUM "  ")

(defn stack "A stack of cells" [cells] {:stack cells})

(defn row
  "Convert a row to cells with gutter"
  [cells gutter]
  {:row [cells gutter]})

(defn align-left "Convert string to left-aligned cell" [s] {:aligned [s :left]})

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

(defn inventory->cell
  "Format an inventory into a cell, ready for tabulation"
  [inv]
  (let [accounts (sort (keys inv))]
    (stack (mapv (fn [account]
                   (let [positions (get inv account)
                         positions-cell (case (count positions)
                                          0 EMPTY
                                          1 (position->cell (first positions))
                                          (stack (mapv position->cell
                                                   positions)))]
                     (row [(align-left account) positions-cell] SPACE-MEDIUM)))
             accounts))))

(defn inventory
  "Tabulate an inventory using lima-pod"
  [inv]
  (tabulate-cell (inventory->cell inv)))
