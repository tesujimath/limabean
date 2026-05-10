#![cfg(test)]

use beancount_parser_lima::{BeancountParser, BeancountSources, ParseSuccess};

use crate::api::types::raw;

fn book_str_ok(source: &str) -> bool {
    let sources = BeancountSources::from(source);
    let parser = BeancountParser::new(&sources);
    let ParseSuccess {
        directives,
        options,
        ..
    } = parser.parse().expect("parse should succeed");
    let raw_directives: Vec<raw::Directive<'_>> = directives.iter().map(Into::into).collect();
    super::book(&raw_directives, &options).is_ok()
}

#[test]
fn balance_within_inferred_tolerance_not_flagged() {
    // Actual: 100.01, asserted: 100.00, diff: 0.01.
    // Beancount infers tolerance = 2 × 0.5 × 0.01 = 0.01 for 2-decimal amounts.
    // 0.01 is not > 0.01, so no error.
    assert!(
        book_str_ok(
            r#"2025-01-01 open Assets:Bank
2025-01-01 open Equity:Opening
2025-01-01 * "Deposit"
  Assets:Bank  100.01 USD
  Equity:Opening  -100.01 USD
2025-01-02 balance Assets:Bank  100.00 USD
"#
        ),
        "balance within inferred tolerance (0.01) should not be an error"
    );
}

#[test]
fn balance_beyond_inferred_tolerance_is_flagged() {
    // Actual: 100.02, asserted: 100.00, diff: 0.02 > tolerance 0.01 → error.
    assert!(
        !book_str_ok(
            r#"2025-01-01 open Assets:Bank
2025-01-01 open Equity:Opening
2025-01-01 * "Deposit"
  Assets:Bank  100.02 USD
  Equity:Opening  -100.02 USD
2025-01-02 balance Assets:Bank  100.00 USD
"#
        ),
        "balance beyond inferred tolerance (0.02 > 0.01) should be an error"
    );
}
