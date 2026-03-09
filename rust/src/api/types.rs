use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use time::Date;

/// A Beancount directive of a particular [DirectiveVariant].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Directive<'a> {
    pub(crate) date: Date,
    // pub(crate) metadata: Metadata<'a>,
    #[serde(borrow)]
    pub(crate) variant: DirectiveVariant<'a>,
}

/// A Beancount directive, without the fields common to all, which belong to [Directive].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
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
    // Event(Event<'a>),
    // Query(Query<'a>),
    // Custom(Custom<'a>),
}

/// A Beancount open directive, without the common [Directive] fields.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Open<'a> {
    pub(crate) account: &'a str,
    pub(crate) currencies: HashSet<&'a str>,
    pub(crate) booking: Option<Booking>,
}

/// The booking method for an account.
#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Source {
    file: u32,
    start: usize,
    end: usize,
}
