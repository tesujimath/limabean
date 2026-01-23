(ns limabean.adapter.shell
  (:require [clojure.java.shell :as shell]))

(defn try-sh
  "Wrap java.shell/sh and handle exception"
  [& args]
  (try (let [{:keys [exit out err]} (apply shell/sh args)]
         (if (= exit 0)
           out
           (throw (ex-info (str (first args) "failed") {:user-error err}))))
       (catch java.io.IOException e
         (throw (ex-info (str (first args) "exception")
                         {:user-error (format "%s\n" (.getMessage e))})))))
