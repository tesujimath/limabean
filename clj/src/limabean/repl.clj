(ns limabean.repl
  (:require [limabean.adapter.beanfile :as beanfile]
            [limabean.adapter.tabulate :as tabulate]
            [limabean.core.filters :as f]
            [limabean.core.inventory :as inv]
            [limabean.core.registry :as registry]
            [limabean.core.xf :as xf]))

(def ^:dynamic *directives* nil)
(def ^:dynamic *options* nil)
(def ^:dynamic *registry* nil)

(defn load-beanfile
  [path]
  (let [beans (beanfile/book path)]
    (alter-var-root #'*directives* (constantly (:directives beans)))
    (alter-var-root #'*options* (constantly (:options beans)))
    (alter-var-root #'*registry*
                    (constantly (registry/build *directives* *options*)))
    (println (count *directives*) "directives loaded"))
  :ok)

(defn print-inventory [inv] (println (tabulate/inventory inv)) :ok)

(defn print-balances
  []
  (let [postings (eduction (comp (xf/postings)
                                 (filter (f/some-sub-acc
                                           (:name-assets *options*)
                                           (:name-liabilities *options*))))
                           *directives*)
        inv (inv/build postings (:acc-booking *registry*))]
    (println (tabulate/inventory inv)))
  :ok)

(defn print-register
  "Print a register of postings with running balance"
  [& filters]
  (let [reg (eduction (comp (xf/postings) (xf/all-of filters) (xf/register))
                      *directives*)]
    (println (tabulate/register (into [] reg))))
  :ok)
