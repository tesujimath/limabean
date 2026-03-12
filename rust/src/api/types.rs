use serde::{Deserialize, Serialize};
use time::Date;

use raw::Span;

/// A report for formatting in the context of the source files
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Report<'a> {
    pub(crate) kind: ReportKind,
    pub(crate) message: &'a str,
    pub(crate) label: &'a str,
    pub(crate) span: Span,
}

/// The booking method for an account.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ReportKind {
    Error,
    Warning,
}

/// Format a date as ISO8601
fn fmt_iso8601date(date: Date) -> String {
    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    date.format(&fmt).unwrap()
}

/// Parse a date as ISO8601
fn parse_iso8601date(s: &str) -> Result<Date, time::error::Parse> {
    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    Date::parse(s, &fmt)
}

time::serde::format_description!(pub(crate) iso8601date, Date, "[year]-[month]-[day]");

pub(crate) mod booked;
pub(crate) mod parser_type_conversions;
pub(crate) mod raw;
