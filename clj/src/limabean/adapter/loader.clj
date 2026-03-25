(ns limabean.adapter.loader
  "Load from beanfile and run plugins"
  (:require [limabean.adapter.plugins :as plugins]
            [limabean.adapter.pod :as pod]))

(defn load-beanfile
  [path]
  (let [pod (pod/start path)
        plugins (pod/plugins pod)
        options {} ;; TODO get options from JSON-RPC method
        resolved-plugins (plugins/resolve-symbols plugins options)
        raw-directives (pod/directives pod)
        {pre-booked-directives :directives, raw-err :err}
          (plugins/run-xf raw-directives resolved-plugins :raw-xf)
        booked-directives (pod/book pod pre-booked-directives)
        {directives :directives, booked-err :err}
          (plugins/run-xf booked-directives resolved-plugins :booked-xf)]
    (cond-> {:pod pod,
             :plugins resolved-plugins,
             :options options,
             :raw-directives raw-directives,
             :pre-booked-directives pre-booked-directives,
             :booked-directives booked-directives,
             :directives directives}
      raw-err (assoc :raw-plugin-errors raw-err)
      booked-err (assoc :booked-plugin-errors booked-err))))
