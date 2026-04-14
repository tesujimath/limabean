(ns limabean.adapter.exception)

(defn print-causes
  "Prints the message of e and all its causes"
  [^Throwable e]
  (loop [ex e] (when ex (println (.getMessage ex)) (recur (.getCause ex)))))

(defn print-exception
  "Print exception to *err* according to what it is."
  [e]
  (binding [*out* *err*]
    (if (instance? clojure.lang.ExceptionInfo e)
      (if (contains? (ex-data e) :user-error)
        (when-let [user-error (:user-error (ex-data e))]
          (print user-error)
          (flush))
        (println "unexpected error" e))
      (do (println "Unexpected error" e) (.printStackTrace e)))))
