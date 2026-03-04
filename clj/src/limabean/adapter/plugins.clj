(ns limabean.adapter.plugins
  (:require [clojure.edn :as edn]))

(defn- resolve-xf
  "Resolve a plugin by loading it from its namespace"
  [name]
  (let [ns-sym (symbol name)]
    (try (let [xf-fn (ns-resolve ns-sym 'booked-directive-xf)]
           (if xf-fn
             {:xf-fn xf-fn}
             {:err "Failed to find booked-directive-xf in plugin"}))
         (catch Exception e {:err (.getMessage e)}))))

(defn- resolve-xf-with-config'
  "Resolve a plugin and apply its config"
  [{:keys [name config]}]
  (let [xf-status (resolve-xf name)]
    (if-let [xf-fn (:xf-fn xf-status)]
      (if config
        (try
          (let [config-val (edn/read-string config)] {:xf (xf-fn config-val)})
          (catch Exception e {:err (str "Error in config: " (.getMessage e))}))
        {:xf (xf-fn)})
      xf-status)))

(defn- resolve-xf-with-config
  "Merge the plugin definition with its resolution"
  [plugin]
  (let [resolved (try (resolve-xf-with-config' plugin)
                      (catch Exception e {:err (.getMessage e)}))]
    (merge plugin resolved)))

(defn resolve-external-plugins
  "Resolve external plugins, returning as an updated map"
  [plugins]
  (update plugins :external #(mapv resolve-xf-with-config %)))

(defn compose-resolved-external-plugins
  "Compose the transducers in the external plugins"
  [resolved-plugins]
  (apply comp (keep :xf (:external resolved-plugins))))
