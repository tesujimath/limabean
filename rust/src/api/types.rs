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

/// A synthetic span is a named content fragment we can subsequently reference as a span
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SyntheticSpan<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) content: Cow<'a, str>,
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

/// A report with elements identified by index rather than span.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct IndexedReport {
    pub(crate) reason: String,
    pub(crate) idx: ElementIdx,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) related: Option<Vec<ElementIdx>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) annotation: Option<String>,
}

impl IndexedReport {
    pub(crate) fn related_to(mut self, element: ElementIdx) -> Self {
        self.related.get_or_insert(Vec::default()).push(element);
        self
    }

    pub(crate) fn with_annotation<S>(mut self, annotation: S) -> Self
    where
        S: Into<String>,
    {
        self.annotation = Some(annotation.into());
        self
    }
}

/// A booking request
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BookingRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(borrow)]
    pub(crate) directives: Option<Vec<raw::Directive<'a>>>, // if omitted, use the as-parsed directives
    pub(crate) validate: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct ElementIdx {
    pub(crate) directive: usize,
    pub(crate) posting: Option<usize>,
}

impl ElementIdx {
    pub(crate) fn report<S>(self, reason: S) -> IndexedReport
    where
        S: Into<String>,
    {
        IndexedReport {
            reason: reason.into(),
            idx: self,
            related: None,
            annotation: None,
        }
    }
}

impl From<usize> for ElementIdx {
    fn from(value: usize) -> Self {
        ElementIdx {
            directive: value,
            posting: None,
        }
    }
}

impl From<(ElementIdx, usize)> for ElementIdx {
    fn from(value: (ElementIdx, usize)) -> Self {
        ElementIdx {
            directive: value.0.directive,
            posting: Some(value.1),
        }
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
mod serializers;
