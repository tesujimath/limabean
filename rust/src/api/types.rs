use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use tabulator::Cell;
use time::Date;

use raw::Span;

/// A report for formatting in the context of the source files
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Report<'a> {
    pub(crate) kind: ReportKind,
    pub(crate) message: Cow<'a, str>,
    pub(crate) label: Cow<'a, str>,
    pub(crate) span: Span,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) contexts: Option<Vec<(Cow<'a, str>, Span)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) related: Option<Vec<(Cow<'a, str>, Span)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(borrow)]
    pub(crate) annotation: Option<Cell<'a, 'a>>,
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
pub(crate) mod booking_type_conversions;
pub(crate) mod parser_type_conversions;
pub(crate) mod raw;
