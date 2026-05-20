(ns limabean.adapter.error
  "Error formatting and printing.

  Various types of error may occur, and they have different attributes.

  Parse errors are returned from limabean-pod as spanned reports, which are internally cross-referenced by span, and
  require formatting using pod/format-errors.

  Plugin errors are returned as :err annotations on directives.  The :err annotations are guaranteed to have :span
  or :span-p fields.  The latter reference synthetic spans for any directives which have been inserted or modified by
  plugins.  Modified directives have both :span and :span-p fields.  There are no cross-references.

  Booking errors are returned from limabean-pod as indexed reports.  Tne index is by directive or
  posting-within-transaction into the list of raw xf directives (as produced by running raw plugins), or raw directives
  (as returned by the parser) in the case of there being no raw plugins.  These may be internally cross-referenced by
  index to related items.  To handle the cross references requires mapping indexes to spans, and then using
  pod/format-errors.  Note that the spans may be synthetic, but these are guaranteed to have been created."
  (:require [clojure.string :as str]
            [limabean.adapter.pod :as pod]))

(defn- dct-errors->reports
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
  [error pod]
  (let [{:keys [spanned-reports dct-errors message exception]} error
        spanned-reports' (or spanned-reports (dct-errors->reports dct-errors))]
    (when message (println message))
    (when spanned-reports' (println (pod/format-errors pod spanned-reports')))
    (when exception (println (:message exception)))))

(defn print-errors
  "Print errors if any."
  [{:keys [error pod]}]
  (doseq [plugin (:plugins error)]
    (println "ERROR in plugin" (:name plugin)
             "-" (get-in plugin [:err :message])))
  (when-let [parser-error (:parser error)] (print-error parser-error pod))
  (when-let [raw-plugins-error (:raw-plugins error)]
    (print-error raw-plugins-error pod))
  (when-let [booking-error (:booking error)]
    (println "Booking failed\n")
    (print-error booking-error pod))
  (when-let [booked-plugins-error (:booked-plugins error)]
    (print-error booked-plugins-error pod)))
