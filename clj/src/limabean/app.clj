(ns limabean.app
  (:require [limabean.adapter.beanfile :as beanfile]
            [limabean.adapter.show :refer [show]]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inventory]
            [limabean.core.journal :as journal]
            [limabean.core.registry :as registry]
            [limabean.core.xf :as xf]
            [limabean.user]
            [rebel-readline.clojure.main :as rebel-clj-main]
            [taoensso.telemere :as tel]))

(defn make-filters
  "Make filters from CLI options and beanfile options"
  [cli options]
  (cond-> []
    (contains? cli :cur) (conj (f/cur (:cur cli)))
    (contains? cli :begin) (conj (f/date>= (:begin cli)))
    (contains? cli :end) (conj (f/date< (:end cli)))
    (contains? cli :balance) (conj (f/sub-acc (:name-assets options)
                                              (:name-liabilities options)))
    (contains? cli :income) (conj (f/sub-acc (:name-income options)
                                             (:name-expenses options)))))

(defn inventory
  "Print inventory, filtered as per cli options"
  [cli]
  (let [{:keys [directives options]} (beanfile/book (:beanfile cli))
        filters (make-filters cli options)
        registry (registry/build directives options)
        _ (tel/log! {:id ::registry, :data registry})
        postings (eduction (comp (xf/postings)
                                 (filter (apply f/every-f filters)))
                           directives)
        inv (inventory/build postings (partial registry/acc-booking registry))
        _ (tel/log! {:id ::inventory, :data inv})]
    (show inv)))

(defn journal
  "Print journal, filtered as per cli options"
  [cli]
  (let [{:keys [directives options]} (beanfile/book (:beanfile cli))
        filters (make-filters cli options)
        registry (registry/build directives options)
        _ (tel/log! {:id ::registry, :data registry})
        postings (eduction (comp (xf/postings)
                                 (filter (apply f/every-f filters)))
                           directives)
        journal (journal/build postings)
        _ (tel/log! {:id ::journal, :data journal})]
    (show journal)))

(defn print-exception
  "Print exception to *err* according to what it is."
  [e]
  (binding [*out* *err*]
    (if (instance? clojure.lang.ExceptionInfo e)
      (if-let [user-error (:user-error (ex-data e))]
        (do (print user-error) (flush))
        (println "unexpected error" e))
      (do (println "Unexpected error" e) (.printStackTrace e)))))

(defn repl
  "Run the REPL"
  [{:keys [beanfile]}]
  (rebel-clj-main/repl
    :init (fn []
            (try (require '[limabean.user :refer :all])
                 (require '[limabean.core.filters :as f])
                 (limabean.user/load-beanfile beanfile)
                 (catch Exception e (print-exception e) (System/exit 1))))
    :caught print-exception))
