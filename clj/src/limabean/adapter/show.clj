(ns limabean.adapter.show
  (:require [limabean.adapter.tabulate :refer [render]]
            [limabean.core.cell :refer [cell]]))

(defn show
  "Convert `x` to a cell and tabulate it."
  [x]
  (print (render (cell x)))
  (flush)
  :ok)
