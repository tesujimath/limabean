(ns user
  (:require [java-time.api :as jt]
            [limabean.core.filters :as f]))

(defn fy
  "Example of financial year date filter"
  [year]
  (let [year (if (< year 100) (+ 2000 year) year)]
    (f/every-f (f/date>= (jt/local-date year 4 1))
               (f/date< (jt/local-date (inc year) 4 1)))))
