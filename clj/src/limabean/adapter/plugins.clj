(ns limabean.adapter.plugins
  (:require [clojure.spec.alpha :as s]
            [clojure.string :as str]
            [limabean.adapter.edn :as limabean-edn]))

(defn- resolve-xfs
  "Resolve a plugin by loading it from its namespace"
  [name]
  (let [ns-sym (symbol (str/replace name #"_" "-"))]
    (try (require ns-sym)
         (let [booked-xf-fn (ns-resolve ns-sym 'booked-xf)
               raw-xf-fn (ns-resolve ns-sym 'raw-xf)
               xfs (into {}
                         (keep (fn [[k v]] (when v [k v])))
                         [[:booked-xf booked-xf-fn] [:raw-xf raw-xf-fn]])]
           (if (seq xfs)
             xfs
             {:err {:message
                      "Failed to find either booked-xf or raw-xf in plugin"}}))
         (catch Exception _ {:err "could not load namespace for plugin"}))))

(defn- resolve-xfs-with-config'
  "Resolve a plugin and apply config and options"
  [{:keys [name config]} options]
  (let [xfs (resolve-xfs name)]
    (if (:err xfs)
      xfs
      (try (let [config-val (limabean-edn/read-string config)]
             (into
               {}
               (map (fn [[k f]] [k (f {:config config-val, :options options})])
                 xfs)))
           (catch Exception e
             {:err {:message (str "Error resolving config: " (.getMessage e)),
                    :exception (Throwable->map e)}})))))

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

(defn- tag-and-validate-unseen
  "Transducer to tag which haven't been seen before, and if spec is non-nil, validate"
  [known-directives tagf spec]
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
           (rf result (if spec (s/assert spec tagged-d) tagged-d)))
         ;; otherwise emit the original directive, whatever it was
         (rf result d))))))

(defn- provenance-tagf
  [provenance]
  (fn [d] (update d :provenance (fnil conj []) provenance)))

(defn- compose-and-wrap-resolved-plugins
  "Compose the transducers in the plugins along with a tagging transducer to set the provenance"
  [resolved-plugins sel directive-spec known-directives]
  (apply comp
    (tag-and-validate-unseen known-directives identity directive-spec)
    (keep (fn [plugin]
            (when-let [xf (get plugin sel)]
              (comp xf
                    (tag-and-validate-unseen known-directives
                                             (provenance-tagf (:name plugin))
                                             directive-spec))))
          resolved-plugins)))

(defn has-specified-plugins?
  "Return whether there are plugins of the given kind to run"
  [resolved-plugins sel]
  (boolean (seq (keep sel resolved-plugins))))

(defn run-plugins-of-kind
  "Run the non-error plugins selected by `sel`, one of `:raw-xf,` `:booked-xf`"
  [directives resolved-plugins sel directive-spec]
  (let [known-directives (volatile! (transient #{}))]
    ;; TODO actually separate out directives and errors with plugin
    ;; transducers wrapper
    {:directives (into []
                       (compose-and-wrap-resolved-plugins resolved-plugins
                                                          sel
                                                          directive-spec
                                                          known-directives)
                       directives),
     :errors []}))
