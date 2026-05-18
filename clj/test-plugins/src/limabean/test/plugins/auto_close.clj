(ns limabean.test.plugins.auto-close)

(defn- xf
  "Transducer on raw/booked directives to immediately close any account in the config hash set according to phase."
  [{:keys [config phase]}]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (if (and (= phase (:phase config))
                (= (:dct dct) :open)
                (contains? (:accs config) (:acc dct)))
         (do
           ;; emit the original open
           (rf result dct)
           ;; now close that account if it's one we're watching for
           (rf result {:date (:date dct), :dct :close, :acc (:acc dct)}))
         ;; otherwise emit the original directive, whatever it was
         (rf result dct))))))


(defn raw-xf
  "Transducer on raw directives to immediately close any account in the config hash set if phase is raw."
  [args]
  (xf (assoc args :phase :raw)))

(defn booked-xf
  "Transducer on booked directives to immediately close any account in the config hash set if phase is booked."
  [args]
  (xf (assoc args :phase :booked)))
