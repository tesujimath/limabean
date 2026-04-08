use limabean_booking::LimaParserBookingTypes;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};
use time::Date;

use crate::api::types::{iso8601date, raw};

/// A Beancount directive of a particular [DirectiveVariant].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Directive<'a> {
    pub(crate) raw_idx: usize,
    #[serde(with = "iso8601date")]
    pub(crate) date: Date,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tags: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) links: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<HashMap<&'a str, raw::MetaValue<'a>>>,
    #[serde(borrow)]
    #[serde(flatten)]
    pub(crate) variant: DirectiveVariant<'a>,
}

/// A Beancount directive, without the fields common to all, which belong to [Directive].
#[derive(Serialize, Deserialize, PartialEq, Eq, strum_macros::IntoStaticStr, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
#[strum(serialize_all = "kebab-case")]
pub enum DirectiveVariant<'a> {
    #[serde(rename = "limabean/txn")]
    Transaction(Transaction<'a>),
    #[serde(rename = "limabean/price")]
    Price(raw::PriceDct<'a>),
    #[serde(rename = "limabean/balance")]
    Balance(raw::Balance<'a>),
    #[serde(borrow)]
    #[serde(rename = "limabean/open")]
    Open(raw::Open<'a>),
    #[serde(rename = "limabean/close")]
    Close(raw::Close<'a>),
    #[serde(rename = "limabean/commodity")]
    Commodity(raw::Commodity<'a>),
    #[serde(rename = "limabean/pad")]
    Pad(raw::Pad<'a>),
    #[serde(rename = "limabean/document")]
    Document(raw::Document<'a>),
    #[serde(rename = "limabean/note")]
    Note(raw::Note<'a>),
    #[serde(rename = "limabean/event")]
    Event(raw::Event<'a>),
    #[serde(rename = "limabean/query")]
    Query(raw::Query<'a>),
    #[serde(rename = "limabean/custom")]
    Custom(raw::Custom<'a>),
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
    pub(crate) postings: Vec<Posting<'a>>,
}

/// A complete posting.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Posting<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) raw_idx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) flag: Option<Cow<'static, str>>,
    pub(crate) acc: &'a str,
    pub(crate) units: Decimal,
    pub(crate) cur: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cost: Option<Cost<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) price: Option<Price<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tags: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) links: Option<HashSet<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<HashMap<&'a str, raw::MetaValue<'a>>>,
}

/// A cost complete with any fields which were missing from its [CostSpec].
///
/// In addition to `per-unit` which is the natural representation, the `total`
/// is also exposed, since this may be what the user originally specified in the
/// beanfile, and ought to be preserved at its original precision.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Cost<'a> {
    #[serde(with = "iso8601date")]
    pub(crate) date: Date,
    pub(crate) per_unit: Decimal,
    pub(crate) total: Decimal,
    pub(crate) cur: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) label: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub(crate) merge: bool,
}

impl<'a> From<&'a limabean_booking::Cost<LimaParserBookingTypes<'a>>> for Cost<'a> {
    fn from(value: &'a limabean_booking::Cost<LimaParserBookingTypes>) -> Self {
        Cost {
            date: value.date,
            per_unit: value.per_unit,
            total: value.total,
            cur: value.currency,
            label: value.label.clone(),
            merge: value.merge,
        }
    }
}
///
/// A price complete with any fields which were missing from its [PriceSpec].
///
/// In addition to `per-unit` which is the natural representation, the `total`
/// is also exposed, since this may be what the user originally specified in the
/// beanfile, and ought to be preserved at its original precision.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Price<'a> {
    pub(crate) per_unit: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<Decimal>,
    pub(crate) cur: &'a str,
}
