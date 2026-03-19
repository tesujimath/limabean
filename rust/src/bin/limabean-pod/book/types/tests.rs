use rstest::rstest;
use rust_decimal_macros::dec;
use std::{borrow::Cow, cmp::Ordering};
use time::{Date, macros::date};

use super::*;

type CostTuple = (
    Date,
    Decimal,
    Decimal,
    &'static str,
    Option<&'static str>,
    bool,
);

fn cost((date, per_unit, total, currency, label, merge): CostTuple) -> Cost<'static> {
    Cost {
        per_unit,
        total,
        currency,
        date,
        label: label.map(Cow::Borrowed),
        merge,
    }
}

#[rstest]
#[case(((date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), (date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", None, false)), Ordering::Equal)]
#[case(((date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), (date!(2020-01-02), dec!(3.70), dec!(2.0), "NZD", None, false)), Ordering::Greater)]
#[case(((date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), (date!(2020-01-03), dec!(10.20), dec!(2.0), "NZD", None, false)), Ordering::Less)]
#[case(((date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), (date!(2020-01-02), dec!(10.20), dec!(2.0), "GBP", None, false)), Ordering::Greater)]
#[case(((date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", Some("fred"), false), (date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", None, false)), Ordering::Greater)]
#[case(((date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", Some("fred"), false), (date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", Some("jim"), false)), Ordering::Less)]
#[case(((date!(2020-01-02), dec!(10.20), dec!(1.0), "NZD", None, false), (date!(2020-01-02), dec!(10.20), dec!(2.0), "NZD", None, true)), Ordering::Less)]
fn cost_cmp<'a>(#[case] input: (CostTuple, CostTuple), #[case] expected: std::cmp::Ordering) {
    let (c0, c1) = input;
    assert_eq!(cost(c0).cmp(&cost(c1)), expected);
}
