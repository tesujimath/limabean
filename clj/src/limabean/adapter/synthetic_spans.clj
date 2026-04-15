(ns limabean.adapter.synthetic-spans
  (:require [limabean.core.format :as format]
            [clojure.string :as str]
            [limabean.adapter.pod :as pod]))

(defn- pop-span!
  "Pop the first span from the front of the volatile spans"
  [spans]
  (let [[span & remaining] @spans]
    (vreset! spans remaining)
    span))

(defn- merge-with-directives
  "Stateful transducer to merge synthetic spans back in with directives"
  [synthetic-spans]
  (fn [rf]
    (let [spans (volatile! synthetic-spans)]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result dct]
         (if (:provenance dct)
           (let [dct' (if (= (:dct dct) :txn)
                        ;; merge txn along with its posting spans
                        (assoc dct
                          :span (pop-span! spans)
                          :postings (mapv (fn [pst]
                                            (assoc pst :span (pop-span! spans)))
                                      (:postings dct)))
                        ;; non-transaction
                        (assoc dct :span (pop-span! spans)))]
             (rf result dct'))
           (rf result dct)))))))

(defn create-with-provenance
  "Create synthetic spans for all directives/postings with provenance, if required."
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

(defn create-and-merge-with-provenance
  "Create and merge synthetic spans for all directives/postings with provenance, if required."
  [directives pod]
  (let [synthetic-spans (create-with-provenance directives pod)]
    (if (seq synthetic-spans)
      (into [] (merge-with-directives synthetic-spans) directives)
      directives)))
