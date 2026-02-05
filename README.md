# limabean

This is a new implementation of [Beancount](https://github.com/beancount/beancount) using [Rust](https://rust-lang.org) and [Clojure](https://clojure.org/) and the [Lima parser](https://github.com/tesujimath/beancount-parser-lima).

Rust is purely used for backend processing, and has no visbility to end users beyond the build process.  The idea is to use Clojure for interactive Beancounting instead of
[Beancount Query Language](https://beancount.github.io/docs/beancount_query_language.html) and Python.  The Clojure REPL will provide all interactivity required.

There is no intention for `limabean` to support either Beancount Query Language or Python.  The interactive experience is provided solely by the Clojure REPL.

- [Installation](clj/doc/20-installation.md)
- [Getting started](clj/doc/30-getting-started.md)
- [Plugins and user-provided code](clj/doc/40-plugins.md)
- [Development version](clj/doc/50-development.md)
- [Differences and gotchas](clj/doc/60-differences.md)
- [Reference manual](https://tesujimath.github.io/limabean)

## Contributions

While issues are welcome and I am particularly interested in making this generally useful to others, given the current pace of development I am unlikely to be able to accept PRs for now.

I am, however, very interested to hear what people think is the priority for adding not-yet-implemented features (of which there are several).

The best place for general discussion of `limabean` is the [GitHub discussions page](https://github.com/tesujimath/limabean/discussions).

## License

Licensed under either of

 * Apache License, Version 2.0
   [LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   [LICENSE-MIT](http://opensource.org/licenses/MIT)

at your option.
