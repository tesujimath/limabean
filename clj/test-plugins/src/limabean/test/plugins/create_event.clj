(ns limabean.test.plugins.create-event)

(defn booked-xf
  "Transducer on booked directives to synthesize planning events, for testing."
  [_]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (let [result' (if (= :event (:dct dct))
                       (rf result
                           (-> (select-keys dct [:date :dct])
                               (assoc :type "Planning"
                                      :description (str "Anticipating "
                                                        (:description dct)))))
                       result)]
         (rf result' dct))))))
