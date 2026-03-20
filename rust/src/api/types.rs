use beancount_parser_lima as parser;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use time::Date;

use raw::Span;

/// A plugin
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Plugin<'a> {
    pub(crate) name: &'a str,
    pub(crate) config: Option<&'a str>,
}

/// A report for formatting in the context of the source files
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Report<'a> {
    pub(crate) message: Cow<'a, str>,
    pub(crate) reason: Cow<'a, str>,
    pub(crate) span: Span,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) contexts: Option<Vec<(Cow<'a, str>, Span)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) related: Option<Vec<(Cow<'a, str>, Span)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(borrow)]
    pub(crate) annotation: Option<Cow<'a, str>>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Element<'a> {
    element_type: &'a str,
}

impl<'a> parser::ElementType<'a> for Element<'a> {
    fn element_type(&self) -> &'a str {
        self.element_type
    }
}

impl<'a> From<&raw::Directive<'a>> for parser::Spanned<Element<'static>> {
    fn from(value: &raw::Directive<'a>) -> Self {
        parser::spanned(
            Element {
                element_type: (&value.variant).into(),
            },
            value.span.into(),
        )
    }
}

impl<'a> From<&raw::PostingSpec<'a>> for parser::Spanned<Element<'static>> {
    fn from(value: &raw::PostingSpec<'a>) -> Self {
        parser::spanned(
            Element {
                element_type: "posting",
            },
            value.span.into(),
        )
    }
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
