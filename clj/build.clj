(ns build
  (:refer-clojure :exclude [test])
  (:require [clojure.tools.build.api :as b]
            [deps-deploy.deps-deploy :as deps-deploy]
            [clojure.java.io :as io]))

(def lib 'io.github.tesujimath/limabean)
(def version "0.2.0-SNAPSHOT")
(def main 'limabean.main)
(def class-dir "target/classes")
(def basis (b/create-basis {:project "deps.edn"}))

(defn- pom-template
  [version]
  [[:description "A new implementation of Beancount using Rust and Clojure."]
   [:url "https://github.com/tesujimath/limabean"]
   [:licenses
    [:license [:name "Apache License, Version 2.0"]
     [:url "https://www.apache.org/licenses/LICENSE-2.0"]]
    [:license [:name "MIT license"]
     [:url "https://opensource.org/licenses/MIT"]]]
   [:developers
    [:developer [:name "Simon Guest"] [:email "simon.guest@tesujimath.org"]
     [:url "https://github.com/tesujimath"]]]
   [:scm [:url "https://github.com/tesujimath/limabean"]
    [:connection "scm:git:git://github.com/tesujimath/limabean.git"]
    [:developerConnection
     "scm:git:ssh://git@github.com/tesujimath/limabean.git"] [:tag version]]])

(defn test
  "Run all the tests."
  [opts]
  (let [cmds (b/java-command {:basis basis,
                              :main 'clojure.main,
                              :main-args ["-m" "cognitect.test-runner"]})
        {:keys [exit]} (b/process cmds)]
    (when-not (zero? exit) (throw (ex-info "Tests failed" {}))))
  opts)

(defn clean [_] (b/delete {:path "target"}))

(defn- uber-opts
  [opts]
  (assoc opts
    :lib lib
    :main main
    :uber-file (format "target/%s-%s.jar" (name lib) version)
    :basis basis
    :class-dir class-dir
    :src-dirs ["src"]
    :ns-compile [main]))

(defn uberjar
  "Build the uberjar."
  [opts]
  (clean nil)
  (let [opts (uber-opts opts)]
    (println "\nCopying source...")
    (b/copy-dir {:src-dirs ["resources" "src"], :target-dir class-dir})
    (println (str "\nCompiling " main "..."))
    (b/compile-clj opts)
    (println "\nBuilding uberjar" (:uber-file opts))
    (b/uber opts)
    opts))

(defn write-pom
  "Write pom.xml from template"
  [opts]
  (b/write-pom (assoc opts
                 :class-dir class-dir
                 :lib lib
                 :version version
                 :basis basis
                 :src-dirs ["src"]
                 :pom-data (pom-template version)))
  (println "wrote" (format "target/classes/META-INF/maven/%s/pom.xml" lib))
  (assoc opts
    :pom-file (format "target/classes/META-INF/maven/%s/pom.xml" lib)))

(defn ci
  "Run the CI pipeline of tests (and build the uberjar)."
  [opts]
  (test opts)
  (clean nil)
  (let [opts (uber-opts opts)]
    (println "\nCopying source...")
    (b/copy-dir {:src-dirs ["resources" "src"], :target-dir class-dir})
    (println (str "\nCompiling " main "..."))
    (b/compile-clj opts)
    (println "\nBuilding uberjar" (:uber-file opts))
    (b/uber opts))
  opts)

(defn deploy
  [opts]
  (let [opts (uberjar opts)
        opts (write-pom opts)]
    (let [artifact (:uber-file opts)
          pom-file (:pom-file opts)]
      (println "deploying pom-file" pom-file "artifact" artifact)
      (deps-deploy/deploy {:installer :remote,
                           :sign-releases true,
                           :artifact artifact,
                           :pom-file pom-file}))
    opts))
