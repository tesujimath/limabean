(ns limabean.core.type)

(defn- type-as
  "Return a function which applies metadata if required to ensure `x` is typed as `kind`"
  [kind]
  (fn [x] (if (= (:type (meta x)) kind) x (with-meta x {:type kind}))))

(defn directives
  "Apply metadata to all directives"
  [directives]
  ((type-as :limabean/directives)
    (into [] (map (type-as :limabean/dct) directives))))
