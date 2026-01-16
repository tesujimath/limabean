(ns limabean.adapter.beanfile
  (:require [clojure.edn :as edn]
            [clojure.java.shell :as shell]
            [java-time.api :as jt]))

(def readers {'time/date #(jt/local-date %)})

(defn read-edn-string
  "Read string as limabean PP EDN"
  [s]
  (edn/read-string {:readers readers} s))

(defn book
  "Read EDN from limabean-pod book and return or throw"
  [beancount-path]
  (let [booked (shell/sh "limabean-pod" "book" "-f" "edn" beancount-path)]
    (if (= (booked :exit) 0)
      (read-edn-string (booked :out))
      (do (println "limabean-pod error" (booked :err))
          (throw (Exception. "limabean-pod failed"))))))
