(ns limabean.app
  (:require [limabean]
            [limabean.adapter.exception :refer [print-causes]]
            [limabean.adapter.user-clj :as user-clj]
            [rebel-readline.clojure.main :as rebel-clj-main]))

(defn- print-exception
  "Print exception to *err* according to what it is."
  [e]
  (binding [*out* *err*]
    (if (instance? clojure.lang.ExceptionInfo e)
      (if-let [user-error (:user-error (ex-data e))]
        (do (print user-error) (flush))
        (println "unexpected error" e))
      (do (println "Unexpected error" e) (.printStackTrace e)))))

(defn- init
  "Return a function which initializes or exits with error message on failure"
  [{:keys [beanfile]}]
  (fn []
    (try (require '[limabean :refer :all])
         (require '[limabean.core.filters :as f])
         (limabean/load-beanfile beanfile)
         (user-clj/load-user-cljs)
         (catch Exception e (print-exception e) (System/exit 1)))))

(defn- try-eval
  [expr-str options]
  (try (let [expr (read-string expr-str)]
         ((init options))
         (eval expr))
       (catch Exception e
         (binding [*out* *err*]
           (println "Error:" expr-str)
           (print-causes e)))))

(defn run
  "Run the REPL or evaluate an expression and exit"
  [options]
  (binding [*ns* (find-ns 'user)]
    (if-let [expr-str (:eval options)]
      (try-eval expr-str options)
      (rebel-clj-main/repl :init (init options) :caught print-exception))))
