use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashSet};
use time::Date;

/// A Beancount directive of a particular [DirectiveVariant].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Directive<'a> {
    pub(crate) src: Source,
    #[serde(with = "serializers::iso8601date")]
    pub(crate) date: Date,
    // pub(crate) metadata: Metadata<'a>,
    #[serde(borrow)]
    #[serde(flatten)]
    pub(crate) variant: DirectiveVariant<'a>,
}

/// A Beancount directive, without the fields common to all, which belong to [Directive].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "dct")]
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
    pub(crate) payee: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) narration: Option<&'a str>,
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
    pub(crate) path: &'a str,
}

/// A Beancount note directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Note<'a> {
    pub(crate) acc: &'a str,
    pub(crate) comment: &'a str,
}

/// A Beancount event directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Event<'a> {
    #[serde(rename = "type")]
    pub(crate) type_: &'a str,
    pub(crate) description: &'a str,
}

/// A Beancount query directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Query<'a> {
    pub(crate) name: &'a str,
    pub(crate) content: &'a str,
}

/// A Beancount custom directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Custom<'a> {
    pub(crate) type_: &'a str,
    // TODO custom meta values
}

/// A potentially incomplete posting-specification.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct PostingSpec<'a> {
    pub(crate) src: Source,
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
    // TODO posting spec metadata
    // pub(crate) metadata: Spanned<Metadata<'a>>>,
}

/// A potentially incomplete cost-specification.
#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone, Copy, Debug)]
pub struct CostSpec<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) per_unit: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cur: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serializers::iso8601date::option")]
    pub(crate) date: Option<Date>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) label: Option<&'a str>,
    pub(crate) merge: bool,
}

/// A potentially incomplete price-specification.
#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone, Copy, Debug)]
pub struct PriceSpec<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) per_unit: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cur: Option<&'a str>,
}

/// A Beancount open directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Price<'a> {
    pub(crate) per_unit: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<Decimal>,
    pub(crate) cur: &'a str,
}

/// The booking method for an account.
#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Booking {
    #[default]
    Strict,
    StrictWithSize,
    None,
    Average,
    Fifo,
    Lifo,
    Hifo,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Source {
    file: usize,
    start: usize,
    end: usize,
}

mod from_parser_types;
mod serializers;
