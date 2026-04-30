(ns limabean.adapter.loader
  "Load from beanfile and run plugins"
  (:require [clojure.java.io :as io]
            [clojure.spec.alpha :as s]
            [clojure.string :as str]
            [limabean.adapter.debug :as debug]
            [limabean.adapter.exception :as exception]
            [limabean.adapter.json]
            [limabean.adapter.plugins :as plugins]
            [limabean.adapter.pod :as pod]
            [limabean.adapter.synthetic-spans :as synthetic-spans]
            [limabean.core.registry :as registry]
            [limabean.core.type :as type]
            [limabean.macros :as macros]
            [limabean.adapter.print]
            [limabean.spec :as spec]))

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

(defn- get-plugins-and-options
  [m]
  (let [{:keys [ok err]} (pod/plugins (:pod m))]
    (if ok
      (let [plugins ok
            options (:ok (pod/options (:pod m)))
            resolved-plugins (plugins/resolve-symbols plugins options)
            plugin-resolution-errors (vec (keep :err resolved-plugins))]
        (cond-> (assoc m
                  :plugins resolved-plugins
                  :options options)
          (seq plugin-resolution-errors) (assoc-in [:error :plugins]
                                           plugin-resolution-errors)))
      (let [spanned-reports (:spanned-reports err)]
        (binding [*out* *err*]
          ;; this is where we first encounter parse errors
          (println (pod/format-errors (:pod m) spanned-reports))
          (assoc-in m [:error :parser] spanned-reports))))))

(defn- get-raw-directives
  [m]
  (let [{:keys [ok err]} (pod/directives (:pod m))]
    (if ok
      (let [{:keys [directives warnings]} ok]
        (cond-> (assoc m :raw-directives (type/directives directives))
          (seq warnings) (assoc :raw-warnings warnings)
          (debug/dump-configured?) (dump :raw-directives)))
      (throw (ex-info "unexpected parser failure on directives" err)))))

(defn- kind-name-key
  "Return a map according to kind-name"
  [kind-name]
  (into {}
        (map (fn [k] [k (keyword (str kind-name "-" (name k)))])
          [:xf :directives :xf-directives :xf-errors])))

(defn- run-plugins
  "Run plugins, kind being :raw or :booked"
  [m kind]
  (let [kind-name (name kind)
        key (kind-name-key kind-name)
        create-synthetic-spans-if-required
          (if (= kind :raw)
            #(synthetic-spans/create-and-merge-with-provenance % (:pod m))
            identity)]
    (if-not (plugins/has-specified-plugins? (:plugins m) (:xf key))
      m
      (try (let [{:keys [directives errors]} (plugins/run-plugins-of-kind
                                               (get m (:directives key))
                                               (:plugins m)
                                               (:xf key)
                                               (spec/directive-spec kind))]
             (cond-> (assoc m
                       (:xf-directives key)
                         (type/directives (create-synthetic-spans-if-required
                                            directives)))
               (debug/dump-configured?) (dump (:directives key))
               (seq errors) (assoc (:xf-errors key) errors)))
           (catch Exception e
             (let [error-key (keyword (str kind-name "-plugin"))]
               (assoc-in m
                 [:error error-key :exception]
                 (exception/handle-exception (ex-info (str "Error in "
                                                           kind-name
                                                           " plugin, all "
                                                           kind-name
                                                           " plugins ignored")
                                                      {}
                                                      e)))))))))

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
               (assoc-in m [:error :booking] err))))
         (catch Exception e
           (println "Booking failed")
           (assoc-in m
             [:error :booking]
             {:exception (exception/handle-exception e)})))))

(defn load-beanfile
  [path]
  (let [pod (pod/start path)]
    (s/check-asserts true)
    (macros/bind->
      {:path path, :pod pod}
      (get-plugins-and-options)
      (get-raw-directives)
      (run-plugins :raw)
      (book-raw-directives)
      (run-plugins :booked)
      (as-> m (assoc m
                :directives
                  (or (:booked-xf-directives m) (:booked-directives m) [])))
      (as-> m (assoc m
                :registry (registry/build (:directives m) (:options m)))))))
