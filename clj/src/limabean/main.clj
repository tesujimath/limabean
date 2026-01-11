(ns limabean.main
  (:require [cli-matic.core :refer [run-cmd]])
  (:require [limabean.cli :refer [CONFIGURATION]])
  (:gen-class))

(defn -main [& args] (run-cmd args CONFIGURATION))
