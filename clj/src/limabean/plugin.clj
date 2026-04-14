(ns limabean.plugin)

(defmacro error!
  [dct message]
  (let [ns-name (str *ns*)]
    `(throw (ex-info ~message {:dct ~dct, :plugin ~ns-name}))))
