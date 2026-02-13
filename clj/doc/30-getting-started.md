# Getting started

By default `limabean` will start a REPL for interactive use, for direct evaluation of Clojure expressions, so learning `limabean` is about learning Clojure, and the particular [Clojure functions and data structures](https://tesujimath.github.io/limabean/) it provides for querying a beanfile.

When `limabean` starts, it simply loads the beanfile into `*directives*` and `*options*`, which are exposed as Clojure variables in the REPL.

The top-level functions in the `limabean` Clojure namespace operate on these global variables, and also `*registry*`, which contains for example the booking method for each account (derived from options and `open` directives).

These functions build further Clojure data structures, generally maps and vectors, which may be inspected in the REPL directly, or tabulated using `show`.

```
kiri> limabean --help
limabean: usage: limabean [options]

Options:
  -h, --help                       Help
  -v, --verbose                    Verbose
      --beanfile PATH  <computed>  path to Beancount file, default $LIMABEAN_BEANFILE
      --eval EXPR                  Evaluate expression then exit


kiri> limabean --beanfile ./test-cases/simple.beancount

[Rebel readline] Type :repl/help for online help info
[limabean] 18 directives loaded from ./test-cases/simple.beancount


user=> (inventory)
{"Assets:US:TD:Checking" [{:units 23954.04M, :cur "USD", :cell/type :position}], "Expenses:Entertainment:Drinks-and-snacks" [{:units 25.00M, :cur "NZD", :cell/type :position}], "Expenses:Groceries" [{:units 39.65M, :cur "NZD", :cell/type :position}], "Assets:CA:RBC-Investing:Taxable-CAD:Cash" [{:units -1395.43M, :cur "CAD", :cell/type :position}], "Assets:Bank:Savings" [{:units 100.00M, :cur "NZD", :cell/type :position}], "Equity:Opening-Balances" [{:units -1164.65M, :cur "NZD", :cell/type :position} {:units -22996.64M, :cur "USD", :cell/type :position}], "Expenses:Car" [{:units 25.00M, :cur "NZD", :cell/type :position}], "Assets:Bank:Current" [{:units 720.00M, :cur "NZD", :cell/type :position}], "Expenses:Car:Fuel" [{:units 255.00M, :cur "NZD", :cell/type :position}]}
user=>

user=> (show (inventory))
Assets:Bank:Current                          720.00 NZD
Assets:Bank:Savings                          100.00 NZD
Assets:CA:RBC-Investing:Taxable-CAD:Cash   -1395.43 CAD
Assets:US:TD:Checking                      23954.04 USD
Equity:Opening-Balances                    -1164.65 NZD
                                          -22996.64 USD
Expenses:Car                                  25.00 NZD
Expenses:Car:Fuel                            255.00 NZD
Expenses:Entertainment:Drinks-and-snacks      25.00 NZD
Expenses:Groceries                            39.65 NZD
:ok


user=> (rollup)
{"Expenses:Entertainment:Drinks-and-snacks" {:item [25.00M 2], :cell/type :rollup/entry}, "Expenses:Groceries" {:item [39.65M 2], :cell/type :rollup/entry}, "Assets:Bank:Savings" {:item [100.00M 2], :cell/type :rollup/entry}, "Equity:Opening-Balances" {:item [-1164.65M 2], :cell/type :rollup/entry}, "Expenses:Car" {:item [25.00M 2], :total [280.00M 1], :cell/type :rollup/entry}, "Expenses:Entertainment" {:total [25.00M 1], :cell/type :rollup/entry}, "Assets:Bank:Current" {:item [720.00M 2], :cell/type :rollup/entry}, "Assets" {:total [820.00M 0], :cell/type :rollup/entry}, "Equity" {:total [-1164.65M 0], :cell/type :rollup/entry}, "Expenses" {:total [344.65M 0], :cell/type :rollup/entry}, "Expenses:Car:Fuel" {:item [255.00M 2], :cell/type :rollup/entry}, "Assets:Bank" {:total [820.00M 1], :cell/type :rollup/entry}}

user=> (show (rollup))
Assets                                      820.00
Assets:Bank                                         820.00
Assets:Bank:Current                                           720.00
Assets:Bank:Savings                                           100.00
Equity                                    -1164.65
Equity:Opening-Balances                                     -1164.65
Expenses                                    344.65
Expenses:Car                                        280.00     25.00
Expenses:Car:Fuel                                             255.00
Expenses:Entertainment                               25.00
Expenses:Entertainment:Drinks-and-snacks                       25.00
Expenses:Groceries                                             39.65
:ok
```

All these functions accept filters, of which [several are available](https://tesujimath.github.io/limabean/limabean.core.filters.html), with more planned to be added.

There is also a `journal` function, for building a journal of all postings with running balance.

```
user=> (journal (f/date>=< 2013 2018))
[{:date #object[java.time.LocalDate 0x5edd911f "2013-01-01"], :narration "Buy CRA shares", :acc "Assets:CA:RBC-Investing:Taxable-CAD:Cash", :units -1395.43M, :cur "CAD", :price {:per-unit 0.6861M, :cur "USD"}, :bal [{:units -1395.43M, :cur "CAD", :cell/type :position}], :cell/type :journal/entry} {:date #object[java.time.LocalDate 0x5edd911f "2013-01-01"], :narration "Buy CRA shares", :acc "Assets:US:TD:Checking", :units 957.40M, :cur "USD", :bal [{:units -1395.43M, :cur "CAD", :cell/type :position} {:units 957.40M, :cur "USD", :cell/type :position}], :cell/type :journal/entry} {:date #object[java.time.LocalDate 0x1ceefb02 "2017-11-17"], :acc "Assets:US:TD:Checking", :units 22996.64M, :cur "USD", :flag "'P", :bal [{:units -1395.43M, :cur "CAD", :cell/type :position} {:units 23954.04M, :cur "USD", :cell/type :position}], :cell/type :journal/entry} {:date #object[java.time.LocalDate 0x1ceefb02 "2017-11-17"], :acc "Equity:Opening-Balances", :units -22996.64M, :cur "USD", :flag "'P", :bal [{:units -1395.43M, :cur "CAD", :cell/type :position} {:units 957.40M, :cur "USD", :cell/type :position}], :cell/type :journal/entry}]
user=>

user=> (show (journal (f/date>=< 2013 2018)))
2013-01-01  Assets:CA:RBC-Investing:Taxable-CAD:Cash    Buy CRA shares   -1395.43  CAD  -1395.43 CAD
2013-01-01  Assets:US:TD:Checking                       Buy CRA shares     957.40  USD  -1395.43 CAD
                                                                                          957.40 USD
2017-11-17  Assets:US:TD:Checking                                        22996.64  USD  -1395.43 CAD
                                                                                        23954.04 USD
2017-11-17  Equity:Opening-Balances                                     -22996.64  USD  -1395.43 CAD
                                                                                          957.40 USD
:ok
```

Notice that in each case, the raw Clojure data structures are available for arbitrary processing by the user.  (Actually running these examples in the REPL gives nicely colourized output, which is not shown here.)

The intention is that `show` is smart enough to make a decent job of tabulating pretty much anything.  But it is rather early to make too big a claim there! 😅

## Further examples

Inventory pre-filtered to assets and liabilities
```
user=> (show (balances))
```

Just expenses and income for the calendar year 2025
```
user=> (show (inventory (f/date>=< 2025 2026) (f/sub-acc "Expenses" "Income")))
```

If you have brought your own `fy` function via [user provided code](40-plugins.md)
```
user=> (show (inventory (fy 25) (f/sub-acc "Expenses" "Income")))
```

This years transactions on current account
```
user=> (show (journal (f/date>= 2026) (f/acc "Assets:Bank:Current")))
```

Rollup for a secondary currency
```
user=> (show (rollup (f/cur "GBP")))
```

Payments made to Z Energy or Repco.  (Note that the `-match` filters take a [Clojure regular expression](https://clojure.org/reference/other_functions#regex).)
```
user=> (show (journal (f/sub-acc "Expenses") (f/payee-match #"Z Energy|Repco")))
```

All the payees as a set
```
user=> (set (keep :payee (journal)))
```

## Batch usage

While the REPL is envisaged as the primary interface to `limabean`, it is possible to invoke batch queries, for example:

```
kiri> limabean --eval '(show (journal (fy 23)))'
```

Quotes are essential, as what is being passed is Clojure code exactly as it would be typed into the REPL, and that is not at all shell-friendly.

## Environment variables

- `LIMABEAN_BEANFILE` - path to default beanfile unless overridden with `--beanfile`
- `LIMABEAN_USER_CLJ` - colon separated list of Clojure source files to load, containing [user-provided code](40-plugins.md)
- `LIMABEAN_CLJ_LOCAL_ROOT` - path to local Clojure source, when running the [development version](50-development.md)
- `LIMABEAN_LOG` - path to logfile, for troubleshooting
