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

(defn resolve-symbols
  "Resolve plugin symbols, returning as an updated map"
  [plugins options]
  (mapv (resolve-xfs-with-config options) plugins))

(defn- compose-resolved-xf
  "Compose the transducers in the plugins"
  [resolved-plugins sel]
  (apply comp (keep sel resolved-plugins)))

(defn has-raw?
  "Return whether there are raw plugins to run"
  [resolved-plugins]
  (boolean (seq (keep :raw-xf resolved-plugins))))

(defn- plugin-error
  "Construct a plugin error from ex-info"
  [e sel]
  (let [exd (ex-data e)
        span (get-in exd [:dct :span])
        sel-str (case sel
                  :raw-xf "raw"
                  :booked-xf "booked")]
    (cond-> (merge {:message (if (:plugin exd)
                               (str sel-str " plugin " (:plugin exd) " failed")
                               (str "unknown " sel-str " plugin failed"))}
                   (select-keys exd [:reason :dct]))
      span (assoc :span span))))

(defn run-xf
  "Run the non-error plugins selected by `sel`, one of `:raw-xf,` `:booked-xf`"
  [directives resolved-plugins sel]
  (try {:ok (into [] (compose-resolved-xf resolved-plugins sel) directives)}
       (catch clojure.lang.ExceptionInfo e {:err (plugin-error e sel)})
       (catch Exception e
         {:err {:message (str "Plugin pipeline failed unexpectedly: "
                              (.getMessage e)
                              "\nPlugin diagnostics coming soon")}})))
