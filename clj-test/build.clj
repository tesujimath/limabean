(ns build
  (:refer-clojure :exclude [test])
  (:require [clojure.tools.build.api :as b]
            [clojure.java.shell :as sh]
            [cheshire.core :as cheshire]
            [deps-deploy.deps-deploy :as deps-deploy]))

(defn cargo-version
  "Read the version from lima"
  []
  (-> (sh/sh "cargo"
             "metadata" "--no-deps"
             "--format-version" "1"
             "--manifest-path" "../rust/Cargo.toml")
      :out
      (cheshire/parse-string true)
      :packages
      first
      :version))

(def lib 'io.github.tesujimath/limabean-test)
(def version (cargo-version))
(def class-dir "target/classes")
(def jar-file (format "target/%s-%s.jar" (name lib) version))

(def basis (b/create-basis {:project "deps.edn"}))

(defn- pom-template
  [version]
  [[:description "Test utility library for limabean and plugins."]
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

(defn clean [opts] (b/delete {:path "target"}) opts)

(defn write-pom
  "Write pom.xml from template"
  [opts]
  (b/write-pom (assoc opts
                 :class-dir class-dir
                 :lib lib
                 :version version
                 :basis basis
                 :src-dirs ["src"]
                 :src-pom :none
                 :pom-data (pom-template version)))
  (let [generated-pom-file (format "target/classes/META-INF/maven/%s/pom.xml"
                                   lib)
        committed-pom-file "pom.xml"]
    (println "wrote" generated-pom-file)
    (b/copy-file {:src generated-pom-file, :target committed-pom-file})
    (println "copied" generated-pom-file "to" committed-pom-file)
    (assoc opts :pom-file generated-pom-file)))

(defn- jar-opts
  [opts]
  (assoc opts
    :class-dir class-dir
    :jar-file jar-file
    :manifest {"Implementation-Version" version}))

(defn jar
  [opts]
  (let [opts (clean opts)
        opts (write-pom opts)
        opts (jar-opts opts)]
    (println "\nCopying source...")
    (b/copy-dir {:src-dirs ["resources" "src"], :target-dir class-dir})
    (println "\nBuilding jar" (:jar-file opts))
    (b/jar opts)
    opts))

(defn deploy
  [opts]
  (let [opts (jar opts)]
    (let [artifact (:jar-file opts)
          pom-file (:pom-file opts)]
      (println "deploying pom-file" pom-file "artifact" artifact)
      (deps-deploy/deploy {:installer :remote,
                           :sign-releases true,
                           :artifact artifact,
                           :pom-file pom-file}))
    opts))
