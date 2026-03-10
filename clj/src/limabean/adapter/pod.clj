(ns limabean.adapter.pod
  (:require [cheshire.core :as cheshire]
            [cheshire.parse]
            [clojure.java.io :as io]
            [clojure.java.process :as process]
            [clojure.walk :as walk]
            [java-time.api :as jt])
  (:import [java.io OutputStreamWriter PrintWriter]
           [java.util.concurrent TimeUnit]))

(defn- next-id! [pod] (swap! (:next-id pod) inc))

(defn write-line
  "Write a single line message to the pod"
  [pod msg]
  (when (some #(= % \newline) msg)
    (throw (ex-info "attempt to write message with newline to pod" {:msg msg})))
  (binding [*out* (:in pod)] (println msg)))

(defn read-line
  "Read a single line message from the pod"
  [pod]
  (.readLine (:out pod)))

(defn write-msg
  "Write a message map with fields [:method :params] to the pod, adding `id` and `jsonrpc` fields"
  [pod msg]
  (let [id (next-id! pod)
        jsonrpc-msg (cheshire/generate-string (assoc msg
                                                :id id
                                                :jsonrpc "2.0"))]
    (write-line pod jsonrpc-msg)))

(defn parse-dates
  [data]
  (walk/postwalk (fn [x]
                   (if (and (map? x) (contains? x :date))
                     (update x :date #(jt/local-date %))
                     x))
                 data))

(defn read-msg
  "Read and decode a response, using BigDecimal for all numbers, and converting :date values in maps into jt/local-date"
  [pod]
  (binding [cheshire.parse/*use-bigdecimals?* true]
    (-> pod
        (read-line)
        (cheshire/parse-string true)
        (parse-dates))))

(defn invoke
  "Invoke a remote procedure call, with the method and params"
  ([pod method] (invoke pod method nil))
  ([pod method params]
   (let [msg (cond-> {:method method} params (assoc :params params))]
     (write-msg pod msg)
     (let [response (read-msg pod)]
       (cond-> {}
         (:result response) (assoc :ok (:result response))
         (:error response) (assoc :err (:error response)))))))

(defn stop
  "Stop the limabean-pod"
  [pod]
  (.close (:in pod))
  (when-not (.waitFor (:process pod) 10 TimeUnit/SECONDS)
    (binding [*out* *err*]
      (println "WARNING: limabean-pod failed to stop, killing with prejudice"))
    (.destroyForcibly (:process pod)))
  (.close (:out pod)))

(defn start
  "Run limabean-pod serve and remain attached"
  [beancount-path]
  (let [pod-process
          (process/start {:err :inherit} "limabean-pod" "serve" beancount-path)
        pod {:process pod-process,
             :in (-> (process/stdin pod-process)
                     OutputStreamWriter.
                     PrintWriter.),
             :out (-> (process/stdout pod-process)
                      io/reader),
             :next-id (atom 0)}
        status (invoke pod "status")]
    (when (:err status)
      (stop pod)
      (throw (ex-info "pod/start failed"
                      {:user-error (get-in status [:err :message])})))
    pod))
