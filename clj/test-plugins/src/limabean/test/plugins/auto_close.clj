(ns limabean.test.plugins.auto-close)

(defn raw-xf
  "Transducer on raw directives to immediately close any account in the config hash set."
  [{:keys [config options]}]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (if (and (= (:dct dct) :open) (contains? config (:acc dct)))
         (do
           ;; emit the original open
           (rf result dct)
           ;; now close that account if it's one we're watching for
           (rf result {:date (:date dct), :dct :close, :acc (:acc dct)}))
         ;; otherwise emit the original directive, whatever it was
         (rf result dct))))))
