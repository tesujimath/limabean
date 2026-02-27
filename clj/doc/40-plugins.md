# Plugins and user-supplied code

`limabean` does not support externally provided plugins.  The intention is that all desired behaviour may be implemented by the end user in Clojure.

That said, there are a handful of internal plugins, as follows.

## Internal Plugins

### Implicit Prices

The existing plugin `beancount.plugins.implicit_prices` is built in.

### Auto Accounts

The existing plugin `beancount.plugins.auto_accounts` is built-in.

### Balance Rollup

As described above, the plugin `limabean.balance_rollup` modifies the behaviour of the `balance` directive.

## User-provided code

The user may provide their own Clojure code.  The environment variable `LIMABEAN_USER_CLJ` is a colon-separated list of Clojure source files, which are loaded in order, and made available in the REPL.

For a very simple example, see the [user-supplied `fy` function](../../examples/clj/user.clj) for a customized financial year filter.

For an example of inserting additional directives, see the [user-supplied `magic-money-xf` function](../../examples/clj/user.clj).  This inserts a transaction after every `open` directive to add some money to the account, from a specified equity account.

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
