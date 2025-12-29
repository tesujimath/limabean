(ns lima.core.inventory-test
  (:require [clojure.test :refer :all]
            [lima.core.inventory :refer :all]))

(defn cmp-eq? [cmp] (= cmp 0))
(defn cmp-lt? [cmp] (< cmp 0))
(defn cmp-gt? [cmp] (> cmp 0))

(deftest compare-nil-first-test
  (testing "nil-first"
    (is (cmp-lt? (compare-nil-first 1 2)))
    (is (cmp-lt? (compare-nil-first nil 2)))
    (is (cmp-gt? (compare-nil-first 1 nil)))
    (is (cmp-eq? (compare-nil-first nil nil)))))
