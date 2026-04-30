(ns limabean.adapter.edn
  (:refer-clojure :exclude [read-string])
  (:require [clojure.edn :as edn]
            [java-time.api :as jt]
            [limabean.adapter.print]))

(def readers {'time/date #(jt/local-date %), 'regex re-pattern})

(defn read-string
  "Read string as limabean EDN"
  [s]
  (edn/read-string {:readers readers} s))
