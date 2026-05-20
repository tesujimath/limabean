(ns limabean.test.plugins.fail
  (:require [limabean.plugin :as plugin]))

(defn- xf
  "Transducer on raw/booked directives to fail on matching directives according to phase."
  [{:keys [config phase]}]
  (let [matching
          (if (= phase (:phase config)) (:matching config) {:unmatched true})
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
         (let [dct' (if (= (select-keys dct keys) matching)
                      (plugin/dct-error! dct message)
                      dct)]
           (rf result dct')))))))

(defn raw-xf
  "Transducer on raw directives to fail on matching directives if phase is raw."
  [args]
  (xf (assoc args :phase :raw)))

(defn booked-xf
  "Transducer on booked directives to fail on matching directives if phase is booked."
  [args]
  (xf (assoc args :phase :booked)))
