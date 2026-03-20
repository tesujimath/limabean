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

(defn- write-line
  "Write a single line message to the pod"
  [pod msg]
  (when (some #(= % \newline) msg)
    (throw (ex-info "attempt to write message with newline to pod" {:msg msg})))
  (binding [*out* (:in pod)] (println msg)))

(defn- read-to-eol
  "Read a single line message from the pod"
  [pod]
  (.readLine (:out pod)))

(defn- write-msg
  "Write a message map with fields [:method :params] to the pod, adding `id` and `jsonrpc` fields"
  [pod msg]
  (let [id (next-id! pod)
        jsonrpc-msg (cheshire/generate-string (assoc msg
                                                :id id
                                                :jsonrpc "2.0"))]
    (write-line pod jsonrpc-msg)))

(defn- convert-values
  [data]
  (walk/postwalk
    (fn [x]
      (cond-> x
        (and (map? x) (contains? x :date)) (update :date jt/local-date)
        (and (map? x) (contains? x :dct)) (update :dct keyword)
        (and (map? x) (contains? x :booking)) (update :booking keyword)))
    data))

(defn- coerce-arrays
  "Map JSON array to Clojure set or vector according to map key"
  [k]
  (if (contains? #{"currencies" "tags" "links"} k) #{} []))

(defn- read-msg
  "Read and decode a response, using BigDecimal for all numbers, and converting values as appropriate"
  [pod]
  (binding [cheshire.parse/*use-bigdecimals?* true]
    (-> pod
        (read-to-eol)
        (cheshire/parse-string true coerce-arrays)
        (convert-values))))

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

(def ERROR-REPORT 2)

(declare format-errors)

(defn ok-or-print-errors-and-throw
  "Either unwrap an ok result, or extract the errors, print them, and throw"
  [pod result]
  (if-let [err (:err result)]
    (binding [*out* *err*]
      (when (= ERROR-REPORT (:code err))
        (println (format-errors pod (:data err)))
        (throw (ex-info (:message err) {:user-error nil})))
      (throw (ex-info (:message err) {:user-error (:message err)})))
    (:ok result)))

(defn ok-or-throw
  "Either unwrap an ok result, or throw"
  [result]
  (if-let [err (:err result)]
    (throw (ex-info (:message err) {:user-error (:message err)}))
    (:ok result)))

;; methods
(defn status "Return pod status" [pod] (invoke pod "status"))
(defn plugins
  "Return parsed plugins"
  [pod]
  (ok-or-print-errors-and-throw pod (invoke pod "parser.plugins")))
(defn directives
  "Return parsed directives"
  [pod]
  (ok-or-print-errors-and-throw pod (invoke pod "parser.directives")))
(defn format-errors
  "Format errors"
  [pod errors]
  (ok-or-throw (invoke pod "parser.format-errors" errors)))
(defn format-warnings
  "Format warnings"
  [pod warnings]
  (ok-or-throw (invoke pod "parser.format-warnings" warnings)))
(defn resolve-span
  "Resolve a span in terms of original sources"
  [pod span]
  (ok-or-throw (invoke pod "parser.resolve-span" span)))
(defn book
  "Book directives, or parsed directives by default"
  ([pod] (ok-or-print-errors-and-throw pod (invoke pod "book")))
  ([pod directives]
   (ok-or-print-errors-and-throw pod (invoke pod "book" directives))))

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
