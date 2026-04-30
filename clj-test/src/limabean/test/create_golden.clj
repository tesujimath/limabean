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
  {:directives {:filename "directives.edn",
                :f (fn [beans]
                     (zprint (limabean.test/remove-spans-and-indexes
                               (:directives beans)))),
                :fyi-filename "directives.fyi.beancount",
                :fyi-f (fn [beans] (print (:directives beans))),
                :required-f (fn [beans exists]
                              (or (contains? beans :booked-xf-directives)
                                  exists))},
   :raw-xf-directives {:filename "raw-xf-directives.edn",
                       :f (fn [beans]
                            (zprint (limabean.test/remove-spans-and-indexes
                                      (:raw-xf-directives beans)))),
                       :fyi-filename "raw-xf-directives.fyi.beancount",
                       :fyi-f (fn [beans] (print (:raw-xf-directives beans))),
                       :required-f (fn [beans exists]
                                     (or (contains? beans :raw-xf-directives)
                                         exists))},
   :inventory {:filename "inventory",
               :f (fn [beans] (show/show (bean-queries/inventory beans []))),
               :required-f (fn [_beans exists] exists)},
   :rollup {:filename "rollup",
            :f (fn [beans]
                 (show/show (bean-queries/rollup (bean-queries/inventory beans
                                                                         [])))),
            :required-f (fn [_beans exists] exists)},
   :journal {:filename "journal",
             :f (fn [beans] (show/show (bean-queries/journal beans []))),
             :required-f (fn [_beans exists] exists)}})

(defn- mkdir
  [dir]
  (.toFile (java.nio.file.Files/createDirectories
             (java.nio.file.Paths/get dir (make-array String 0))
             nil)))

(defn update-all
  "Update any golden test output files which exist, or any required by plugins"
  [{:keys [root-dir]}]
  (run!
    (fn [{:keys [beanfile golden-dir]}]
      (let [beans (loader/load-beanfile beanfile)
            bad-plugins (filter :err (:plugins beans))
            golden-dir (if (.exists golden-dir) golden-dir (mkdir golden-dir))]
        (if (empty? bad-plugins)
          (run! (fn [[k output]]
                  (let [output-file (io/file golden-dir (:filename output))
                        exists (.exists output-file)
                        fyi-filename (:fyi-filename output)
                        fyi-f (:fyi-f output)
                        required-f (:required-f output)]
                    (when (required-f beans exists)
                      (println "writing" (name k) "to" (.getPath output-file))
                      (create-output-file beans (:f output) output-file)
                      (when (and fyi-filename fyi-f)
                        (let [fyi-file (io/file golden-dir fyi-filename)]
                          (println "writing"
                                   (name k)
                                   "to"
                                   (.getPath fyi-file)
                                   "for information only")
                          (create-output-file beans fyi-f fyi-file))))))
                OUTPUTS)
          (println "not creating output files for " beanfile
                   "because bad plugins" bad-plugins))))
    (limabean.test/find-golden-tests root-dir :ignore-golden-dirs true)))
