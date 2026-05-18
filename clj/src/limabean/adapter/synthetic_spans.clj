(ns limabean.adapter.synthetic-spans
  (:require [clojure.string :as str]
            [limabean.adapter.pod :as pod]
            [limabean.core.format :as format]
            [limabean.util :as util]))

(defn- create-with-provenance
  "Create synthetic spans for all directives/postings with provenance."
  [directives pod]
  (let [synthetic-span-requests
          (into []
                (comp (filter :provenance)
                      (format/elements-xf
                        (fn [dct s]
                          {:name (str "Synthetic directive from "
                                      (str/join " " (:provenance dct))),
                           :content s})))
                directives)]
    (if (seq synthetic-span-requests)
      (pod/create-synthetic-spans pod synthetic-span-requests)
      [])))

(defn- n-spans
  "Determine how many synthetic spans need merging"
  [dct]
  (if (:provenance dct) (inc (count (:postings dct))) 0))

(defn merge-with-provenance
  "Create and merge synthetic spans as :span-p for all directives/postings with provenance."
  [directives pod]
  (let [synthetic-spans (create-with-provenance directives pod)]
    (if (seq synthetic-spans)
      (vec (util/map-n n-spans
                       (fn [dct spans]
                         (let [dct-span (first spans)
                               pst-spans (rest spans)]
                           (cond-> (assoc dct :span-p dct-span)
                             (seq pst-spans) (assoc :postings
                                               (mapv #(assoc %1 :span-p %2)
                                                 (:postings dct)
                                                 pst-spans)))))
                       directives
                       synthetic-spans))
      directives)))
