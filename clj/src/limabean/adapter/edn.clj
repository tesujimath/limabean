(ns limabean.adapter.edn
  (:require [clojure.edn :as edn]
            [java-time.api :as jt]
            [limabean.adapter.print]))

(def readers {'time/date #(jt/local-date %)})

(defn read-edn-string
  "Read string as limabean PP EDN"
  [s]
  (edn/read-string {:readers readers} s))
