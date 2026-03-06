(ns limabean.adapter.loader
  "Load from beanfile and run plugins"
  (:require [limabean.adapter.beanfile :as beanfile]
            [limabean.adapter.plugins :as plugins]))

(defn load-beanfile
  [path]
  (let [beans (plugins/resolve-external (beanfile/book path))]
    (let [booked-directives (:directives beans)
          {:keys [directives err]} (plugins/run-booked-xf booked-directives
                                                          (:plugins beans))]
      (cond-> (assoc beans
                :directives directives
                :booked-directives booked-directives)
        err (assoc :plugin-errors err)))))
