(ns limabean.core.validation)

(def ^{:private true} DEFAULT-INFERRED-TOLERANCE-MULTIPLIER 0.5M)

(defn- sum-with-min-nonzero-scale
  [{:keys [sum scale]} x]
  (let [sum' (+ (or sum 0) x)
        x_scale (if (decimal? x) (.scale x) 0)
        scale' (cond (zero? x_scale) scale
                     (nil? scale) x_scale
                     :else (min scale x_scale))]
    {:sum sum', :scale scale'}))

(defn- scaled-unit
  "BigDecimal unit at scale"
  [scale]
  (BigDecimal. (biginteger 1) (int scale)))

(defn- txn-balances?
  [txn multipler cur-tol]
  (println "checking balance for txn" (:date txn) (:payee txn) (:narration txn))
  (let [sum-scale-by-cur
          (reduce (fn [m pst]
                    (update m
                            (:cur pst)
                            #(sum-with-min-nonzero-scale % (:units pst))))
            {}
            (:postings txn))
        _ (println "sum-scale-by-cur for"
                   (mapv #(select-keys % [:units :cur]) (:postings txn))
                   sum-scale-by-cur)
        residual (into {}
                       (keep (fn [[cur {:keys [sum scale]}]]
                               (let [abs_sum (abs sum)]
                                 (when (or (and scale
                                                (> abs_sum (scaled-unit scale)))
                                           (when-let [tol (cur-tol cur)]
                                             (> abs_sum tol))
                                           (> abs_sum 0))
                                   [cur sum])))
                             sum-scale-by-cur))]))

(defn- txn-balance-xf
  "Validate that transactions balance.

   Only transactions with provenance are checked, since these were created by plugins.
   Others are assumed to be OK, since they were produced from the core booking algorithm."
  [options]
  (let [multiplier (or (:inferred_tolerance_multiplier options)
                       DEFAULT-INFERRED-TOLERANCE-MULTIPLIER)
        inferred-tolerance-default (:inferred-tolerance-default options)
        fallback (:inferred-tolerance-default-fallback options)
        cur-tol (fn [cur] (get inferred-tolerance-default cur fallback))]
    (fn [rf]
      (fn
        ;; init
        ([] (rf))
        ;; completion
        ([result] (rf result))
        ;; step
        ([result dct]
         (when (and (= :txn (:dct dct)) (:provenance dct))
           (txn-balances? dct multiplier cur-tol))
         (rf result dct))))))

(defn post-booking-xf
  "Post-booking validation transducer"
  [options]
  (comp (txn-balance-xf options)))
