(ns limabean.adapter.plugins
  (:require [clojure.edn :as edn]))

(defn- resolve-xfs
  "Resolve a plugin by loading it from its namespace"
  [name]
  (let [ns-sym (symbol name)]
    (try (require ns-sym)
         (let [booked-xf-fn (ns-resolve ns-sym 'booked-xf)
               raw-xf-fn (ns-resolve ns-sym 'raw-xf)
               xfs (into {}
                         (keep (fn [[k v]] (when v [k v])))
                         [[:booked-xf booked-xf-fn] [:raw-xf raw-xf-fn]])]
           (if (seq xfs)
             xfs
             {:err "Failed to find either booked-xf or raw-xf in plugin"}))
         (catch Exception e {:err "could not load namespace for plugin"}))))

(defn- resolve-xfs-with-config'
  "Resolve a plugin and apply config and options"
  [{:keys [name config]} options]
  (let [xfs (resolve-xfs name)]
    (if (:err xfs)
      xfs
      (try (let [config-val (edn/read-string config)]
             (into
               {}
               (map (fn [[k f]] [k (f {:config config-val, :options options})])
                 xfs)))
           (catch Exception e
             {:err (str "Error resolving config: " (.getMessage e))})))))

(defn- resolve-xfs-with-config
  "Return a function which merges the plugin definition with its resolution"
  [options]
  (fn [plugin]
    (let [resolved (try (resolve-xfs-with-config' plugin options)
                        (catch Exception e {:err (.getMessage e)}))]
      (merge plugin resolved))))

(defn resolve-external
  "Resolve external plugins, returning as an updated map"
  [beans]
  (update-in beans
             [:plugins :external]
             #(mapv (resolve-xfs-with-config (:options beans)) %)))

(defn- compose-resolved-external-booked-xf
  "Compose the transducers in the external plugins"
  [resolved-plugins]
  (apply comp (keep :booked-xf (:external resolved-plugins))))

(defn run-booked-xf
  "Run the non-error external plugins"
  [directives resolved-plugins]
  (try {:directives (into []
                          (compose-resolved-external-booked-xf resolved-plugins)
                          directives)}
       (catch Exception e
         {:directives directives,
          :err (str "ERROR running plugin pipeline, all plugins ignored: "
                    (.getMessage e)
                    "\nPlugin diagnostics coming soon")})))
