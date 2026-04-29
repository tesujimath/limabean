(ns limabean.app-test
  (:require [limabean.test]
            [clojure.test :refer [deftest]]))

(def TEST-CASES "../test-cases")

(deftest app-tests (limabean.test/app-tests TEST-CASES))

(deftest loader-tests (limabean.test/loader-tests TEST-CASES))
