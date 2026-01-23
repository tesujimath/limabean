(ns limabean.adapter.tabulate
  (:require [cheshire.core :as cheshire]
            [limabean.adapter.shell :as shell]))

(defn render
  "Render a cell using limabean-pod"
  [cell]
  (let [cell-json (cheshire/generate-string cell)
        tabulated (shell/try-sh "limabean-pod" "tabulate" :in cell-json)]
    tabulated))
