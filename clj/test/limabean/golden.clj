(ns limabean.golden
  (:require [clojure.java.io :as io]
            [limabean.adapter.bean-queries :as bean-queries]
            [limabean.adapter.json]
            [limabean.adapter.loader :as loader]
            [limabean.adapter.print]
            [limabean.adapter.show :as show]
            [limabean.test-support :as test-support]
            [zprint.core :refer [zprint]]))

(defn create-output-file
  [beans f output-file]
  (with-open [w (io/writer output-file)] (binding [*out* w] (f beans))))

(def OUTPUTS
  {:directives {:file "directives.edn",
                :f (fn [beans]
                     (zprint (test-support/remove-spans-and-indexes
                               (:directives beans))))},
   :inventory {:file "inventory",
               :f (fn [beans] (show/show (bean-queries/inventory beans [])))},
   :rollup {:file "rollup",
            :f (fn [beans]
                 (show/show (bean-queries/rollup
                              (bean-queries/inventory beans []))))},
   :journal {:file "journal",
             :f (fn [beans] (show/show (bean-queries/journal beans [])))}})

(defn generate
  [{:keys [force refresh]}]
  (println "gen-golden" force refresh)
  (run! (fn [{:keys [beanfile golden-dir]}]
          (let [beans (loader/load-beanfile beanfile)
                bad-plugins (filter :err (:plugins beans))]
            (if (empty? bad-plugins)
              (run! (fn [[k output]]
                      (let [output-file (io/file golden-dir
                                                 (str (:file output)))
                            exists (.exists output-file)]
                        (when (or force (and refresh exists))
                          (println "writing" (name k) "to" output-file)
                          (create-output-file beans (:f output) output-file))))
                    OUTPUTS)
              (println "not creating output files for " beanfile
                       "because bad plugins" bad-plugins))))
        (test-support/get-tests)))
