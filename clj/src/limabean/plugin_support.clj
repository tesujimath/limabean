(ns limabean.plugin-support)

(defmacro plugin-failed-on-directive!
  "Fail a plugin with the specified reason on the given directive"
  [reason dct]
  (let [plugin (str *ns*)]
    `(throw (ex-info (str "Plugin " ~plugin " error")
                     (merge {:plugin ~plugin, :reason ~reason, :dct ~dct})))))
