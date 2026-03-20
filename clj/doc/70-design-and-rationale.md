# Design and Rationale

The ideas in `limabean` have been evolving since May 2023, with the [Rust parser](https://github.com/tesujimath/beancount-parser-lima) it uses.

Following this, a proof-of-concept of a front-end was built in [Steel Scheme](https://github.com/mattwparas/steel).  This validated the approach of using an established functional programming language in place of [Beancount Query Language](https://beancount.github.io/docs/beancount_query_language.html), but proved to be insufficiently mature for a polished user experience, especially around developer tooling.  (Steel Scheme is nonetheless an impressive project!)  At this stage, `limabean` pivoted to Clojure, a more established language and environment.

## Mixed language approach

By the time Clojure was introduced, the Rust parser was well established, along with an implementation of the Beancount booking algorithm in Rust.  Abandoning these in favour of Clojure-native implementation was extremely unappealing.  My experiments with Steel Scheme had cooled my enthusiasm for an FFI approach to the mixed language model, hence the use of the Rust parser and booking algorithm via the external program `limabean-pod`.

Initially `limabean-pod` was used by the Clojure code as a one-shot, passing its output on standard output in EDN format.  However, this was insufficiently flexible, and in particular failed to support Clojure plugins running in raw mode, which run before the booking algorithm.
This was the reason for introducing `limabean-pod serve` which is a JSON-RPC server which replaces `limabean-pod book`, and is available for a number of queries on the same parsed data, or on raw directives passed as JSON.

Notice how `limabean-pod` encapsulates all the complexities of the Beancount booking algorithm (in particular, reductions which involve matching of positions held at cost against cost specs).  Accumulating positions in the Clojure code is consequently simple and straightforward.

```
kiri> echo '{"jsonrpc": "2.0", "method": "book"}' | limabean-pod serve ../test-cases/trading.beancount | jq '[.result[] | select(.narration == "Selling all my blue chips.")]'
[
  {
    "span": [
      0,
      1235,
      1569
    ],
    "date": "2014-03-18",
    "dct": "txn",
    "flag": "*",
    "narration": "Selling all my blue chips.",
    "postings": [
      {
        "span": [
          0,
          1279,
          1376
        ],
        "acc": "Assets:US:ETrade:IBM",
        "units": -7,
        "cur": "IBM",
        "cost": {
          "date": "2014-02-16",
          "per-unit": 160.00,
          "total": 1600.00,
          "cur": "USD"
        }
      },
      {
        "span": [
          0,
          1279,
          1376
        ],
        "acc": "Assets:US:ETrade:IBM",
        "units": -5,
        "cur": "IBM",
        "cost": {
          "date": "2014-02-18",
          "per-unit": 180.00,
          "total": 900.00,
          "cur": "USD"
        }
      },
      {
        "span": [
          0,
          1379,
          1460
        ],
        "acc": "Assets:US:ETrade:Cash",
        "units": 2054.05,
        "cur": "USD"
      },
      {
        "span": [
          0,
          1463,
          1544
        ],
        "acc": "Expenses:Financial:Commissions",
        "units": 9.95,
        "cur": "USD"
      },
      {
        "span": [
          0,
          1547,
          1567
        ],
        "acc": "Income:US:ETrade:PnL",
        "units": -2052.00,
        "cur": "USD"
      }
    ]
  }
]
```

Alas the previously supported Beancount format output from `limabean-pod book` is not currently available, but used to appear like this:

```
kiri> limabean-pod book ../test-cases/trading.beancount
...

2014-03-18 * "Selling all my blue chips."
  Assets:US:ETrade:IBM -7 IBM {2014-02-16, 160.00 USD}
  Assets:US:ETrade:IBM -5 IBM {2014-02-18, 180.00 USD}
  Assets:US:ETrade:Cash 2054.05 USD
  Expenses:Financial:Commissions 9.95 USD
  Income:US:ETrade:PnL -2052.00 USD

...
```

Reinstating such a readable format is on the TODO list.  So too is documenting the JSON-RPC interface.

Previously EDN was preferred over JSON because of support for BigDecimals, but it turns out that JSON supports arbitrary precision decimals _if your JSON serialization library supports that_.

(Note that `limabean-booking` is available as a separate Rust crate with no dependencies on `limabean` or the parser, in case others wish to make use of it in other contexts.)

## Tabulation

The tabular output produced by `limabean show` was [implemented in Rust](https://github.com/tesujimath/tabulator), and again, I had little appetite to re-implement the layout algorithm in Clojure.  Therefore this Rust library was integrated into `limabean-pod` for ease of use by `limabean`, without requiring installation of the `tabulator` binary from that other repo.
