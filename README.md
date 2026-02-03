# limabean

This is a new implementation of [Beancount](https://github.com/beancount/beancount) using [Rust](https://rust-lang.org) and [Clojure](https://clojure.org/) and the [Lima parser](https://github.com/tesujimath/beancount-parser-lima).

Rust is purely used for backend processing, and has no visbility to end users beyond the build process.  The idea is to use Clojure for interactive Beancounting instead of
[Beancount Query Language](https://beancount.github.io/docs/beancount_query_language.html) and Python.  The Clojure REPL will provide all interactivity required.

There is no intention for `limabean` to support either Beancount Query Language or Python.

Some pre-canned queries are likely to be provided as command line options, but the main interactive experience is intended to be within the Clojure REPL.

## Installation

Tarballs and zipfiles are provided for each GitHub release for Linux, macOS, and Windows.

The [Clojure CLI](https://clojure.org/reference/clojure_cli) is required to be installed separately, and `clojure` must be on the user's path.

On macOS it is necessary to remove the quarantine attributes after unpacking the tarball, e.g.

```
xattr -rd com.apple.quarantine ./limabean/bin
```

## Usage

```
kiri> limabean --beanfile ./examples/beancount/simple.beancount

[Rebel readline] Type :repl/help for online help info
[limabean] 18 directives loaded from ./examples/beancount/simple.beancount
user=> (show (inventory))
Assets:Bank:Current                       -100.78 NZD
Assets:Bank:UK                              -5.00 GBP
Expenses:Donations                          10.00 NZD
Expenses:Donations:Other                    20.00 NZD
Expenses:Entertainment:Drinks-and-snacks    48.00 NZD
Expenses:Groceries                           5.00 GBP
                                            27.50 NZD
Income:Unknown                              -4.72 NZD
:ok
```

_To be completed_

## Import

For a new approach to import see [limabean-harvest](https://github.com/tesujimath/limabean-harvest).

## Balance assertions

A point of difference from classic Beancount is that balance assertions may be configured to assert the total for an account an all its subaccounts, using
the internal plugin `limabean.balance_rollup`.  For example, if a bank account holds multiple logical amounts, they may be tracked as subaccounts, without violating
balance assertions.

Padding is only ever performed on the actual account asserted in the balance directive, never on its subaccounts.

Unless the plugin is enabled, the default behaviour is not to do this.

## Plugins

`limabean` does not (yet) support externally provided plugins.  The intention is that all desired behaviour may be implemented by the end user in Clojure. It remains to be seen whether auto-loading of Clojure plugins will be a useful feature.

That said, there are a handful of internal plugins, as follows.

### Implicit Prices

The existing plugin `beancount.plugins.implicit_prices` is built in.

### Auto Accounts

The existing plugin `beancount.plugins.auto_accounts` is not yet supported, but will be implemented as a built-in plugin.

### Balance Rollup

As described above, the plugin `limabean.balance_rollup` modifies the behaviour of the `balance` directive.

## Contributions

While issues are welcome and I am particularly interested in making this generally useful to others, given the current pace of development I am unlikely to be able to accept PRs for now.

## License

Licensed under either of

 * Apache License, Version 2.0
   [LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   [LICENSE-MIT](http://opensource.org/licenses/MIT)

at your option.
