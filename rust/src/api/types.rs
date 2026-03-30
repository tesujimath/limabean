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

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct IndexedErrorOrWarning {
    pub(crate) reason: String,
    pub(crate) element: IndexedElement,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) related: Option<Vec<IndexedElement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) annotation: Option<String>,
}

impl IndexedErrorOrWarning {
    pub(crate) fn related_to(mut self, element: &IndexedElement) -> Self {
        self.related.get_or_insert(Vec::default()).push(*element);
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

#[derive(Serialize, Copy, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct IndexedElement {
    #[serde(rename = "type")]
    pub(crate) element_type: &'static str,
    pub(crate) raw_idx: ElementIdx,
}

impl IndexedElement {
    pub(crate) fn error_or_warning<S>(self, reason: S) -> IndexedErrorOrWarning
    where
        S: Into<String>,
    {
        IndexedErrorOrWarning {
            reason: reason.into(),
            element: self,
            related: None,
            annotation: None,
        }
    }

    fn dct_idx(self) -> usize {
        self.raw_idx.dct_idx()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ElementIdx {
    Directive(usize),
    Posting(usize, usize),
}

impl ElementIdx {
    fn dct_idx(self) -> usize {
        use ElementIdx::*;

        match self {
            Directive(idx) => idx,
            Posting(idx, _) => idx,
        }
    }
}

impl From<usize> for ElementIdx {
    fn from(value: usize) -> Self {
        ElementIdx::Directive(value)
    }
}

impl From<(usize, usize)> for ElementIdx {
    fn from(value: (usize, usize)) -> Self {
        ElementIdx::Posting(value.0, value.1)
    }
}

impl<'a> From<(&raw::Directive<'a>, usize)> for IndexedElement {
    fn from(value: (&raw::Directive<'a>, usize)) -> Self {
        IndexedElement {
            element_type: (&value.0.variant).into(),
            raw_idx: value.1.into(),
        }
    }
}

impl<'a> From<&booked::Directive<'a>> for IndexedElement {
    fn from(value: &booked::Directive<'a>) -> Self {
        IndexedElement {
            element_type: (&value.variant).into(),
            raw_idx: value.raw_idx.into(),
        }
    }
}

impl From<(IndexedElement, usize)> for IndexedElement {
    fn from(value: (IndexedElement, usize)) -> Self {
        IndexedElement {
            element_type: "posting",
            raw_idx: (value.0.dct_idx(), value.1).into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct SpannedElement {
    pub(crate) element_type: &'static str,
    pub(crate) span: Span,
    pub(crate) context: Option<(&'static str, Span)>,
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
