(ns limabean.adapter.loader
  "Load from beanfile and run plugins"
  (:require [limabean.adapter.plugins :as plugins]
            [limabean.adapter.pod :as pod]
            [limabean.core.registry :as registry]
            [clojure.java.io :as io]
            [clojure.string :as str]
            [limabean.adapter.debug :as debug]
            [limabean.core.type :as type]))

(defn- dump
  "Dump the beans and return the map"
  [m k filename]
  (debug/dump (get m k) filename)
  m)

(defn load-beanfile
  [path]
  (let [pod (pod/start path)
        plugins (pod/plugins pod)
        options {} ;; TODO get options from JSON-RPC method
        resolved-plugins (plugins/resolve-symbols plugins options)
        basename (-> (io/file path)
                     .getName
                     (str/replace #"\.[^.]*$" ""))
        {:keys [directives warnings]} (pod/directives pod)]
    (cond-> {:pod pod,
             :plugins resolved-plugins,
             :options options,
             :raw-directives (type/directives directives),
             :basename basename}
      (debug/dump-configured?) (dump :raw-directives
                                     (str basename ".raw.beancount"))
      (seq warnings) (assoc :raw-warnings warnings)
      (plugins/has-specified-plugins? resolved-plugins :raw-xf)
        (as-> m
          (let [{:keys [directives errors]}
                  (plugins/run-xf (:raw-directives m) resolved-plugins :raw-xf)]
            (cond-> (assoc m :raw-xf-directives (type/directives directives))
              (debug/dump-configured?) (dump :raw-xf-directives
                                             (str basename ".raw-xf.beancount"))
              (seq errors) (assoc :raw-xf-errors errors))))
      true (as-> m (let [{:keys [directives warnings]}
                           (pod/book pod (:raw-xf-directives m))]
                     (cond-> (assoc m
                               :booked-directives (type/directives directives))
                       (debug/dump-configured?) (dump :booked-directives
                                                      (str basename
                                                           ".booked.beancount"))
                       (seq warnings) (assoc :booked-warnings warnings))))
      (plugins/has-specified-plugins? resolved-plugins :booked-xf)
        (as-> m (let [{:keys [directives errors]} (plugins/run-xf
                                                    (:booked-directives m)
                                                    resolved-plugins
                                                    :booked-xf)]
                  (cond-> (assoc m
                            :booked-xf-directives (type/directives directives))
                    (debug/dump-configured?) (dump :booked-xf-directives
                                                   (str basename
                                                        ".booked-xf.beancount"))
                    (seq errors) (assoc :booked-xf-errors errors))))
      true (as-> m (assoc m
                     :directives (or (:booked-xf-directives m)
                                     (:booked-directives m)))
             (assoc m :registry (registry/build (:directives m) (:options m)))
             (select-keys m
                          [:pod :plugins :options :raw-directives
                           :raw-xf-directives :booked-directives
                           :booked-xf-directives :directives])))))
