use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use time::Date;

/// A Beancount directive of a particular [DirectiveVariant].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Directive<'a> {
    #[serde(rename = "src")]
    pub(crate) source: Source,
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
    // Transaction(Transaction<'a>),
    // Price(Price<'a>),
    // Balance(Balance<'a>),
    #[serde(borrow)]
    Open(Open<'a>),
    // Close(Close<'a>),
    // Commodity(Commodity<'a>),
    // Pad(Pad<'a>),
    // Document(Document<'a>),
    // Note(Note<'a>),
    Event(Event<'a>),
    // Query(Query<'a>),
    // Custom(Custom<'a>),
}

/// A Beancount open directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Open<'a> {
    #[serde(rename = "acc")]
    pub(crate) account: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) currencies: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) booking: Option<Booking>,
}

/// A Beancount event directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Event<'a> {
    #[serde(rename = "type")]
    pub(crate) _type: &'a str,
    pub(crate) description: &'a str,
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
