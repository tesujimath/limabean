(ns limabean.adapter.beanfile
  (:require [clojure.edn :as edn]
            [java-time.api :as jt]
            [limabean.adapter.shell :as shell]))

(def readers {'time/date #(jt/local-date %)})

(defn read-edn-string
  "Read string as limabean PP EDN"
  [s]
  (edn/read-string {:readers readers} s))

(defn book
  "Read EDN from limabean-pod book and return or throw"
  [beancount-path]
  (let [booked (shell/try-sh "limabean-pod" "book" "-f" "edn" beancount-path)]
    (read-edn-string booked)))
