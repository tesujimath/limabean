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

(defn- resolve-idx
  [[dct-idx pst-idx] directives]
  (let [dct (get directives dct-idx)
        pst (get (:postings dct) pst-idx)]
    (cond-> {:kind (if pst "posting" (name (:dct dct))),
             :span (or (:span pst) (:span dct))}
      pst (assoc :context ["txn" (:span dct)]))))

(defn- resolve-related
  [directives]
  (fn [idx]
    (let [{:keys [kind span]} (resolve-idx idx directives)] [kind span])))

(defn- resolve-indexed-report
  [report directives]
  (let [{:keys [kind span context]} (resolve-idx (:idx report) directives)]
    (cond-> (assoc (select-keys report [:reason :annotation])
              :message (str "invalid " kind)
              :span span)
      context (assoc :context context)
      (:related report) (assoc :related
                          (mapv (resolve-related directives)
                            (:related report))))))

(defn- resolve-indexed-reports
  [reports directives]
  (map #(resolve-indexed-report % directives) reports))

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
             (let [data (ex-data e)
                   {:keys [dct plugin]} data]
               (if (and dct plugin)
                 (do
                   (println "Plugin" plugin "failed with" (.getMessage e) "at")
                   (println dct))
                 (do (println "Exception in" kind-name "plugin")
                     (.printStackTrace e)))
               (println "All" kind-name "plugins ignored")))
           m))))

(defn- book-raw-directives
  "Book the raw-xf directives if any, otherwise use the raw directives as parsed."
  [m]
  (binding [*err* *out*]
    (try (let [{:keys [ok err]} (pod/book (:pod m) (:raw-xf-directives m))]
           (if ok
             (let [{:keys [directives warnings]} ok]
               (cond-> (assoc m :booked-directives (type/directives directives))
                 (debug/dump-configured?) (dump :booked-directives)
                 (seq warnings) (assoc :booked-warnings warnings)))
             (let [{:keys [spanned-reports indexed-reports message]} err
                   raw-directives (or (:raw-xf-directives m)
                                      (:raw-directives m))
                   resolved-reports (or spanned-reports
                                        (and indexed-reports
                                             (resolve-indexed-reports
                                               indexed-reports
                                               raw-directives)))
                   message (if resolved-reports
                             (pod/format-errors (:pod m) resolved-reports)
                             message)]
               (println "Booking failed\n" message)
               (assoc-in m [:errors :booking] err))))
         (catch Exception e
           (println "Booking failed")
           (exception/print-exception e)
           m))))

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
