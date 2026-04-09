(ns limabean.adapter.print)

;; make printing LocalDate use the same form
(defmethod print-method java.time.LocalDate
  [v w]
  (.write w (str "#time/date \"" v "\"")))
