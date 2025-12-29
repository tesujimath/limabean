(ns lima.adapter.beanfile
  (:require [clojure.edn :as edn]
            [clojure.java.shell :as shell]
            [java-time.api :as jt]
            [lima.core.inventory :as inv]))

(def readers {'time/date #(jt/local-date %)})

(defn read-edn-string
  "Read string as LIma PP EDN"
  [s]
  (edn/read-string {:readers readers} s))

(defn book
  "Read EDN from lima-pod book and return or throw"
  [beancount-path]
  (let [booked (shell/sh "lima-pod" "book" "-f" "edn" beancount-path)]
    (if (= (booked :exit) 0)
      (read-edn-string (booked :out))
      (do (println "lima-pod error" (booked :err))
          (throw (Exception. "lima-pod failed"))))))

(defn inventory
  "Read EDN from lima-pod book and return or throw"
  [beancount-path]
  (let [booked (shell/sh "lima-pod" "book" "-f" "edn" beancount-path)]
    (if (= (booked :exit) 0)
      (let [{:keys [directives options]} (read-edn-string (booked :out))]
        (inv/build directives options))
      (do (println "lima-pod error" (booked :err))
          (throw (Exception. "lima-pod failed"))))))
