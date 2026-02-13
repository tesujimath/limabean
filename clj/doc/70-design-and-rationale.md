# Design and Rationale

The ideas in `limabean` have been evolving since May 2023, with the [Rust parser](https://github.com/tesujimath/beancount-parser-lima) it uses.

Following this, a proof-of-concept of a front-end was built in [Steel Scheme](https://github.com/mattwparas/steel).  This validated the approach of using an established functional programming language in place of [Beancount Query Language](https://beancount.github.io/docs/beancount_query_language.html), but proved to be insufficiently mature for a polished user experience, especially around developer tooling.  (Steel Scheme is nonetheless an impressive project!)  At this stage, `limabean` pivoted to Clojure, a more established language and environment.

## Mixed language approach

By the time Clojure was introduced, the Rust parser was well established, along with an implementation of the Beancount booking algorithm in Rust.  Abandoning these in favour of Clojure-native implementation was extremely unappealing.  My experiments with Steel Scheme had cooled my enthusiasm for an FFI approach to the mixed language model, hence the use of the Rust parser and booking algorithm via the external program `limabean-pod`.

Notice how `limabean-pod` encapsulates all the complexities of the Beancount booking algorithm (in particular, reductions which involve matching of positions held at cost against cost specs).  Accumulating positions in the Clojure code is consequently simple and straightforward.

```
kiri> limabean-pod book ../test-cases/trading.beancount
1970-01-01 commodity USD

1970-01-01 commodity AAPL

2010-03-01 open Assets:US:ETrade:Cash

2010-03-01 open Assets:US:ETrade:IBM

2010-03-01 open Expenses:Financial:Commissions

2010-03-01 open Income:US:ETrade:PnL

2014-02-16 * "Buying some IBM"
  Assets:US:ETrade:IBM 10 IBM {2014-02-16, 160.00 USD}
  Assets:US:ETrade:Cash -1609.95 USD
  Expenses:Financial:Commissions 9.95 USD

2014-02-17 * "Selling some IBM"
  Assets:US:ETrade:IBM -3 IBM {2014-02-16, 160.00 USD}
  Assets:US:ETrade:Cash 500.05 USD
  Expenses:Financial:Commissions 9.95 USD
  Income:US:ETrade:PnL -507.00 USD

2014-02-18 * "I put my chips on big blue!"
  Assets:US:ETrade:IBM 5 IBM {2014-02-18, 180.00 USD}
  Assets:US:ETrade:Cash -909.95 USD
  Expenses:Financial:Commissions 9.95 USD

2014-03-18 * "Selling all my blue chips."
  Assets:US:ETrade:IBM -7 IBM {2014-02-16, 160.00 USD}
  Assets:US:ETrade:IBM -5 IBM {2014-02-18, 180.00 USD}
  Assets:US:ETrade:Cash 2054.05 USD
  Expenses:Financial:Commissions 9.95 USD
  Income:US:ETrade:PnL -2052.00 USD

2014-04-16 * "Buying some more IBM"
  Assets:US:ETrade:IBM 10 IBM {2014-04-16, 173.00 USD}
  Assets:US:ETrade:Cash -1739.95 USD
  Expenses:Financial:Commissions 9.95 USD

2014-04-26 * "Buying IBM yet again"
  Assets:US:ETrade:IBM 10 IBM {2014-04-26, 180.00 USD}
  Assets:US:ETrade:Cash -1809.95 USD
  Expenses:Financial:Commissions 9.95 USD

2014-05-01 * "Selling some older blue chips."
  Assets:US:ETrade:IBM -4 IBM {2014-04-16, 173.00 USD}
  Assets:US:ETrade:Cash 740.00 USD
  Expenses:Financial:Commissions 9.95 USD
  Income:US:ETrade:PnL -745.95 USD
  ```

To avoid re-parsing the Beancount file format in Clojure, `limabean-pod book` supports output in [EDN](https://github.com/edn-format/edn), which is read natively by Clojure (and, critically, supports BigDecimals, unlike say, JSON).

(Note that `limabean-booking` is available as a separate Rust crate with no dependencies on `limabean` or the parser, in case others wish to make use of it in other contexts.)

## Tabulation

The tabular output produced by `limabean show` was [implemented in Rust](https://github.com/tesujimath/tabulator), and again, I had little appetite to re-implement the layout algorithm in Clojure.  Therefore this Rust library was integrated into `limabean-pod` for ease of use by `limabean`, without requiring installation of the `tabulator` binary from that other repo.
