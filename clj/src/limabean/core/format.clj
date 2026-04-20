(ns limabean.core.format
  (:require [clojure.string :as str]))

(defn- invalid-directive!
  [dct]
  (throw (ex-info "Cannot format invalid directive" {:directive dct})))

(defn- dct-type
  [dct]
  (let [t (:dct dct)]
    (cond (= t :txn) (or (:flag dct) "txn")
          (keyword? t) (name t)
          :else (invalid-directive! dct))))

(defn- double-quote
  "Double quote a string, otherwise nil"
  [s]
  (and (string? s) (format "\"%s\"" s)))

(defn- prefix
  "Prefix a string, otherwise nil"
  [s prefix]
  (and (string? s) (format "%s%s" prefix s)))

(defn- booking->str
  [booking]
  (and (keyword? booking)
       (double-quote (str/replace (str/upper-case (name booking)) "-" "_"))))

(defn- print-tags-and-links-inline
  [tags links]
  (when tags (doseq [tag tags] (print "" (prefix tag "#"))))
  (when links (doseq [link links] (print "" (prefix link "^")))))

(defn- print-tags-and-links-on-separate-lines
  [indent tags links]
  (when tags (doseq [tag tags] (println (str indent (prefix tag "#")))))
  (when links (doseq [link links] (println (str indent (prefix link "^"))))))

(defn- meta-value->str
  [v]
  (cond (nil? v) ""
        (contains? v :acc) (get v :acc)
        (and (contains? v :units) (contains? v :cur))
          (format "%s %s" (str (get v :units)) (get v :cur))
        (contains? v :string) (double-quote (get v :string))
        (contains? v :bool) (str/upper-case (str (get v :bool)))
        (contains? v :cur) (get v :cur)
        (contains? v :number) (str (get v :number))
        (contains? v :tag) (prefix (get v :tag) "#")
        (contains? v :link) (prefix (get v :link) "^")
        (contains? v :date) (str (get v :date))))

(defn- print-meta-key-values-on-separate-lines
  [indent meta-key-values]
  (doseq [k (sort (keys meta-key-values))]
    (let [v (get meta-key-values k)]
      (println (format "%s%s:%s%s"
                       indent
                       (name k)
                       (if v " " "")
                       (meta-value->str v))))))

(defn- cost->str
  "Convert cost or cost-spec to string"
  [cost]
  (let [bare-cost-str
          (with-out-str
            ;; print at most one of per-unit and total, preferring per-unit
            (if-let [per-unit (:per-unit cost)]
              (print (str per-unit))
              (when-let [total (:total cost)] (print "#" (str total))))
            (when-let [cur (:cur cost)] (print "" cur))
            (when-let [date (:date cost)] (print "," (str date)))
            (when-let [label (double-quote (:label cost))] (print "," label))
            (when (:merge cost) (print ", *")))]
    (str "{" (str/replace-first bare-cost-str #"^,? " "") "}")))

(defn- price->str
  "Convert price or price-spec to string"
  [price]
  (with-out-str (print "@")
                ;; print at most one of per-unit and total, preferring
                ;; per-unit
                (if-let [per-unit (:per-unit price)]
                  (print "" (str per-unit))
                  (when-let [total (:total price)] (print " #" (str total))))
                (when-let [cur (:cur price)] (print "" cur))))

(defn- print-posting
  [pst]
  (print " ")
  (when-let [flag (:flag pst)] (print "" flag))
  (print "" (:acc pst))
  (when-let [units (:units pst)] (print "" (str units)))
  (when-let [cur (:cur pst)] (print "" cur))
  (when-let [cost (or (:cost pst) (:cost-spec pst))]
    (print "" (cost->str cost)))
  (when-let [price (or (:price pst) (:price-spec pst))]
    (print "" (price->str price)))
  (println)
  (print-tags-and-links-on-separate-lines "  " (:tags pst) (:links pst))
  (when-let [metadata (:metadata pst)]
    (print-meta-key-values-on-separate-lines "  " metadata)))

(defn- posting->str [pst] (with-out-str (print-posting pst)))

(defn- print-dct-common-header-fields
  [dct]
  (print (str (or (:date dct) (invalid-directive! dct))) (dct-type dct)))

(defn- print-txn-specific-header-fields
  [txn]
  (let [payee (double-quote (:payee txn))
        narration (double-quote (:narration txn))
        empty (double-quote "")]
    (cond (and payee narration) (print "" payee narration)
          payee (print "" payee empty)
          narration (print "" narration))))

(defn- txn-header->str
  [txn]
  (with-out-str (print-dct-common-header-fields txn)
                (print-txn-specific-header-fields txn)))

(defn directive->str
  "Convert directive to string"
  [dct]
  (with-out-str
    (print-dct-common-header-fields dct)
    (case (:dct dct)
      :txn (print-txn-specific-header-fields dct)
      :price (let [price (:price dct)]
               (print "" (:cur dct) (str (:per-unit price)) (:cur price)))
      :balance (do (print "" (:acc dct) (str (:units dct)) (:cur dct))
                   (when-let [tol (:tolerance dct)] (print " ~" (str tol))))
      :open (do (print "" (:acc dct))
                (when-let [currencies (:currencies dct)]
                  (print "" (str/join "," (sort currencies))))
                (when-let [booking (booking->str (:booking dct))]
                  (print "" booking)))
      :close (print "" (:acc dct))
      :commodity (print "" (:cur dct))
      :pad (print "" (:acc dct) (:source dct))
      :document (print "" (:acc dct) (double-quote (:path dct)))
      :note (print "" (:acc dct) (double-quote (:comment dct)))
      :event
        (print "" (double-quote (:type dct)) (double-quote (:description dct)))
      :query (print "" (double-quote (:name dct)) (double-quote (:content dct)))
      :custom (do (print "" (double-quote (:type dct)))
                  (doseq [v (:values dct)] (print "" (meta-value->str v))))
      nil)
    (when-not (= (:dct dct) :custom)
      (print-tags-and-links-inline (:tags dct) (:links dct)))
    (println)
    (when (= (:dct dct) :custom)
      ;; print tags/links for custom on separate lines to avoid confusion
      ;; with meta values
      (print-tags-and-links-on-separate-lines "  " (:tags dct) (:links dct)))
    (when-let [metadata (:metadata dct)]
      (print-meta-key-values-on-separate-lines "  " metadata))
    (when (= (:dct dct) :txn)
      (doseq [pst (:postings dct)] (print-posting pst)))))

(defn elements-xf
  "Transducer to produce elements from directives, to facilitate building synthetic spans.

  Except for transactions, each directive results in a single string, its human-readable representation.

  Transactions result in strings for the header and each posting separately.

  `element-builder` is a function taking the directive and string, and returning the element
  "
  [element-builder]
  (fn [rf]
    (fn
      ;; init
      ([] (rf))
      ;; completion
      ([result] (rf result))
      ;; step
      ([result dct]
       (if (= (:dct dct) :txn)
         ;; transaction header line and postings separately
         (let [result' (rf result (element-builder dct (txn-header->str dct)))]
           (reduce rf
             result'
             (map #(element-builder dct (posting->str %)) (:postings dct))))
         ;; otherwise emit the whole formatted directive
         (rf result (element-builder dct (directive->str dct))))))))
