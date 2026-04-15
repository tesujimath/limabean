(ns limabean.test.plugins.fail
  (:require [limabean.plugin :as plugin]))

(defn raw-xf
  "Transducer on raw directives to fail on matching directives."
  [{:keys [config options]}]
  (let [matching (or (:matching config) {:unmatched true})
        keys (vec (keys matching))
        message (or (:message config) "bad directive")]
    (fn [rf]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result dct]
         (if (= (select-keys dct keys) matching)
           (plugin/error! dct message)
           (rf result dct)))))))
