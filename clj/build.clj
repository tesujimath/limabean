(ns build
  (:refer-clojure :exclude [test])
  (:require [clojure.tools.build.api :as b]
            [clojure.java.io :as io]
            [clojure.java.shell :as sh]
            [cheshire.core :as cheshire]
            [deps-deploy.deps-deploy :as deps-deploy]))

(defn cargo-version
  "Read the version from lima"
  []
  (-> (sh/sh "cargo"
             "metadata" "--no-deps"
             "--format-version" "1"
             "--manifest-path" "../rust/limabean/Cargo.toml")
      :out
      (cheshire/parse-string true)
      :packages
      first
      :version))

(def lib 'io.github.tesujimath/limabean)
(def version (cargo-version))
(def main 'limabean.main)
(def class-dir "target/classes")
(def jar-file (format "target/%s-%s.jar" (name lib) version))

;; TODO remove extra once rebel readline available on Clojars
;; but for now we need it because transitive dependencies aren't
;; loaded via a git co-ordinate
(def basis
  (b/create-basis
    {:project "deps.edn",
     :extra {:deps {'compliment/compliment {:mvn/version "0.6.0"},
                    'dev.weavejester/cljfmt {:mvn/version "0.13.0"},
                    'org.jline/jline-reader {:mvn/version "3.30.0"},
                    'org.jline/jline-terminal {:mvn/version "3.30.0"},
                    'org.jline/jline-terminal-jni {:mvn/version "3.30.0"}}}}))

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
                 :pom-data (pom-template version)))
  (println "wrote" (format "target/classes/META-INF/maven/%s/pom.xml" lib))
  (assoc opts
    :pom-file (format "target/classes/META-INF/maven/%s/pom.xml" lib)))

(defn- jar-opts
  [opts]
  (assoc opts
    :class-dir class-dir
    :jar-file jar-file
    :basis basis
    :manifest {"Implementation-Version" version}))

(defn jar
  [opts]
  (let [opts (clean opts)
        opts (write-pom opts)
        opts (jar-opts opts)]
    (println "\nCopying source...")
    (b/copy-dir {:src-dirs ["resources" "src"], :target-dir class-dir})
    (println (str "\nCompiling " main "..."))
    (b/compile-clj opts)
    (println "\nBuilding jar" (:jar-file opts))
    (b/jar opts)
    opts))

(defn ci
  "Run the CI pipeline of tests (and build the jar)."
  [opts]
  (test opts)
  (clean nil)
  (let [opts (jar-opts opts)]
    (println "\nCopying source...")
    (b/copy-dir {:src-dirs ["resources" "src"], :target-dir class-dir})
    (println (str "\nCompiling " main "..."))
    (b/compile-clj opts)
    (println "\nBuilding jar" (:jar-file opts))
    (b/jar opts))
  opts)

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
