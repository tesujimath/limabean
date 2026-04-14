(ns limabean.adapter.loader
  "Load from beanfile and run plugins"
  (:require [limabean.adapter.plugins :as plugins]
            [limabean.adapter.pod :as pod]
            [limabean.core.registry :as registry]
            [clojure.java.io :as io]
            [clojure.string :as str]
            [limabean.adapter.debug :as debug]
            [limabean.core.type :as type]
            [limabean.adapter.exception :as exception]
            [limabean.adapter.synthetic-spans :as synthetic-spans]))

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

(defn- run-plugins
  "Run plugins, kind being :raw or :booked"
  [m kind]
  (let [kind-name (name kind)
        key (into {}
                  (map (fn [k] [k (keyword (str kind-name "-" (name k)))])
                    [:xf :directives :xf-directives :xf-errors]))
        create-synthetic-spans-if-required
          (if (= kind :raw) #(synthetic-spans/create % (:pod m)) identity)]
    (try (let [{:keys [directives errors]} (plugins/run-plugins-of-kind
                                             (get m (:directives key))
                                             (:plugins m)
                                             (:xf key))]
           (cond-> (assoc m
                     (:xf-directives key) (type/directives
                                            (create-synthetic-spans-if-required
                                              directives)))
             (debug/dump-configured?) (dump (:directives key))
             (seq errors) (assoc (:xf-errors key) errors)))
         (catch Exception e
           (binding [*err* *out*]
             (println "Exception in"
                      kind-name
                      "plugin, all"
                      kind-name
                      "plugins ignored")
             (.printStackTrace e))
           m))))

(defn- book-raw-directives
  "Book the raw-xf directives if any, otherwise use the raw plugins as parsed."
  [m]
  (try (let [{:keys [directives warnings]} (pod/book (:pod m)
                                                     (:raw-xf-directives m))]
         (cond-> (assoc m :booked-directives (type/directives directives))
           (debug/dump-configured?) (dump :booked-directives)
           (seq warnings) (assoc :booked-warnings warnings)))
       (catch Exception e
         (binding [*err* *out*] (println "Booking failed"))
         (exception/print-exception e)
         m)))

(defn load-beanfile
  [path]
  (let [pod (pod/start path)
        plugins (pod/plugins pod)
        options (pod/options pod)
        resolved-plugins (plugins/resolve-symbols plugins options)]
    (cond-> {:path path, :pod pod, :plugins resolved-plugins, :options options}
      true (get-raw-directives)
      (plugins/has-specified-plugins? resolved-plugins :raw-xf) (run-plugins
                                                                  :raw)
      true (book-raw-directives)
      (plugins/has-specified-plugins? resolved-plugins :booked-xf) (run-plugins
                                                                     :booked)
      true (as-> m (assoc m
                     :directives (or (:booked-xf-directives m)
                                     (:booked-directives m)))
             (assoc m
               :registry (registry/build (:directives m) (:options m)))))))
