(ns limabean.main
  (:require [clojure.tools.cli :refer [parse-opts]]
            [clojure.java.io :as io]
            [clojure.string :as str]
            [limabean.app :as app]
            [limabean.adapter.logging :as logging]
            [taoensso.telemere :as tel])
  (:gen-class))



(def cli-options
  [[nil "--beanfile PATH" "path to Beancount file" :default-fn
    (fn [opts] (System/getenv "LIMABEAN_BEANFILE"))]])

(def actions #{"balances"})

(defn usage
  [options-summary]
  (->> ["limabean: usage: limabean [options] action" "" "Options:" options-summary ""
        "Actions:" "  report [report-name]    Run a canned report" ""]
       (str/join \newline)))

(defn error-msg
  [errors]
  (str "limabean: argument parsing errors:\n"
       (str/join \newline errors)))

(defn validate-args
  "Validate command line arguments. Either return a map indicating the program
  should exit (with an error message, and optional ok status), or a map
  indicating the action the program should take and the options provided."
  [args]
  (let [{:keys [options arguments errors summary]} (parse-opts args
                                                               cli-options)]
    (tel/log! {:id ::options, :data options})
    (cond (:help options) ; help => exit OK with usage summary
            {:exit-message (usage summary), :ok? true}
          errors ; errors => exit with description of errors
            {:exit-message (error-msg errors)}
          ;; custom validation on arguments
          (not (:beanfile options))
            {:exit-message "limabean: --beanfile or $LIMABEAN_BEANFILE is required"}
          (let [beanfile (io/file (:beanfile options))]
            (not (and (.exists beanfile) (.isFile beanfile))))
            {:exit-message (str "limabean: no such beanfile " (:beanfile options))}
          (empty? arguments) {:action "repl", :options options}
          (and (= 1 (count arguments)) (get actions (first arguments)))
            {:action (first arguments), :options options}
          :else ; failed custom validation => exit with usage summary
            {:exit-message (usage summary)})))

(defn exit [status msg] (println msg) (System/exit status))

(defn -main
  [& args]
  (logging/initialize)
  (tel/log! {:id ::main, :data {:args args}})
  (let [{:keys [action options exit-message ok?]} (validate-args args)]
    (if exit-message
      (exit (if ok? 0 1) exit-message)
      (case action
        "repl" (app/repl options)
        "balances" (app/balances options))))
  (flush)
  (System/exit 0) ;; TODO why is this required, hangs otherwise
)
