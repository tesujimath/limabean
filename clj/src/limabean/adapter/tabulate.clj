(ns limabean.adapter.tabulate
  (:require [clojure.java.shell :as shell]
            [cheshire.core :as cheshire]
            [limabean.core.cell :as cell]))

(defn tabulate-cell
  "Tabulate a cell using limabean-pod"
  [cell]
  (let [cell-json (cheshire/generate-string cell)
        tabulated (shell/sh "limabean-pod" "tabulate" :in cell-json)]
    (if (= (tabulated :exit) 0)
      (tabulated :out)
      (do (println "limabean-pod error" (tabulated :err))
          (throw (Exception. "limabean-pod failed"))))))


(defn inventory
  "Tabulate an inventory using limabean-pod"
  [inv]
  (tabulate-cell (cell/inventory->cell inv)))

(defn register
  "Tabulate a register using limabean-pod"
  [reg]
  (tabulate-cell (cell/register->cell reg)))
