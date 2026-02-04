(ns limabean.core.filters-test
  (:require [java-time.api :as jt]
            [limabean.core.filters :as sut]
            [clojure.test.check.generators :as gen]
            [clojure.test.check.properties :as prop]
            [clojure.test :refer [deftest is]]
            [clojure.test.check.clojure-test :refer [defspec]])
  (:import [java.time LocalDate]))

(def constrained-date-gen
  "Generate dates with a deliberately small range, so we get a good number of same-date"
  (gen/fmap (fn [days-since-epoch] (LocalDate/ofEpochDay days-since-epoch))
            (gen/choose 20000 20003)))

(defspec
  date-test
  20
  (prop/for-all [target-date constrained-date-gen
                 date constrained-date-gen]
    (let [x {:date date}]
      (and (if (jt/before? date target-date)
             (and ((sut/date< target-date) x) ((sut/date<= target-date) x))
             true)
           (if (= date target-date)
             (and ((sut/date<= target-date) x) ((sut/date>= target-date) x))
             true)
           (if (jt/after? date target-date)
             (and ((sut/date> target-date) x) ((sut/date>= target-date) x))
             true)))))

(deftest date<-test
  (is ((sut/date< "2025-10-15") {:date (jt/local-date "2025-10-14")}))
  (is ((sut/date< 2025 10 15) {:date (jt/local-date "2025-10-14")}))
  (is ((sut/date< 2025 11) {:date (jt/local-date "2025-10-14")}))
  (is ((sut/date< 2026) {:date (jt/local-date "2025-10-14")})))

(deftest date>=<-test
  (is ((sut/date>=< "2025-10-14" "2025-10-15")
        {:date (jt/local-date "2025-10-14")}))
  (is ((sut/date>=< 2025 10 14 2025 10 15)
        {:date (jt/local-date "2025-10-14")}))
  (is ((sut/date>=< 2025 10 2025 11) {:date (jt/local-date "2025-10-14")}))
  (is ((sut/date>=< 2025 2026) {:date (jt/local-date "2025-10-14")})))
