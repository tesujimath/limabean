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
         (catch Exception _ {:err "could not load namespace for plugin"}))))

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

(defn has-specified-plugins?
  "Return whether there are plugins of the given kind to run"
  [resolved-plugins sel]
  (boolean (seq (keep sel resolved-plugins))))

(defn run-xf
  "Run the non-error plugins selected by `sel`, one of `:raw-xf,` `:booked-xf`"
  [directives resolved-plugins sel]
  ;; TODO actually separate out directives and errors with plugin
  ;; transducers wrapper
  {:directives (into [] (compose-resolved-xf resolved-plugins sel) directives),
   :errors []})
