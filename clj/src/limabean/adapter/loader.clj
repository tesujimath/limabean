(ns limabean.adapter.loader
  "Load from beanfile and run plugins"
  (:require [limabean.adapter.plugins :as plugins]
            [limabean.adapter.pod :as pod]
            [limabean.core.registry :as registry]))

(defn load-beanfile
  [path]
  (let [pod (pod/start path)
        plugins (pod/plugins pod)
        options {} ;; TODO get options from JSON-RPC method
        resolved-plugins (plugins/resolve-symbols plugins options)
        {:keys [directives warnings]} (pod/directives pod)]
    (cond-> {:pod pod,
             :plugins resolved-plugins,
             :options options,
             :raw-directives directives}
      (seq warnings) (assoc :raw-warnings warnings)
      (plugins/has-specified-plugins? resolved-plugins :raw-xf)
        (as-> m (let [{:keys [directives errors]} (plugins/run-xf
                                                    (:raw-directives m)
                                                    resolved-plugins
                                                    :raw-xf)]
                  (cond-> (assoc m :raw-xf-directives directives)
                    (seq errors) (assoc :raw-xf-errors errors))))
      true (as-> m (let [directives-to-book (or (:raw-xf-directives m)
                                                (:raw-directives m))
                         {:keys [directives warnings]}
                           (pod/book pod directives-to-book)]
                     (cond-> (assoc m :booked-directives directives)
                       (seq warnings) (assoc :booked-warnings warnings))))
      (plugins/has-specified-plugins? resolved-plugins :booked-xf)
        (as-> m (let [{:keys [directives errors]} (plugins/run-xf
                                                    (:booked-directives m)
                                                    resolved-plugins
                                                    :booked-xf)]
                  (cond-> (assoc m :booked-xf-directives directives)
                    (seq errors) (assoc :booked-xf-errors errors))))
      true (as-> m (assoc m
                     :directives (or (:booked-xf-directives m)
                                     (:booked-directives m)))
             (assoc m
               :registry (registry/build (:directives m) (:options m)))))))
