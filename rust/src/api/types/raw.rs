use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};
use strum_macros;
use time::Date;

use super::iso8601date;

/// A Beancount directive of a particular [DirectiveVariant].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Directive<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) span: Option<Span>,
    #[serde(with = "iso8601date")]
    pub(crate) date: Date,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tags: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) links: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<HashMap<&'a str, MetaValue<'a>>>,
    #[serde(borrow)]
    #[serde(flatten)]
    pub(crate) variant: DirectiveVariant<'a>,
}

/// A Beancount directive, without the fields common to all, which belong to [Directive].
#[derive(Serialize, Deserialize, PartialEq, Eq, strum_macros::IntoStaticStr, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "dct")]
#[strum(serialize_all = "kebab-case")]
pub enum DirectiveVariant<'a> {
    #[serde(rename = "txn")]
    Transaction(Transaction<'a>),
    Price(PriceDct<'a>),
    Balance(Balance<'a>),
    #[serde(borrow)]
    Open(Open<'a>),
    Close(Close<'a>),
    Commodity(Commodity<'a>),
    Pad(Pad<'a>),
    Document(Document<'a>),
    Note(Note<'a>),
    Event(Event<'a>),
    Query(Query<'a>),
    Custom(Custom<'a>),
}

/// A Beancount transaction directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Transaction<'a> {
    pub(crate) flag: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) payee: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) narration: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub(crate) postings: Vec<PostingSpec<'a>>,
}

/// A Beancount price directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PriceDct<'a> {
    pub(crate) cur: &'a str,
    pub(crate) price: Price<'a>,
}

/// A Beancount balance directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Balance<'a> {
    pub(crate) acc: &'a str,
    pub(crate) units: Decimal,
    pub(crate) cur: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tolerance: Option<Decimal>,
}

/// A Beancount open directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Open<'a> {
    pub(crate) acc: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) currencies: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) booking: Option<Booking>,
}

/// A Beancount close directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Close<'a> {
    pub(crate) acc: &'a str,
}

/// A Beancount commodity directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Commodity<'a> {
    pub(crate) cur: &'a str,
}

/// A Beancount pad directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Pad<'a> {
    pub(crate) acc: &'a str,
    pub(crate) source: &'a str,
}

/// A Beancount document directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Document<'a> {
    pub(crate) acc: &'a str,
    pub(crate) path: Cow<'a, str>,
}

/// A Beancount note directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Note<'a> {
    pub(crate) acc: &'a str,
    pub(crate) comment: Cow<'a, str>,
}

/// A Beancount event directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Event<'a> {
    #[serde(rename = "type")]
    pub(crate) type_: Cow<'a, str>,
    pub(crate) description: Cow<'a, str>,
}

/// A Beancount query directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Query<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) content: Cow<'a, str>,
}

/// A Beancount custom directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Custom<'a> {
    pub(crate) type_: Cow<'a, str>,
    // TODO custom meta values
}

/// A potentially incomplete posting-specification.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PostingSpec<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) span: Option<Span>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) flag: Option<Cow<'static, str>>,
    pub(crate) acc: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) units: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cur: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cost_spec: Option<CostSpec<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) price_spec: Option<PriceSpec<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tags: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) links: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<HashMap<&'a str, MetaValue<'a>>>,
}

/// A potentially incomplete cost-specification.
#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct CostSpec<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) per_unit: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cur: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "iso8601date::option")]
    #[serde(default)]
    pub(crate) date: Option<Date>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) label: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub(crate) merge: bool,
}

/// A potentially incomplete price-specification.
#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PriceSpec<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) per_unit: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cur: Option<&'a str>,
}

/// A complete price, with total iff it belongs to a posting.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Price<'a> {
    pub(crate) per_unit: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<Decimal>,
    pub(crate) cur: &'a str,
}

/// A metadata value
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum MetaValue<'a> {
    Amount(Decimal, &'a str), // units, currency
    String(Cow<'a, str>),
    Currency(&'a str),
    Account(&'a str),
    Tag(&'a str),
    Link(&'a str),
    Date(Date),
    Bool(bool),
    Number(Decimal),
    Null,
}

/// The booking method for an account.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Booking {
    Strict,
    StrictWithSize,
    None,
    Average,
    Fifo,
    Lifo,
    Hifo,
}

/// The booking method for an account.
#[derive(Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SpannedSource<'a> {
    /// File-name of source, only None in the case of the source being an inline string,
    /// which the API server never uses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<&'a str>,
    pub start_line: usize,
    pub end_line: usize,
    pub content: &'a str,
}

/// A span which identifies a source file location
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub(crate) struct Span {
    pub(crate) source: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

mod serializers;
