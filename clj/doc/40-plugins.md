# Plugins and user-supplied code

External plugins in `limabean` are [Clojure transducers](https://clojure.org/reference/transducers).  Currently, they run on fully booked directives.  Running on raw directives before validation is planned but not yet implemented, see [issue #46](https://github.com/tesujimath/limabean/issues/46).

In addition, arbitrary user-provided code may be loaded into the REPL (below).

There are also a handful of internal plugins, as follows.

## Internal Plugins

### Implicit Prices

The existing plugin `beancount.plugins.implicit_prices` is built in.

### Auto Accounts

The existing plugin `beancount.plugins.auto_accounts` is built-in.

### Balance Rollup

As described in [Differences from OG Beancount](60-differences.md), the plugin `limabean.balance_rollup` modifies the behaviour of the `balance` directive.

## External Plugins

External plugins are referenced in the Beanfile by their namespace, e.g.

```
plugin "limabean.contrib.plugins.examples.magic-money"
```

A single argument may be supplied, which is a single Clojure value in a Beancount string, i.e. with escaped quotes, e.g.

```
plugin "limabean.contrib.plugins.examples.magic-money" "{:units 1000.00M :cur \"USD\" :acc \"Equity:Rich-American-Uncle\"}"
```

In fact, any Clojure value may be supplied, not necessarily a map.  But it must match what the particular plugin is expecting.

The Clojure namespace must define a function `booked-xf`, which is a Clojure transducer on booked directives.
Soon (but not yet) it will be possible to define a function `raw-xf`, a Clojure transducer on raw directives, which runs before validation.

## Running plugins

Plugins are run automatically when loading a beanfile.  Any errors resolving a particular plugin will cause that plugin to be disabled (with an error message).  See `*plugins*` to see what has been applied and what has not.

The original directives loaded from the file are available in the REPL as `*booked-directives*`, with the post-plugin ones available as `*directives*`.  See the `set-narration` example below for how to use the original `*booked-directives*` instead of the post-plugin ones.

Any errors in actually running any plugin cause the whole pipeline to be discarded with an error message, in which case `*directives*` will be the same as `*booked-directived*`.

### Configuration required to resolve plugins

Plugins are Clojure code, so they must be on the Java class-path in order to be resolvable at runtime.  The `limabean` launcher supports running the clojure command line with the [`-Sdeps` option to pass the required dependencies](https://clojure.org/guides/deps_and_cli#command_line_deps).

The Clojure package containing the desired namespace should be passed in the environment variable `LIMABEAN_CLJ_DEPS`, as e.g. `io.github.tesujimath/limabean-contrib {:mvn/version "0.1.0"}`.  This environment variable comprises a space-separated list of package name, co-ordinate pairs, that is without the `{:deps {...} }` wrapper.

See the [Clojure deps reference](https://clojure.org/reference/deps_edn#deps) for what is possible, which includes local directories and git repos.

The following examples make use of `limabean-contrib` as a source for plugins, but you are free to create your own.  But do please consider contributing your plugins to [limabean-contrib](https://github.com/tesujimath/limabean-contrib).

To use a specified version from Clojars: `io.github.tesujimath/limabean-contrib {:mvn/version "0.1.0"}`

To use a library directly from GitHub: `io.github.tesujimath/limabean-contrib {:git/sha "bc55aa4105ca1b050fffe12301e1829c908a4689"}` - in this case the GitHub organization and repo are inferred from the library name, [unless overridden](https://clojure.org/reference/deps_edn#deps_git) using `:git/url`.

To use a library from a local path: `io.github.tesujimath/limabean-contrib {:local/root "/path/to/limabean-contrib"}`

As always, run with `limabean -v` to see what is going on with the Clojure invocation.

Note: it is not possible to load plugins when running in the standalone mode, which uses `java` rather than `clojure`.

### Plugin namespaces

A limabean plugin namespace is simply a [Clojure namespace](https://guide.clojure.style/#naming-ns-naming-schemas).  Please avoid defining your own plugins in the `limabean` namespace, although [limabean.contrib.plugins](https://github.com/tesujimath/limabean-contrib) is a good choice if you want to contribute your plugin there (please do!).  Otherwise, use your own domain.

## Examples

### Set narration

The test plugin [set-narration](../test/limabean/test/plugins/set_narration.clj) is the simplest possible plugin example, which overrides the narration field of each transaction according to its configuration, as in [this example beancount file](../../test-cases/set-narration-plugin-with-config.beancount).

```
kiri> limabean --beanfile ../examples/beancount/set-narration-plugin.beancount
[Rebel readline] Type :repl/help for online help info
[limabean] 4 directives loaded from ../examples/beancount/set-narration-plugin.beancount
[limabean] 4 directives resulting from running plugins

user=> (show (journal))
2023-05-29  Expenses:Groceries   New World  Plugins rule ok!   10.00  NZD  10.00 NZD
2023-05-29  Assets:Bank:Current  New World  Plugins rule ok!  -10.00  NZD
2023-05-30  Expenses:Groceries   Countdown  Plugins rule ok!   17.50  NZD  17.50 NZD
2023-05-30  Assets:Bank:Current  Countdown  Plugins rule ok!  -17.50  NZD
:ok

user=> (binding [*directives* *booked-directives*] (show (journal)))
2023-05-29  Expenses:Groceries   New World     10.00  NZD  10.00 NZD
2023-05-29  Assets:Bank:Current  New World    -10.00  NZD
2023-05-30  Expenses:Groceries   Countdown     17.50  NZD  17.50 NZD
2023-05-30  Assets:Bank:Current  Countdown    -17.50  NZD
:ok
```

### Running plugins manually

The external resolved plugins are readily available in the `*plugins*` map, so may be applied manually.

```
user=> *plugins*
{:internal [],
 :external [{:name "limabean.contrib.plugins.examples.set-narration",
             :config "{:narration \"Plugins rule ok!\"}",
             :booked-xf #object[limabean.contrib.plugins.examples.set_narration$booked_xf$fn__16968 0x1ecc1a99
                                "limabean.contrib.plugins.examples.set_narration$booked_xf$fn__16968@1ecc1a99"]}]}

user=> (def set-narration-xf (get-in *plugins* [:external 0 :booked-xf]))

user=> (into [] set-narration-xf *booked-directives*)
[{:date #object[java.time.LocalDate 0x3922c5bc "2016-03-01"], :dct :open, :acc "Assets:Bank:Current"}
 {:date #object[java.time.LocalDate 0x63190b1 "2016-03-01"], :dct :open, :acc "Expenses:Groceries"}
 {:date #object[java.time.LocalDate 0x4325de9e "2023-05-29"], :dct :txn, :flag "*", :payee "New World",
  :postings [{:acc "Expenses:Groceries", :units 10.00M, :cur "NZD"}
             {:acc "Assets:Bank:Current", :units -10.00M, :cur "NZD"}], :narration "Plugins rule ok!"}
 {:date #object[java.time.LocalDate 0x1c0b38af "2023-05-30"], :dct :txn, :flag "*", :payee "Countdown",
  :postings [{:acc "Expenses:Groceries", :units 17.50M, :cur "NZD"}
             {:acc "Assets:Bank:Current", :units -17.50M, :cur "NZD"}], :narration "Plugins rule ok!"}]
```

### Magic Money

The [magic-money example](https://github.com/tesujimath/limabean-contrib/blob/main/src/limabean/contrib/plugins/examples/magic_money.clj) is a more sophisticated plugin which inserts additional directives, namely a transaction after every `open` directive to add some money to the account, from a specified equity account.  It works as a [stateful transducer](https://clojure.org/reference/transducers#_transducers_with_reduction_state).

```
kiri> limabean --beanfile ../examples/beancount/magic-money-plugin.beancount
[Rebel readline] Type :repl/help for online help info
[limabean] 4 directives loaded from ../examples/beancount/magic-money-plugin.beancount
[limabean] 7 directives resulting from running plugins

user=> (show (journal))
2016-03-01  Equity:Rich-American-Uncle                        -1000.00  USD  -1000.00 USD
2016-03-01  Assets:Bank:Current         magical benefactor     1000.00  USD
2016-03-01  Equity:Rich-American-Uncle                        -1000.00  USD  -1000.00 USD
2016-03-01  Expenses:Groceries          magical benefactor     1000.00  USD
2023-05-29  Expenses:Groceries          New World                10.00  NZD     10.00 NZD
2023-05-29  Assets:Bank:Current         New World               -10.00  NZD
2023-05-30  Expenses:Groceries          Countdown                17.50  NZD     17.50 NZD
2023-05-30  Assets:Bank:Current         Countdown               -17.50  NZD
:ok
```

## User-provided code

The user may provide their own Clojure code.  The environment variable `LIMABEAN_USER_CLJ` is a colon-separated list of Clojure source files, which are loaded in order, and made available in the REPL.
This facility is not suitable for plugins, because the functions are loaded too late.  But it is a useful place for defining custom filters.

For a very simple example, see the [user-supplied `fy` function](../../examples/clj/user.clj) for a customized financial year filter.
