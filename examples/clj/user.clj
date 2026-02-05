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
