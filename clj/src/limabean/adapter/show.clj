(ns limabean.adapter.show
  (:require [limabean.adapter.tabulate :refer [render]]
            [limabean.core.cell :refer [cell]]))

(defn show
  "Render x as a cell and print it"
  [x]
  (print (render (cell x)))
  (flush)
  :ok)
