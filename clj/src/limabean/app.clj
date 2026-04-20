(ns limabean.app
  (:require [limabean]
            [limabean.adapter.exception :as exception]
            [limabean.adapter.user-clj :as user-clj]
            [rebel-readline.clojure.main :as rebel-clj-main]))

(defn- init
  "Return a function which initializes, printing error messages on failure"
  [{:keys [beanfile]}]
  (fn []
    (require '[limabean :refer :all])
    (require '[limabean.core.filters :as f])
    (require '[limabean.core.type :as type])
    (require '[limabean.adapter.exception :refer [*exception*]])
    (require '[clojure.pprint :refer [pprint]])
    (limabean/load-beanfile beanfile)
    (user-clj/load-user-cljs)))

(defn- try-eval
  [expr-str options]
  (try (let [expr (read-string expr-str)]
         ((init options))
         (eval expr))
       (catch Exception e
         (binding [*out* *err*]
           (exception/handle-exception e)
           (System/exit 1)))))

(defn run
  "Run the REPL or evaluate an expression and exit"
  [options]
  (binding [*ns* (find-ns 'user)]
    (if-let [expr-str (:eval options)]
      (try-eval expr-str options)
      (rebel-clj-main/repl :init (init options)
                           :caught exception/handle-exception))))
