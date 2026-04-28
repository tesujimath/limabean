(ns limabean.test.create-golden
  (:require [clojure.java.io :as io]
            [limabean.adapter.bean-queries :as bean-queries]
            [limabean.adapter.json]
            [limabean.adapter.loader :as loader]
            [limabean.adapter.print]
            [limabean.adapter.show :as show]
            [limabean.test]
            [zprint.core :refer [zprint]]))


(defn- create-output-file
  [beans f output-file]
  (with-open [w (io/writer output-file)] (binding [*out* w] (f beans))))

(def ^{:private true} OUTPUTS
  {:directives {:file "directives.edn",
                :f (fn [beans]
                     (zprint (limabean.test/remove-spans-and-indexes
                               (:directives beans)))),
                :fyi-file "directives.fyi.beancount",
                :fyi-f (fn [beans] (print (:directives beans)))},
   :raw-xf-directives {:file "raw-xf-directives.edn",
                       :f (fn [beans]
                            (zprint (limabean.test/remove-spans-and-indexes
                                      (:raw-xf-directives beans)))),
                       :fyi-file "raw-xf-directives.fyi.beancount",
                       :fyi-f (fn [beans] (print (:raw-xf-directives beans)))},
   :inventory {:file "inventory",
               :f (fn [beans] (show/show (bean-queries/inventory beans [])))},
   :rollup {:file "rollup",
            :f (fn [beans]
                 (show/show (bean-queries/rollup
                              (bean-queries/inventory beans []))))},
   :journal {:file "journal",
             :f (fn [beans] (show/show (bean-queries/journal beans [])))}})

(defn update-existing
  "Update only golden test output files which exist"
  [{:keys [root-dir]}]
  (run! (fn [{:keys [beanfile golden-dir]}]
          (let [beans (loader/load-beanfile beanfile)
                bad-plugins (filter :err (:plugins beans))]
            (if (empty? bad-plugins)
              (run! (fn [[k output]]
                      (let [output-file (io/file golden-dir (:file output))
                            exists (.exists output-file)
                            fyi-file (:fyi-file output)
                            fyi-f (:fyi-f output)]
                        (when exists
                          (println "writing" (name k) "to" output-file)
                          (create-output-file beans (:f output) output-file)
                          (when (and fyi-file fyi-f)
                            (println "writing"
                                     (name k)
                                     "to"
                                     fyi-file
                                     "for information only")
                            (create-output-file beans
                                                fyi-f
                                                (io/file golden-dir
                                                         fyi-file))))))
                    OUTPUTS)
              (println "not creating output files for " beanfile
                       "because bad plugins" bad-plugins))))
        (limabean.test/find-golden-tests root-dir)))
