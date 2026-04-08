(ns limabean.adapter.json
  (:require [cheshire.generate :as cheshire-generate])
  (:import [java.time Instant ZonedDateTime ZoneId]
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
