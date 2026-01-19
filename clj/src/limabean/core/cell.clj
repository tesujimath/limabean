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

(defn align-left "Convert string to left-aligned cell" [s] {:aligned [s :left]})

(defn anchored "Anchor a string at i" [s i] {:anchored [s i]})

(defn real-keys [m] (remove #(= :cell/type %) (keys m)))

(defn mark
  "Mark a map as a cell of specified type, for the cell multimethod."
  [m type]
  (assoc m :cell/type type))

(defn unmark "Unmark a previously marked cell" [m] (dissoc m :cell/type))

(defmulti cell
  "Prepare as a cell for tabulation, or nil if unsupported"
  (fn [x & _]
    (cond (nil? x) ::nil
          (and (map? x) (:cell/type x)) (:cell/type x)
          (vector? x) ::vector
          (string? x) ::string
          (jt/local-date? x) ::local-date
          (decimal? x) ::decimal
          :else ::default)))

(defmethod cell ::vector
  [x]
  (case (count x)
    0 EMPTY
    1 (cell (first x))
    ;;If every element may be prepared as a cell, return them in a stack,
    ;;else nil.
    (let [cells (stack (mapv cell x))] (if (every? some? cells) cells nil))))

(defmethod cell ::string [x] (align-left x))

(defmethod cell ::local-date [x] (cell (str x)))

(defmethod cell ::decimal
  [x]
  (let [s (str x)
        dp (or (str/index-of s ".") (count s))]
    (anchored s (dec dp))))

(defmethod cell ::nil [x] EMPTY)

(defmethod cell ::default [x] nil)
