(ns lima.cli
  (:require [cli-matic.core :as cli-matic]
            [lima.adapter.beanfile :as beanfile]
            [lima.adapter.tabulate :as tabulate]
            [lima.core.inventory :as inv]))

(defn report
  "Run the named report"
  [{:keys [name beanpath]}]
  (case name
    "count" (let [{:keys [directives options]} (beanfile/book beanpath)
                  inv (inv/build directives options)
                  tab (tabulate/inventory inv)]
              (println tab))))

(def CONFIGURATION
  {:command "lima",
   :description "A new implementation of Beancount in Clojure/Rust",
   :version "0.0.1",
   :subcommands [{:command "report",
                  :description "Run a canned Lima report",
                  :opts [{:as "Name",
                          :option "name",
                          :short 0,
                          :type #{"count"},
                          :default "count"}
                         {:as "Beancount file path",
                          :option "beanpath",
                          :short 1,
                          :type :string,
                          :env "LIMA_BEANPATH",
                          :default :present}],
                  :runs report}]})
