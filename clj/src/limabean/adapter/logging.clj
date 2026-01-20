(ns limabean.adapter.logging
  (:require [cheshire.core :as cheshire]
            [cheshire.generate :as cheshire-generate]
            [taoensso.telemere :as tel])
  (:import [com.fasterxml.jackson.databind ObjectMapper]
           [com.fasterxml.jackson.datatype.jsr310 JavaTimeModule]
           [java.time Instant ZonedDateTime ZoneId]
           [java.time.format DateTimeFormatter]))

;; encode Instant as localtime, i.e. ISO_OFFSET_DATE_TIME
(cheshire-generate/add-encoder
  java.time.Instant
  (fn [^Instant inst ^com.fasterxml.jackson.core.JsonGenerator jg]
    (let [zdt (ZonedDateTime/ofInstant inst (ZoneId/systemDefault))]
      (.writeString jg (.format zdt DateTimeFormatter/ISO_OFFSET_DATE_TIME)))))

;; ensure cheshire/jackson can encode Java LocalDate
(cheshire-generate/add-encoder
  java.time.LocalDate
  (fn [^java.time.LocalDate d ^com.fasterxml.jackson.core.JsonGenerator jg]
    (.writeString jg (.toString d))))

(defn json-file-handler
  [path]
  (tel/handler:file {:path path,
                     :output-fn (tel/pr-signal-fn
                                  {:pr-fn cheshire/generate-string})}))

(defn xf
  "Logging transducer"
  [{:keys [id level data]}]
  (let [level (or level :info)
        data (or data {})]
    (map (fn [x] (tel/log! {:id id, :level level, :data (merge data x)}) x))))

(defn wrap
  "Wrap a transducer in a logging decorator"
  [f opts]
  (comp f (xf opts)))

(defn initialize
  "Initialize logging, only if environment variable LIMABEAN_LOG is defined."
  []
  (tel/remove-handler! :default/console)
  (if-let [logpath (System/getenv "LIMABEAN_LOG")]
    (do (tel/add-handler! :json-file (json-file-handler logpath))
        (tel/call-on-shutdown! (fn [] (tel/stop-handlers!))))))
