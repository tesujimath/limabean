(ns limabean.adapter.show
  (:require [clojure.pprint :refer [pprint]]
            [limabean.adapter.tabulate :refer [render]]
            [limabean.core.cell :refer [cell]]
            [taoensso.telemere :as tel]))

(defn show
  "Show anything which can be rendered as a cell, with fallback to pprint"
  [x]
  (let [c (cell x)]
    (if c
      (let [_ (tel/log! {:id ::show-cell, :data c})
            r (render c)
            _ (tel/log! {:id ::show-rendered-cell, :data r})]
        (print r))
      (let [_ (tel/log! {:id ::show-pprint, :data x})] (pprint x))))
  :ok)
