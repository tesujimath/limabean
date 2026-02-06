# limabean

This is a new implementation of [Beancount](https://github.com/beancount/beancount) using [Rust](https://rust-lang.org) and [Clojure](https://clojure.org/) and the [Lima parser](https://github.com/tesujimath/beancount-parser-lima).

It is an implementation of Beancount in the sense that the file format and the booking algorithm are the same, although there are several new and different ideas.
Foremost among these being that the user interface is solely the Clojure REPL, with no support for [Beancount Query Language](https://beancount.github.io/docs/beancount_query_language.html) nor Python.
All the directives, inventory positions, and so on, are exposed as Clojure data structures, enabling the full power of Clojure for querying the ledger.

Rust is purely used for parsing and the booking algorithm, with essentially no visibility of this to end users.

- [Installation](clj/doc/20-installation.md)
- [Getting started](clj/doc/30-getting-started.md)
- [Plugins and user-provided code](clj/doc/40-plugins.md)
- [Development version](clj/doc/50-development.md)
- [Differences and gotchas](clj/doc/60-differences.md)
- [Design and rationale](clj/doc/70-design-and-rationale.md)
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
