(ns limabean.test.plugins.identity)

(defn raw-xf
  "Identity raw plugin"
  [_]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct] (rf result dct)))))

(defn booked-xf
  "Identity booked plugin"
  [_]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct] (rf result dct)))))
