(ns limabean.adapter.print
  (:require [limabean.core.format :as format]))

;; make printing LocalDate use the same form
(defmethod print-method java.time.LocalDate
  [v w]
  (.write w (str "#time/date \"" v "\"")))

(defmethod print-method :limabean/dct
  [dct writer]
  (doto writer (.write (format/directive->str dct))))

(defmethod print-method :limabean/directives
  [directives writer]
  (run! #(.write writer %)
        (interpose "\n" (map format/directive->str directives))))
