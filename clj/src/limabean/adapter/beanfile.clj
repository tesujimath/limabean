(ns limabean.adapter.beanfile
  (:require [clojure.edn :as edn]
            [clojure.java.shell :as shell]
            [java-time.api :as jt]
            [limabean.core.inventory :as inv]))

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

(defn digest
  "Read EDN from limabean-pod digest and return or throw"
  [beancount-path]
  (let [digested (shell/sh "limabean-pod" "digest" beancount-path)]
    (if (= (digested :exit) 0)
      (read-edn-string (digested :out))
      (do (println "limabean-pod error" (digested :err))
          (throw (Exception. "limabean-pod failed"))))))

(defn inventory
  "Read EDN from limabean-pod book and return or throw"
  [beancount-path]
  (let [booked (shell/sh "limabean-pod" "book" "-f" "edn" beancount-path)]
    (if (= (booked :exit) 0)
      (let [{:keys [directives options]} (read-edn-string (booked :out))]
        (inv/build directives options))
      (do (println "limabean-pod error" (booked :err))
          (throw (Exception. "limabean-pod failed"))))))
