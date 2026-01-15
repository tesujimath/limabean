(ns limabean.main
  (:require [cli-matic.core :refer [run-cmd]]
            [limabean.app :as app]
            [limabean.adapter.logging :as logging])
  (:gen-class))



(def CONFIGURATION
  {:command "limabean",
   :description "A new implementation of Beancount in Clojure/Rust",
   :version "0.0.1",
   :subcommands [{:command "report",
                  :description "Run a canned limabean report",
                  :opts [{:as "Name",
                          :option "name",
                          :short 0,
                          :type app/reports,
                          :default app/default-report}
                         {:as "Beancount file path",
                          :option "beanpath",
                          :short 1,
                          :type :string,
                          :env "LIMA_BEANPATH",
                          :default :present}],
                  :runs app/report}]})


(defn -main [& args] (logging/initialize) (run-cmd args CONFIGURATION))
