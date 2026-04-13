(ns limabean.core.format
  (:require [clojure.string :as str]))

(defn- dct-type
  [dct]
  (let [type (:dct dct)] (if (= type :txn) (or (:flag dct) "txn") (name type))))

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
  [posting]
  (print " ")
  (when-let [flag (:flag posting)] (print "" flag))
  (print "" (:acc posting))
  (when-let [units (:units posting)] (print "" (str units)))
  (when-let [cur (:cur posting)] (print "" cur))
  (when-let [cost (or (:cost posting) (:cost-spec posting))]
    (print "" (cost->str cost)))
  (when-let [price (or (:price posting) (:price-spec posting))]
    (print "" (price->str price)))
  (println)
  (print-tags-and-links-on-separate-lines "  " (:tags posting) (:links posting))
  (when-let [metadata (:metadata posting)]
    (print-meta-key-values-on-separate-lines "  " metadata)))

(defn- print-txn-header
  [txn]
  (let [payee (double-quote (:payee txn))
        narration (double-quote (:narration txn))
        empty (double-quote "")]
    (cond (and payee narration) (print "" payee narration)
          payee (print "" payee empty)
          narration (print "" narration))))

(defn directive->str
  "Convert directive to string"
  [dct]
  (with-out-str
    (print (str (:date dct)) (dct-type dct))
    (case (:dct dct)
      :txn (print-txn-header dct)
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

(defn directive->str-or-strs
  "Return a str or multiple strings for a directive, according to whether it is a transaction.

  Transactions are returned as a header line then all postings, to facilitate building synthetic spans.
  "
  [dct]
  (if (= (:dct dct) :txn)
    (reduce (fn [result pst] (conj result (with-out-str (print-posting pst))))
      [(with-out-str (print-txn-header dct) (println))]
      (:postings dct))
    (directive->str dct)))
