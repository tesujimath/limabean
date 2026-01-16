(ns limabean.adapter.tabulate
  (:require [clojure.java.shell :as shell]
            [cheshire.core :as cheshire]))

(defn render
  "Render a cell using limabean-pod"
  [cell]
  (let [cell-json (cheshire/generate-string cell)
        tabulated (shell/sh "limabean-pod" "tabulate" :in cell-json)]
    (if (= (tabulated :exit) 0)
      (tabulated :out)
      (do (println "limabean-pod error" (tabulated :err))
          (throw (Exception. "limabean-pod failed"))))))
