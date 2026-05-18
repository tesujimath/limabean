(ns limabean.plugin)

(defmacro ^{:deprecated "0.6", :superseded-by "dct-error!"} error!
  "Deprecated macro to throw an exception when a plugin detects a bad directive."
  [dct message]
  (let [ns-name (str *ns*)]
    `(throw (ex-info ~message {:dct ~dct, :plugin ~ns-name}))))

(defmacro dct-error!
  "Annotate a directive with the given error message, appending to any other errors."
  [dct message]
  (let [ns-name (str *ns*)]
    `(update ~dct
             :err
             #(conj (or % []) {:message ~message, :plugin ~ns-name}))))
