use rstest::rstest;
use rust_decimal_macros::dec;
use std::cmp::Ordering;
use time::{Date, macros::date};

use super::*;

const fn cost(
    date: Date,
    per_unit: Decimal,
    total: Decimal,
    currency: &'static str,
    label: Option<&'static str>,
    merge: bool,
) -> Cost<'static> {
    Cost {
        per_unit,
        total,
        currency,
        date,
        label,
        merge,
    }
}

#[rstest]
#[case((cost(date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), cost(date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", None, false)), Ordering::Equal)]
#[case((cost(date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), cost(date!(2020-01-02), dec!(3.70), dec!(2.0), "NZD", None, false)), Ordering::Greater)]
#[case((cost(date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), cost(date!(2020-01-03), dec!(10.20), dec!(2.0), "NZD", None, false)), Ordering::Less)]
#[case((cost(date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), cost(date!(2020-01-02), dec!(10.20), dec!(2.0), "GBP", None, false)), Ordering::Greater)]
#[case((cost(date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", Some("fred"), false), cost(date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", None, false)), Ordering::Greater)]
#[case((cost(date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", Some("fred"), false), cost(date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", Some("jim"), false)), Ordering::Less)]
#[case((cost(date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), cost(date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", None, true)), Ordering::Less)]
fn cost_cmp<'a>(#[case] input: (Cost<'a>, Cost<'a>), #[case] expected: std::cmp::Ordering) {
    let (c0, c1) = input;
    assert_eq!(c0.cmp(&c1), expected);
}
