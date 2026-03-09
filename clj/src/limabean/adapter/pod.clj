(ns limabean.adapter.pod
  (:require [cheshire.core :as cheshire]
            [clojure.java.io :as io]
            [clojure.java.process :as process]
            [clojure.string :as str])
  (:import [java.io OutputStreamWriter PrintWriter]
           [java.util.concurrent TimeUnit]))

(defn start
  "Run limabean-pod serve and remain attached"
  [beancount-path]
  (let [pod-process
          (process/start {:err :inherit} "limabean-pod" "serve" beancount-path)]
    {:process pod-process,
     :in (-> (process/stdin pod-process)
             OutputStreamWriter.
             PrintWriter.),
     :out (-> (process/stdout pod-process)
              io/reader),
     :next-id (atom 0)}))

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
                                                :id (str id) ;; TODO string
                                                             ;; until
                                                             ;; limabean-pod
                                                             ;; accepts
                                                             ;; number
                                                :jsonrpc "2.0"))]
    (binding [*out* *err*] (println "write-msg" jsonrpc-msg))
    (write-line pod jsonrpc-msg)))

(defn read-msg
  "Read and decode a response"
  [pod]
  (-> pod
      (read-line)
      (cheshire/parse-string true)))

(defn stop
  "Stop the limabean-pod"
  [pod]
  (.close (:in pod))
  (when-not (.waitFor (:process pod) 10 TimeUnit/SECONDS)
    (binding [*out* *err*]
      (println "WARNING: limabean-pod failed to stop, killing with prejudice"))
    (.destroyForcibly (:process pod)))
  (.close (:out pod)))
