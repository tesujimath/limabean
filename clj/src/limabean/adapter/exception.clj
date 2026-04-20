(ns limabean.adapter.exception
  (:require [clojure.pprint :refer [pprint]]))

(def ^:dynamic *exception* "Last exception, if any" nil)

(defn handle-exception
  "Print exception to *err* and preserve it in *exception*."
  [^Throwable exc]
  (let [e (Throwable->map exc)]
    (alter-var-root #'*exception* (constantly e))
    (binding [*out* *err*]
      (print "Exception: ")
      (cond (:via e) (run! println (keep :message (:via e)))
            (:message e) (println (:message e))
            :else (pprint e))
      (flush))
    e))
