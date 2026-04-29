(ns limabean.test
  (:require [clojure.java.io :as io]
            [clojure.java.shell :as shell]
            [clojure.pprint :refer [pprint]]
            [clojure.string :as str]
            [clojure.test :refer [is testing]]
            [clojure.walk :as walk]
            [limabean.adapter.edn :as edn]
            [limabean.adapter.json]
            [limabean.adapter.loader :as loader]
            [limabean.adapter.print]
            [limabean.app :as app]
            [matcho.core :as matcho])
  (:import [java.nio.file Files]))

(defn remove-spans-and-indexes
  "Remove spans and indexes from all maps"
  [data]
  (walk/postwalk (fn [x]
                   (cond-> x
                     (and (map? x) (contains? x :span)) (dissoc :span)
                     (and (map? x) (contains? x :raw-idx)) (dissoc :raw-idx)))
                 data))

(defn find-golden-tests
  "Walk the filesystem from root-dir looking for beancount files and golden directories."
  [root-dir]
  (let [root-dir (.getPath (io/file root-dir))]
    (into []
          (comp (filter #(str/ends-with? (.getName %) ".beancount"))
                (map (fn [beanfile]
                       (let [base-path (io/file (str/replace (.getPath beanfile) #".beancount$" ""))
                             test-name (.getName base-path)
                             golden-dir (io/file (str base-path ".golden"))]
                         {:test-name test-name,
                          :beanfile (.getPath beanfile),
                          :golden-dir golden-dir})))
                (filter #(.exists (:golden-dir %))))
          (file-seq (io/file root-dir))))
  )


(defn- temp-file-path
  [prefix ext]
  (str (Files/createTempFile prefix
                             ext
                             (make-array java.nio.file.attribute.FileAttribute
                                         0))))

(defn- diff
  "Return diff as a string, or nil if no diffs"
  [actual expected]
  (let [diff (shell/sh "diff" actual expected)]
    (case (:exit diff)
      0 nil
      1 (:out diff)
      (throw (Exception. (str "unexpected diff failure, exit code"
                              (:exit diff)
                              (:err diff)))))))

(defn- golden-text
  "Golden test of actual and expected paths"
  [test-name actual expected]
  (let [diffs (diff actual expected)]
    (if diffs
      (do
        (println
          (format
            "%s actual != expected\n====================\n%s\n====================\n"
            test-name
            diffs))
        false)
      true)))

(defn app-tests
  [root-dir]
  (doseq [{:keys [test-name beanfile golden-dir]} (find-golden-tests root-dir)]
    (testing test-name
      (doseq [query ["inventory" "rollup" "journal"]]
        (let [actual (temp-file-path test-name query)
              expected (io/file golden-dir query)
              query-expr (case query
                           "rollup" "(show (rollup (inventory)))"
                           (format "(show (%s))" query))]
          (when (.exists expected)
            (with-open [w (io/writer actual)]
              (binding [*out* w]
                (app/run {:beanfile beanfile, :eval query-expr})))
            (is (golden-text (format "%s.%s" test-name query)
                             actual
                             (.getPath expected)))))))))

(defn loader-tests
  [root-dir]
  (doseq [{:keys [test-name beanfile golden-dir]} (find-golden-tests root-dir)]
    (testing test-name
      (let [beans (delay (try (println "loading" beanfile)
                              (loader/load-beanfile beanfile)
                              (catch Exception e
                                (println "Exception while processing"
                                         beanfile
                                         (.getMessage e))
                                (pprint (Throwable->map e))
                                nil)))]
        (doseq [key [:raw-xf-directives :directives]]
          (let [expected-file (io/file golden-dir (str (name key) ".edn"))]
            (when (.exists expected-file)
              (let [actual (force beans)
                    expected (edn/read-edn-string (slurp expected-file))
                    expected-strict (walk/postwalk
                                      (fn [x]
                                        (if (instance? clojure.lang.IObj x)
                                          (with-meta x {:matcho/strict true})
                                          x))
                                      expected)]
                (matcho/assert expected-strict
                               (limabean.test/remove-spans-and-indexes
                                 (get actual key)))))))))))
