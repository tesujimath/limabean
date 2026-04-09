(ns limabean.adapter.debug
  (:require [clojure.java.io :as io]
            [limabean.core.format :as format]))

(def ^{:private true} DEBUG-DIR (System/getenv "LIMABEAN_DEBUG_DIR"))

(defn dump-configured? [] (not (nil? DEBUG-DIR)))

(defn dump
  "Dump directives in human-readable form into LIMABEAN_DEBUG_DIR if defined"
  [directives filename]
  (when DEBUG-DIR
    (let [debug-file (io/file DEBUG-DIR filename)]
      (try
        (with-open [w (io/writer debug-file)]
          (binding [*out* w]
            (run! print
                  (interpose "\n" (map format/directive->str directives)))))
        (catch Exception e
          (binding [*out* *err*]
            (println
              "WARNING: $LIMABEAN_DEBUG_DIR is defined but failed to write directives to"
              (.getPath debug-file)
              (.getMessage e))))))))
