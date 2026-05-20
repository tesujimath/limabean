(ns limabean.test.plugins.tag-matching
  (:require [limabean.plugin :as plugin]))

(defn raw-xf
  "Transducer on raw directives to tag matching directives."
  [{:keys [config]}]
  (let [matching (or (:matching config) {:unmatched true})
        keys (vec (keys matching))
        tag (or (:tag config) "auto-tag")]
    (fn [rf]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result dct]
         (let [dct' (if (= (select-keys dct keys) matching)
                      (update dct :tags #(conj (or % #{}) tag))
                      dct)]
           (rf result dct')))))))
