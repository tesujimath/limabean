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
  [m k]
  (let [basename (-> (io/file (:path m))
                     .getName
                     (str/replace #"\.[^.]*$" ""))
        prefix (str/replace (name k) #"-directives$" "")]
    (debug/dump (get m k) (str basename "." prefix ".beancount"))
    m))

(defn- get-raw-directives
  [m]
  (let [{:keys [directives warnings]} (pod/directives (:pod m))]
    (cond-> (assoc m :raw-directives (type/directives directives))
      (seq warnings) (assoc :raw-warnings warnings)
      (debug/dump-configured?) (dump :raw-directives))))

(defn- run-raw-plugins
  [m]
  (let [{:keys [directives errors]}
          (plugins/run-xf (:raw-directives m) (:plugins m) :raw-xf)]
    (cond-> (assoc m :raw-xf-directives (type/directives directives))
      (debug/dump-configured?) (dump :raw-xf-directives)
      (seq errors) (assoc :raw-xf-errors errors))))

(defn- book-raw-directives
  "Book the raw-xf directives if any, otherwise use the raw plugins as parsed."
  [m]
  (let [{:keys [directives warnings]} (pod/book (:pod m)
                                                (:raw-xf-directives m))]
    (cond-> (assoc m :booked-directives (type/directives directives))
      (debug/dump-configured?) (dump :booked-directives)
      (seq warnings) (assoc :booked-warnings warnings))))

(defn- run-booked-plugins
  [m]
  (let [{:keys [directives errors]}
          (plugins/run-xf (:booked-directives m) (:plugins m) :booked-xf)]
    (cond-> (assoc m :booked-xf-directives (type/directives directives))
      (debug/dump-configured?) (dump :booked-xf-directives)
      (seq errors) (assoc :booked-xf-errors errors))))

(defn load-beanfile
  [path]
  (let [pod (pod/start path)
        plugins (pod/plugins pod)
        options {} ;; TODO get options from JSON-RPC method
        resolved-plugins (plugins/resolve-symbols plugins options)]
    (cond-> {:path path, :pod pod, :plugins resolved-plugins, :options options}
      true (get-raw-directives)
      (plugins/has-specified-plugins? resolved-plugins :raw-xf)
        (run-raw-plugins)
      true (book-raw-directives)
      (plugins/has-specified-plugins? resolved-plugins :booked-xf)
        (run-booked-plugins)
      true (as-> m (assoc m
                     :directives (or (:booked-xf-directives m)
                                     (:booked-directives m)))
             (assoc m
               :registry (registry/build (:directives m) (:options m)))))))
