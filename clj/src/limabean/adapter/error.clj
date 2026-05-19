(ns limabean.adapter.error
  "Error formatting and printing.

  Various types of error may occur, and they have different attributes.

  Parse errors are returned from limabean-pod as spanned reports, which are internally cross-referenced by span, and
  require formatting using pod/format-errors.

  Raw plugin errors are returned as :err annotations on directives.  The :err annotations are guaranteed to have :span
  or :span-p fields.  The latter reference synthetic spans for any directives which have been inserted or modified by
  plugins.  Modified directives have both :span and :span-p fields.  There are no cross-references.

  Booking errors are returned from limabean-pod as indexed reports.  Tne index is by directive or
  posting-within-transaction into the list of raw xf directives (as produced by running raw plugins), or raw directives
  (as returned by the parser) in the case of there being no raw plugins.  These may be internally cross-referenced by
  index to related items.  To handle the cross references requires mapping indexes to spans, and then using
  pod/format-errors.  Note that the spans may be synthetic, but these are guaranteed to have been created."
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

(defn- spanned-dct-errors->reports
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
  [error directives pod]
  (let [{:keys [spanned-reports spanned-dct-errors indexed-reports message
                exception]}
          error
        spanned-reports' (or spanned-reports
                             (spanned-dct-errors->reports spanned-dct-errors))
        resolved-reports (or (seq spanned-reports')
                             (and indexed-reports
                                  (resolve-indexed-reports indexed-reports
                                                           directives)))]
    (when message (println message))
    (when resolved-reports (println (pod/format-errors pod resolved-reports)))
    (when spanned-dct-errors nil)
    (when exception (println (:message exception)))))

(defn print-errors
  "Print errors if any"
  [{:keys [error pod raw-directives raw-xf-directives]}]
  (doseq [plugin (:plugins error)]
    (println "ERROR in plugin" (:name plugin)
             "-" (get-in plugin [:err :message])))
  (when-let [parser-error (:parser error)] (print-error parser-error nil pod))
  (when-let [raw-plugins-error (:raw-plugins error)]
    (print-error raw-plugins-error nil pod))
  (when-let [booking-error (:booking error)]
    (println "Booking failed\n")
    (print-error booking-error (or raw-xf-directives raw-directives) pod))
  (when-let [booked-plugins-error (:booked-plugins error)]
    (print-error booked-plugins-error nil pod)))
