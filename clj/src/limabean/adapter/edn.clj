(ns limabean.adapter.edn
  (:require [clojure.edn :as edn]
            [java-time.api :as jt]))

(def readers {'time/date #(jt/local-date %)})

(defn read-edn-string
  "Read string as limabean PP EDN"
  [s]
  (edn/read-string {:readers readers} s))

;; make printing LocalDate use the same form
(defmethod print-method java.time.LocalDate
  [v w]
  (.write w (str "#time/date \"" v "\"")))
