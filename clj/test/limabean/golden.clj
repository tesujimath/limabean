(ns limabean.golden
  (:require [clojure.java.io :as io]
            [limabean.adapter.edn] ;; for print-method java.time.LocalDate
            [limabean.adapter.loader :as loader]
            [limabean.test-support :as test-support]
            [zprint.core :refer [zprint]]))

(defn- create-directives-file
  [beanfile directives-file]
  (let [beans (loader/load-beanfile beanfile)
        bad-plugins (filter :err (:plugins beans))
        directives (:directives beans)]
    (if (empty? bad-plugins)
      (do (println "writing directives to" directives-file)
          (with-open [w (io/writer directives-file)]
            (binding [*out* w] (zprint directives))))
      (println "not writing directives to" directives-file
               "because bad plugins" bad-plugins))))

(defn create-directives-files
  [{:keys [force refresh]}]
  (println "gen-golden" force refresh)
  (run! (fn [{:keys [beanfile golden-dir]}]
          (let [directives-file (io/file golden-dir "directives.edn")
                exists (.exists directives-file)]
            (when (or force (and refresh exists))
              (create-directives-file beanfile directives-file))))
        (test-support/get-tests)))
