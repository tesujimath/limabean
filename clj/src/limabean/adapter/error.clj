(ns limabean.adapter.error
  (:require [limabean.adapter.pod :as pod]))

(defn- resolve-idx
  [[dct-idx pst-idx] directives]
  (let [dct (get directives dct-idx)
        pst (get (:postings dct) pst-idx)]
    (cond-> {:kind (if pst "posting" (name (:dct dct))),
             :span (or (:span pst) (:span dct))}
      pst (assoc :context ["txn" (:span dct)]))))

(defn- resolve-related
  [directives]
  (fn [idx]
    (let [{:keys [kind span]} (resolve-idx idx directives)] [kind span])))

(defn- resolve-indexed-report
  [report directives]
  (let [{:keys [kind span context]} (resolve-idx (:idx report) directives)]
    (cond-> (assoc (select-keys report [:reason :annotation])
              :message (str "invalid " kind)
              :span span)
      context (assoc :context context)
      (:related report) (assoc :related
                          (mapv (resolve-related directives)
                            (:related report))))))

(defn- resolve-indexed-reports
  [reports directives]
  (map #(resolve-indexed-report % directives) reports))

(defn- print-error
  [err directives pod]
  (let [{:keys [spanned-reports indexed-reports message exception]} err
        resolved-reports (or spanned-reports
                             (and indexed-reports
                                  (resolve-indexed-reports indexed-reports
                                                           directives)))]
    (when message (println message))
    (when resolved-reports (println (pod/format-errors pod resolved-reports)))
    (when exception (println (:message exception)))))

(defn print-errors
  "Print errors if any"
  [{:keys [error pod raw-directives raw-xf-directives]}]
  (doseq [plugin (:plugins error)]
    (println "ERROR in plugin" (:name plugin)
             "-" (get-in plugin [:err :message])))
  (when-let [parser-error (:parser error)] (print-error parser-error nil pod))
  (when-let [raw-plugin-error (:raw-plugin error)]
    (print-error raw-plugin-error nil pod))
  (when-let [booking-error (:booking error)]
    (println "Booking failed\n")
    (print-error booking-error (or raw-xf-directives raw-directives) pod))
  (when-let [booked-plugin-error (:booked-plugin error)]
    (print-error booked-plugin-error nil pod)))
