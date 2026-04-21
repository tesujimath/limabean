(ns limabean.spec
  (:require [clojure.spec.alpha :as s]
            [java-time.api :as jt]))

;; common fields
(s/def ::date jt/local-date?)
(s/def ::acc string?)
(s/def ::units
  (s/or :decimal decimal?
        :int int?))
(s/def ::cur string?)

;; metadata
(s/def ::tag string?)
(s/def ::link string?)
(s/def ::tags (s/coll-of ::tag :kind set?))
(s/def ::links (s/coll-of ::link :kind set?))

(s/def ::metavalue
  (s/or :acc (s/map-of #{:acc} ::acc :count 1)
        :bool (s/map-of #{:bool} boolean? :count 1)
        :cur (s/map-of #{:cur} ::cur :count 1)
        :date (s/map-of #{:date} ::date :count 1)
        :link (s/map-of #{:link} ::link :count 1)
        :null nil?
        :number (s/map-of #{:number} decimal? :count 1)
        :string (s/map-of #{:string} string? :count 1)
        :tag (s/map-of #{:tag} ::tag :count 1)
        :units (s/map-of #{:units} ::units :count 1)))
(s/def ::metadata (s/map-of keyword? ::metavalue))

;; txn fields
(s/def ::flag string?)
(s/def ::payee string?)
(s/def ::narration string?)

;; balance fields
(s/def ::tolerance decimal?)

;; open fields
(s/def ::currencies (s/coll-of ::cur :kind set?))
(s/def ::booking #{:strict :strict-with-size :none :average :fifo :lifo :hifo})

;; pad fields
(s/def ::source string?)

;; document fields
(s/def ::path string?)

;; note fields
(s/def ::comment string?)

;; event fields
(s/def ::type string?)
(s/def ::description string?)

;; query fields
(s/def ::name string?)
(s/def ::content string?)

;; custom fields
(s/def ::values (s/coll-of ::metavalue :kind vector?))

;; cost/cost-spec/price/price-spec fields
(s/def ::per-unit ::units)
(s/def ::total ::units)
(s/def ::label string?)
(s/def ::merge boolean?)

(s/def ::cost-spec
  (s/keys :opt-un [::per-unit ::total ::cur ::date ::label ::merge]))
(s/def ::cost
  (s/keys :req-un [::per-unit ::total ::cur ::date] :opt-un [::label ::merge]))
(s/def ::price-spec (s/keys :opt-un [::per-unit ::total ::cur]))
(s/def ::price (s/keys :req-un [::per-unit ::cur] :opt-un [::total]))

;; posting/posting-spec
;; since both appear as :postings, we differentiate using namespaces raw and
;; booked and similarly for txn
(s/def :limabean.spec.raw/posting
  (s/keys :opt-un [::flag ::acc ::units ::cur ::cost-spec ::price-spec ::tags
                   ::links ::metadata]))
(s/def :limabean.spec.raw/postings
  (s/coll-of :limabean.spec.raw/posting :kind vector?))

(s/def :limabean.spec.booked/posting
  (s/keys :req-un [::acc ::units ::cur]
          :opt-un [::flag ::cost ::price ::tags ::links ::metadata]))
(s/def :limabean.spec.booked/postings
  (s/coll-of :limabean.spec.booked/posting :kind vector?))


;; directives
(s/def ::base-directive
  (s/keys :req-un [::date] :opt-un [::tags ::links ::metadata]))

(s/def ::raw-txn
  (s/keys :req-un [::flag]
          :opt-un [::payee ::narration :limabean.spec.raw/postings]))
(s/def ::booked-txn
  (s/keys :req-un [::flag]
          :opt-un [::payee ::narration :limabean.spec.booked/postings]))

;; the awkward name is because ::price is the basic value not the directive:
(s/def :limabean.spec.dct/price (s/keys :req-un [::cur ::price]))
(s/def ::balance (s/keys :req-un [::acc ::units ::cur] :opt-un [::tolerance]))
(s/def ::open (s/keys :req-un [::acc] :opt-un [::currencies ::booking]))
(s/def ::close (s/keys :req-un [::acc]))
(s/def ::commodity (s/keys :req-un [::cur]))
(s/def ::pad (s/keys :req-un [::acc ::source]))
(s/def ::document (s/keys :req-un [::acc ::path]))
(s/def ::note (s/keys :req-un [::acc ::comment]))
(s/def ::event (s/keys :req-un [::type ::description]))
(s/def ::query (s/keys :req-un [::name ::content]))
(s/def ::custom (s/keys :req-un [::type ::values]))

(defmulti raw-dct :dct)
(defmethod raw-dct :txn [_] ::raw-txn)
(defmethod raw-dct :price [_] :limabean.spec.dct/price)
(defmethod raw-dct :balance [_] ::balance)
(defmethod raw-dct :open [_] ::open)
(defmethod raw-dct :close [_] ::close)
(defmethod raw-dct :commodity [_] ::commodity)
(defmethod raw-dct :pad [_] ::pad)
(defmethod raw-dct :document [_] ::document)
(defmethod raw-dct :note [_] ::note)
(defmethod raw-dct :event [_] ::event)
(defmethod raw-dct :query [_] ::query)
(defmethod raw-dct :custom [_] ::custom)
(defmethod raw-dct nil [_] (s/and map? #(contains? % :dct)))
(s/def ::raw-dct (s/and ::base-directive (s/multi-spec raw-dct :dct)))

(defmulti booked-dct :dct)
(defmethod booked-dct :txn [_] ::booked-txn)
(defmethod booked-dct :price [_] :limabean.spec.dct/price)
(defmethod booked-dct :balance [_] ::balance)
(defmethod booked-dct :open [_] ::open)
(defmethod booked-dct :close [_] ::close)
(defmethod booked-dct :commodity [_] ::commodity)
(defmethod booked-dct :pad [_] ::pad)
(defmethod booked-dct :document [_] ::document)
(defmethod booked-dct :note [_] ::note)
(defmethod booked-dct :event [_] ::event)
(defmethod booked-dct :query [_] ::query)
(defmethod booked-dct :custom [_] ::custom)
(defmethod booked-dct nil [_] (s/and map? #(contains? % :dct)))
(s/def ::booked-dct (s/and ::base-directive (s/multi-spec booked-dct :dct)))

(s/def ::raw-directives (s/coll-of ::raw-directive))
(s/def ::booked-directives (s/coll-of :booked-directive))

(defn directive-spec
  "Directive spec for `kind` of directive"
  [kind]
  (case kind
    :raw ::raw-dct
    :booked ::booked-dct))

(defn directives-spec
  "Directive spec for `kind` of directives"
  [kind]
  (case kind
    :raw ::raw-directives
    :booked ::booked-directives))
