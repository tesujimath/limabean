(ns limabean.adapter.loader
  "Load from beanfile and run plugins"
  (:require [limabean.adapter.plugins :as plugins]
            [limabean.adapter.pod :as pod]))

(defn load-beanfile
  [path]
  (let [pod (pod/start path)
        beans (-> (plugins/resolve-external (pod/book pod))
                  (assoc :pod pod))
        booked-directives (:directives beans)
        {:keys [directives err]} (plugins/run-booked-xf booked-directives
                                                        (:plugins beans))]
    (cond-> (assoc beans
              :directives directives
              :booked-directives booked-directives)
      err (assoc :plugin-errors err))))
