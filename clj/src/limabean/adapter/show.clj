(ns limabean.adapter.show
  (:require [clojure.pprint :refer [pprint]]
            [limabean.adapter.tabulate :refer [render]]
            [limabean.core.tabulate :refer [tabulate]]))

(defn show
  "Generic pretty printer"
  [x]
  (let [cell (tabulate x)] (if cell (println (render cell)) (pprint x)))
  :ok)
