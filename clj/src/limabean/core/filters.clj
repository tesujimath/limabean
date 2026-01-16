(ns limabean.core.filters
  (:require [java-time.api :as jt]
            [clojure.string :as str]))

(defn ensure-local-date
  "Ensure x is a local-data, by converting if not"
  [x]
  (cond (jt/local-date? x) x
        (string? x) (jt/local-date x)
        :else (throw (ex-info (format "Unsupported type %s for date predicate"
                                      (type x))
                              {:type :limabean.harvest/error-type}))))

(defn date<
  "Predicate for :date field to be < begin-date, or false if no date field"
  [end-date]
  (let [end-date (ensure-local-date end-date)]
    #(let [date (:date %)] (and date (jt/before? date end-date)))))

(defn date<=
  "Predicate for :date field to be <= end-date, or false if no date field"
  [end-date]
  (let [end-date (ensure-local-date end-date)]
    #(let [date (:date %)] (and date (jt/not-after? date end-date)))))

(defn date>
  "Predicate for :date field to be > begin-date, or false if no date field"
  [begin-date]
  (let [begin-date (ensure-local-date begin-date)]
    #(let [date (:date %)] (and date (jt/after? date begin-date)))))

(defn date>=
  "Predicate for :date field to be >= begin-date, or false if no date field"
  [begin-date]
  (let [begin-date (ensure-local-date begin-date)]
    #(let [date (:date %)] (and date (jt/not-before? date begin-date)))))

(defn date-between
  "Predicate for :date field to be >= begin-date and < end-date, or false if no date field"
  [begin-date end-date]
  (let [begin-date (ensure-local-date begin-date)
        end-date (ensure-local-date end-date)]
    #(let [date (:date %)]
       (and date (jt/not-before? date begin-date) (jt/before? date end-date)))))

(defn some-acc
  "Predicate for :acc field to be equal to one of target-accs, or false if no acc field"
  [& target-accs]
  #(let [acc (:acc %)] (and acc (contains? (set target-accs) acc))))

(defn some-sub-acc
  "Predicate for :acc field to be equal to acc or a subaccount of it, or false if no acc field"
  [& target-accs]
  #(let [acc (:acc %)]
     (and acc
          (boolean (some (fn [target-acc]
                           (or (= acc target-acc)
                               (str/starts-with? acc (str target-acc ":"))))
                         target-accs)))))

(defn- field-match
  "Helper for whether the given field matches the regex"
  [key regex]
  #(let [field (get % key)] (and field (seq (re-seq regex field)) true)))

(defn payee-match
  "Predicate for whether the payee matches the given regex"
  [regex]
  (field-match :payee regex))

(defn narration-match
  "Predicate for whether the narration matches the given regex"
  [regex]
  (field-match :narration regex))
