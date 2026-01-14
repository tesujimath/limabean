(ns limabean.core.filters
  (:require [java-time.api :as jt]
            [clojure.string :as str]))

(defn date<
  "Predicate for :date field to be < begin-date, or false if no date field"
  [end-date-str]
  (let [end-date (jt/local-date end-date-str)]
    #(let [date (:date %)] (and date (jt/before? date end-date)))))

(defn date<=
  "Predicate for :date field to be <= end-date, or false if no date field"
  [end-date-str]
  (let [end-date (jt/local-date end-date-str)]
    #(let [date (:date %)] (and date (jt/not-after? date end-date)))))

(defn date>
  "Predicate for :date field to be > begin-date, or false if no date field"
  [begin-date-str]
  (let [begin-date (jt/local-date begin-date-str)]
    #(let [date (:date %)] (and date (jt/after? date begin-date)))))

(defn date>=
  "Predicate for :date field to be >= begin-date, or false if no date field"
  [begin-date-str]
  (let [begin-date (jt/local-date begin-date-str)]
    #(let [date (:date %)] (and date (jt/not-before? date begin-date)))))

(defn acc=
  "Predicate for :acc field to be equal to acc, or false if no acc field"
  [target-acc]
  #(let [acc (:acc %)]
     (and acc)
     (= acc target-acc)))

(defn acc-sub
  "Predicate for :acc field to be equal to acc or a subaccount of it, or false if no acc field"
  [target-acc]
  #(let [acc (:acc %)]
     (and acc)
     (or (= acc target-acc) (str/starts-with? acc (str target-acc ":")))))
