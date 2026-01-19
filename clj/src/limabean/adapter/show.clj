(ns limabean.adapter.show
  (:require [clojure.pprint :refer [pprint]]
            [limabean.adapter.tabulate :refer [render]]
            [limabean.core.cell :refer [cell]]))

(defn show
  "Show anything which can be rendered as a cell, with fallback to pprint"
  [x]
  (let [c (cell x)] (if c (println (render c)) (pprint x)))
  :ok)
