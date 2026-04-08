(ns limabean.test-support
  (:require [clojure.java.io :as io]
            [clojure.string :as str]
            [clojure.walk :as walk]))

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
       (map (fn [beanfile-name]
              (let [name (str/replace beanfile-name ".beancount" "")
                    beanfile (.getPath (io/file TEST-CASES-DIR beanfile-name))
                    golden-dir (io/file TEST-CASES-DIR
                                        (format "%s.golden" name))]
                {:name name, :beanfile beanfile, :golden-dir golden-dir})))
       (filter #(.exists (:golden-dir %)))))

(defn remove-spans-and-indexes
  "Remove spans and indexes from all maps"
  [data]
  (walk/postwalk (fn [x]
                   (cond-> x
                     (and (map? x) (contains? x :span)) (dissoc :span)
                     (and (map? x) (contains? x :raw-idx)) (dissoc :raw-idx)))
                 data))
