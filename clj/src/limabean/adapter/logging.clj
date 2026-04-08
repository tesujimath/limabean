(ns limabean.adapter.logging
  (:require [cheshire.core :as cheshire]
            [limabean.adapter.json]
            [taoensso.telemere :as tel]))

(defn json-file-handler
  [path]
  (tel/handler:file {:path path,
                     :output-fn (tel/pr-signal-fn
                                  {:pr-fn cheshire/generate-string})}))

(defn xf
  "Logging transducer"
  [{:keys [id level data]}]
  (let [level (or level :info)
        data (or data {})]
    (map (fn [x] (tel/log! {:id id, :level level, :data (merge data x)}) x))))

(defn wrap
  "Wrap a transducer in a logging decorator"
  [f opts]
  (comp f (xf opts)))

(defn initialize
  "Initialize logging, only if environment variable LIMABEAN_LOG is defined."
  []
  (tel/remove-handler! :default/console)
  (when-let [logpath (System/getenv "LIMABEAN_LOG")]
    (tel/add-handler! :json-file (json-file-handler logpath))
    (tel/call-on-shutdown! (fn [] (tel/stop-handlers!)))))
