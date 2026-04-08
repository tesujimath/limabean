(ns limabean.app-test
  (:require [limabean.app :as sut]
            [limabean.adapter.edn :as edn]
            [limabean.adapter.loader :as loader]
            [limabean.test-support :as test-support]
            [clojure.java.io :as io]
            [clojure.java.shell :as shell]
            [clojure.test :refer [deftest is testing]]
            [matcho.core :as matcho])
  (:import [java.nio.file Files]))

(defn temp-file-path
  [prefix ext]
  (str (Files/createTempFile prefix
                             ext
                             (make-array java.nio.file.attribute.FileAttribute
                                         0))))

(defn diff
  "Return diff as a string, or nil if no diffs"
  [actual expected]
  (let [diff (shell/sh "diff" actual expected)]
    (case (:exit diff)
      0 nil
      1 (:out diff)
      (throw (Exception. (str "unexpected diff failure, exit code"
                              (:exit diff)
                              (:err diff)))))))

(defn golden
  "Golden test of actual and expected paths"
  [name actual expected]
  (let [diffs (diff actual expected)]
    (if diffs
      (do
        (println
          (format
            "%s actual != expected\n====================\n%s\n====================\n"
            name
            diffs))
        false)
      true)))

(deftest app-tests
  (doseq [{:keys [name beanfile golden-dir]} (test-support/get-tests)]
    (testing name
      (doseq [query ["inventory" "rollup" "journal"]]
        (let [actual (temp-file-path name query)
              expected (io/file golden-dir query)]
          (when (.exists expected)
            (with-open [w (io/writer actual)]
              (binding [*out* w]
                (sut/run {:beanfile beanfile,
                          :eval (format "(show (%s))" query)})))
            (is (golden (format "%s.%s" name query)
                        actual
                        (.getPath expected)))))))))

(deftest beanfile-tests
  (doseq [{:keys [name beanfile golden-dir]} (test-support/get-tests)]
    (testing name
      (let [expected-directives (io/file golden-dir "directives.edn")]
        (when (.exists expected-directives)
          (let [actual (try (println "loading" beanfile "to check directives")
                            (loader/load-beanfile beanfile)
                            (catch Exception e
                              (println "Exception while processing"
                                       beanfile
                                       (.getMessage e))
                              []))
                expected (edn/read-edn-string (slurp expected-directives))]
            (matcho/assert expected
                           (test-support/remove-spans-and-indexes
                             (:directives actual)))))))))
