(ns limabean.adapter.error
  (:require [clojure.string :as str]
            [limabean.adapter.pod :as pod]))

(defn- resolve-idx
  [[dct-idx pst-idx] directives]
  (let [dct (get directives dct-idx)
        dct-span (or (:span dct) (:span-p dct))
        pst (get (:postings dct) pst-idx)
        pst-span (or (:span pst) (:span-p pst))]
    (cond-> {:description (if pst "posting" (name (:dct dct))),
             :span (or pst-span dct-span)}
      (and (:provenance dct) (or (:span pst) (:span dct)))
        (update :description
                #(str % ", modified by " (str/join " " (:provenance dct))))
      pst (assoc :context ["txn" dct-span]))))

(defn- resolve-related
  [directives]
  (fn [idx]
    (let [{:keys [description span]} (resolve-idx idx directives)]
      [description span])))

(defn- resolve-indexed-report
  [report directives]
  (let [{:keys [description span context]} (resolve-idx (:idx report)
                                                        directives)]
    (cond-> (assoc (select-keys report [:reason :annotation])
              :message (str "invalid " description)
              :span span)
      context (assoc :context context)
      (:related report) (assoc :related
                          (mapv (resolve-related directives)
                            (:related report))))))

(defn- resolve-indexed-reports
  [reports directives]
  (map #(resolve-indexed-report % directives) reports))

(defn dct-errors->reports
  [dct-errors]
  (vec (mapcat (fn [dct]
                 (map (fn [err]
                        (cond-> {:message (:plugin err),
                                 :reason (:message err),
                                 :span (or (:span-p dct) (:span dct))}
                          (and (:span dct) (:span-p :dct))
                            (assoc :related [["source" (:span dct)]])))
                   (:err dct)))
         dct-errors)))

(defn- print-error
  [err directives pod]
  (let [{:keys [spanned-reports raw-xf-directives indexed-reports message
                exception]}
          err
        spanned-reports' (or spanned-reports
                             (dct-errors->reports raw-xf-directives))
        resolved-reports (or (seq spanned-reports')
                             (and indexed-reports
                                  (resolve-indexed-reports indexed-reports
                                                           directives)))]
    (when message (println message))
    (when resolved-reports (println (pod/format-errors pod resolved-reports)))
    (when raw-xf-directives nil)
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
