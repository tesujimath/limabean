# Plugins and user-supplied code

External plugins in `limabean` are Clojure transducers.  Currently, they run on fully booked directives.  Running on raw directives may be possible in future, see [issue #46](https://github.com/tesujimath/limabean/issues/46).

In addition, arbitrary user-provided code may be loaded into the REPL.

There are also a handful of internal plugins, as follows.

## Internal Plugins

### Implicit Prices

The existing plugin `beancount.plugins.implicit_prices` is built in.

### Auto Accounts

The existing plugin `beancount.plugins.auto_accounts` is built-in.

### Balance Rollup

As described above, the plugin `limabean.balance_rollup` modifies the behaviour of the `balance` directive.

## External Plugins

External plugins are referenced in the Beanfile by their namespace, e.g.

```
plugin "limabean.contrib.plugins.examples.magic-money"
```

A single argument may be supplied, which must be provided as a Clojure map in a Beancount string, i.e. with escaped quotes, e.g.

```
plugin "limabean.contrib.plugins.examples.magic-money" "{:units 1000.00M :cur \"USD\" :acc \"Equity:Rich-American-Uncle\"}"
```

The Clojure package containing the desired namespace must be passed in the environment variable `LIMABEAN_CLJ_DEPS`, as e.g. `io.github.tesujimath/limabean-contrib {:mvn/version "0.1.0"}`.  This environment variable comprises a space-separated list of package name, co-ordinate pairs.
See the [Clojure deps reference](https://clojure.org/reference/deps_edn#deps) for what is possible, which includes local directories and git repos.

The Clojure namespace must define a function `booked-directive-xf`, which is a Clojure transducer on booked directives.

### Plugin namespaces

A limabean plugin namespace is simply a [Clojure namespace](https://guide.clojure.style/#naming-ns-naming-schemas).  Please avoid defining your own plugins in the `limabean` namespace, although [limabean.contrib.plugins]((https://github.com/tesujimath/limabean-contrib) is a good choice.  Otherwise, use your own domain.

### Magic Money example

The [magic-money example](https://github.com/tesujimath/limabean-contrib/src/limabean/contrib/plugins/examples/magic_money.clj) is a plugin which inserts additional directives, namely a transaction after every `open` directive to add some money to the account, from a specified equity account.

TODO describe `*directives*` vs `*booked-directives*`

```
user=> (show (journal))
2023-05-29  Expenses:Groceries   New World     10.00  NZD  10.00 NZD
2023-05-29  Assets:Bank:Current  New World    -10.00  NZD
2023-05-30  Expenses:Groceries   Countdown     17.50  NZD  17.50 NZD
2023-05-30  Assets:Bank:Current  Countdown    -17.50  NZD
:ok

user=> (def directives-with-magical (into [] (magic-money-xf) *directives* ))    ;; using default values for magic money

user=> (show (journal :directives directives-with-magical))
2016-03-01  Equity:Magic                               -100.00  NZD  -100.00 NZD
2016-03-01  Assets:Bank:Current  magical benefactor     100.00  NZD
2016-03-01  Equity:Magic                               -100.00  NZD  -100.00 NZD
2016-03-01  Expenses:Groceries   magical benefactor     100.00  NZD
2023-05-29  Expenses:Groceries   New World               10.00  NZD    10.00 NZD
2023-05-29  Assets:Bank:Current  New World              -10.00  NZD
2023-05-30  Expenses:Groceries   Countdown               17.50  NZD    17.50 NZD
2023-05-30  Assets:Bank:Current  Countdown              -17.50  NZD
:ok

user=> (def directives-with-magical-us(into [] (magic-money-xf {:units 1000.00M :cur "USD" :acc "Equity:Rich-American-Uncle"}) *directives*))

user=> (show (journal :directives directives-with-magical-us))
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

Using the Clojure REPL, the pre-defined variable `*directives*` and the user-defined `directives-with-magical` may easily be compared.


## User-provided code

The user may provide their own Clojure code.  The environment variable `LIMABEAN_USER_CLJ` is a colon-separated list of Clojure source files, which are loaded in order, and made available in the REPL.

For a very simple example, see the [user-supplied `fy` function](../../examples/clj/user.clj) for a customized financial year filter.
