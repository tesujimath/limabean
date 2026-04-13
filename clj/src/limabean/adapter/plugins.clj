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

(defn tag-unknown
  "Transducer to tag unknown directives"
  [known-directives tagf]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result d]
       (if (not (contains? @known-directives (System/identityHashCode d)))
         (let [tagged-d (tagf d)]
           (vreset! known-directives
                    (conj! @known-directives
                           (System/identityHashCode tagged-d)))
           (rf result tagged-d))
         ;; otherwise emit the original directive, whatever it was
         (rf result d))))))

(defn provenance-tagf
  [provenance]
  (fn [d] (update d :provenance (fnil conj []) provenance)))

(defn compose-resolved-xf
  "Compose the transducers in the plugins along with a tagging transducer to set the provenance"
  [resolved-plugins sel known-directives]
  (apply comp
    (tag-unknown known-directives identity)
    (keep (fn [plugin]
            (when-let [xf (get plugin sel)]
              (comp xf
                    (tag-unknown known-directives
                                 (provenance-tagf (:name plugin))))))
          resolved-plugins)))

(defn has-specified-plugins?
  "Return whether there are plugins of the given kind to run"
  [resolved-plugins sel]
  (boolean (seq (keep sel resolved-plugins))))

(defn run-xf
  "Run the non-error plugins selected by `sel`, one of `:raw-xf,` `:booked-xf`"
  [directives resolved-plugins sel]
  (let [known-directives (volatile! (transient #{}))]
    ;; TODO actually separate out directives and errors with plugin
    ;; transducers wrapper
    {:directives (into
                   []
                   (compose-resolved-xf resolved-plugins sel known-directives)
                   directives),
     :errors []}))
