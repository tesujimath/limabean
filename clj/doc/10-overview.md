# Overview

This is a new implementation of [Beancount](https://github.com/beancount/beancount) using [Rust](https://rust-lang.org) and [Clojure](https://clojure.org/) and the [Lima parser](https://github.com/tesujimath/beancount-parser-lima).

Rust is purely used for backend processing, and has no visbility to end users beyond the build process.  The idea is to use Clojure for interactive Beancounting instead of
[Beancount Query Language](https://beancount.github.io/docs/beancount_query_language.html) and Python.  The Clojure REPL will provide all interactivity required.

There is no intention for `limabean` to support either Beancount Query Language or Python.  The interactive experience is provided solely by the Clojure REPL.

See also the [project on GitHub](https://github.com/tesujimath/limabean).
