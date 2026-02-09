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
  "Prepare as a cell for tabulation, or ? if unsupported"
  (fn [x & _]
    (cond (nil? x) ::nil
          (and (map? x) (:cell/type x)) (:cell/type x)
          (map? x) ::map
          (vector? x) ::vector
          (set? x) ::set
          (string? x) ::string
          (jt/local-date? x) ::local-date
          (number? x) ::number
          (keyword? x) ::keyword
          (char? x) ::char
          (boolean? x) ::boolean
          :else ::unsupported)))

(defn- try-sort
  "Sort if possible, otherwise don't"
  [xs]
  (try (sort xs) (catch ClassCastException _ xs)))

(defmethod cell ::map
  [x]
  (let [keys (try-sort (vec (keys x)))]
    (stack (mapv (fn [k] (row [(cell k) (cell (get x k))] SPACE-MEDIUM))
             keys))))

(defmethod cell ::vector
  [x]
  (case (count x)
    0 EMPTY
    1 (cell (first x))
    (stack (mapv cell x))))

(defmethod cell ::set
  [x]
  (let [items (try-sort (vec x))]
    (case (count items)
      0 EMPTY
      1 (cell (first items))
      (stack (mapv cell items)))))

(defmethod cell ::string [x] (align-left x))

(defmethod cell ::local-date [x] (cell (str x)))

(defmethod cell ::number
  [x]
  (let [s (str x)
        dp (or (str/index-of s ".") (str/index-of s "/") (count s))]
    (anchored s (dec dp))))

(defmethod cell ::keyword [x] (align-left (str x)))

(defmethod cell ::char [x] (align-left (str x)))

(defmethod cell ::boolean [x] (align-left (str x)))

(defmethod cell ::nil [_] EMPTY)

(defmethod cell ::unsupported [_] (cell "?"))
