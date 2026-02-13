(ns limabean.app-test
  (:require [limabean.app :as sut]
            [clojure.java.io :as io]
            [clojure.java.shell :as shell]
            [clojure.string :as str]
            [clojure.test :refer [deftest is testing]])
  (:import [java.nio.file Files]))

(def TEST-CASES-DIR "../test-cases")

(defn- sorted-dir-entries
  "Return a sorted list of files in `dir`, an `io/file`"
  [dir]
  (let [unsorted (.list dir)] (sort (vec unsorted))))

(defn get-tests
  "Look for beancount files in test-cases to generate test base paths"
  []
  (->> (sorted-dir-entries (io/file TEST-CASES-DIR))
       (filter #(str/ends-with? % ".beancount"))
       (mapv (fn [beanfile-name]
               (let [name (str/replace beanfile-name ".beancount" "")
                     beanfile (.getPath (io/file TEST-CASES-DIR beanfile-name))
                     golden-dir (io/file TEST-CASES-DIR
                                         (format "%s.golden" name))]
                 {:name name, :beanfile beanfile, :golden-dir golden-dir})))))

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
      (binding [*out* *err*]
        (println "unexpected diff failure, exit code"
                 (:exit diff)
                 (:err diff))))))

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
  (doseq [{:keys [name beanfile golden-dir]} (get-tests)]
    (testing name
      (doseq [query ["inventory" "rollup" "journal"]]
        (let [actual (temp-file-path name query)
              expected (.getPath (io/file golden-dir query))]
          (with-open [w (io/writer actual)]
            (binding [*out* w]
              (sut/run {:beanfile beanfile,
                        :eval (format "(show (%s))" query)})))
          (is (golden (format "%s.%s" name query) actual expected)))))))
