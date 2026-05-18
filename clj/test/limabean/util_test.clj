(ns limabean.util-test
  (:require [limabean.util :as sut]
            [clojure.test :refer [deftest is]]))

(deftest map-if-test
  (is (= (sut/map-if even? + [1 2 3 4] [10 20]) [1 12 3 24])))

(deftest map-n-test
  (is (= (sut/map-n :n
                    #(assoc %1 :items (vec %2))
                    [{:n 0} {:n 2} {:n 1}]
                    [1 2 3 4 5 6 7 8 9 10])
         [{:n 0} {:n 2, :items [1 2]} {:n 1, :items [3]}])))
