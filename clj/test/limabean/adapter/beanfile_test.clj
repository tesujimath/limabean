(ns limabean.adapter.beanfile-test
  (:require [limabean.adapter.beanfile :as sut]
            [clojure.java.io :as io]
            [clojure.test :refer [deftest is testing]]
            [limabean.app-test :as app-test]))

(deftest beanfile-tests
  (doseq [{:keys [name beanfile golden-dir]} (app-test/get-tests)]
    (testing name
      (let [actual (sut/book beanfile)
            expected-directives (io/file golden-dir "directives.edn")]
        (when (.exists expected-directives)
          (let [expected (sut/read-edn-string (slurp expected-directives))]
            (is (= (:directives actual) expected))))))))
