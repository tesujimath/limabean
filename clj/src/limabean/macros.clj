(ns limabean.macros)

(defmacro bind->
  "Thread expr through forms, short-circuiting if any step returns an :error"
  [expr & forms]
  (reduce (fn [acc form]
            `(let [r# ~acc]
               (if (:error r#) r# (~(first form) r# ~@(next form)))))
    expr
    forms))
