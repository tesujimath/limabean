(ns limabean.util)

(defn- pop-first!
  "Pop the first element from the front of volatile `xs`"
  [xs]
  (let [[x & remaining] @xs]
    (vreset! xs remaining)
    x))

(defn map-if
  "Map over collections `c1` and `c2`, where elements are combined using f if (pred e1) returns true, else simply an element from c1"
  [pred f c1 c2]
  (let [c2 (volatile! c2)]
    (map (fn [e1] (if (pred e1) (let [e2 (pop-first! c2)] (f e1 e2)) e1)) c1)))

(defn- pop-first-n!
  "Pop the first `n` elements from the front of volatile `xs`"
  [n xs]
  (let [[taken remaining] (split-at n @xs)]
    (vreset! xs remaining)
    taken))

(defn map-n
  "Map over collections `c1` and `c2`, where elements are combined using f which takes (num-f e1) elements from c2, else simply an element from c1"
  [num-f f c1 c2]
  (let [c2 (volatile! c2)]
    (map (fn [e1]
           (let [n (num-f e1)] (if (zero? n) e1 (f e1 (pop-first-n! n c2)))))
      c1)))
