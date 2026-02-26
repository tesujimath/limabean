use beancount_parser_lima as parser;
use limabean_booking::LimaParserBookingTypes;
use rust_decimal::Decimal;
use std::collections::HashSet;
use tabulator::{Align, Cell};

use crate::format::GUTTER_MINOR;

#[derive(Clone, Debug)]
pub(crate) struct Directive<'a> {
    pub(crate) parsed: &'a parser::Spanned<parser::Directive<'a>>,
    pub(crate) loaded: DirectiveVariant<'a>,
}

#[derive(Clone, Debug)]
pub(crate) enum DirectiveVariant<'a> {
    NA, // not applicable, as no extra data at load stage for this variant
    Transaction(Transaction<'a>),
    Pad(Pad<'a>),
}

#[derive(Clone, Debug)]
pub(crate) struct Transaction<'a> {
    pub(crate) postings: Vec<Posting<'a>>,
    // TODO use for implicit prices plugin if enabled
    pub(crate) _prices: HashSet<(parser::Currency<'a>, parser::Currency<'a>, Decimal)>,
    pub(crate) auto_accounts: HashSet<&'a str>,
}

#[derive(Clone, Debug)]
pub(crate) struct Pad<'a> {
    pub(crate) postings: Vec<Posting<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Posting<'a> {
    pub(crate) flag: Option<parser::Flag>,
    pub(crate) account: &'a str,
    pub(crate) units: Decimal,
    pub(crate) currency: parser::Currency<'a>,
    pub(crate) cost: Option<Cost<'a>>,
    pub(crate) price: Option<Price<'a>>,
    // pub(crate) metadata: Metadata<'a>,
}

pub(crate) type Cost<'a> = limabean_booking::Cost<limabean_booking::LimaParserBookingTypes<'a>>;

pub(crate) fn cost_into_cell<'a>(cost: Cost<'a>) -> Cell<'a, 'static> {
    let Cost {
        date,
        per_unit,
        currency,
        label: _label,
        merge: _merge,
    } = cost;
    let mut cells = vec![
        (date.to_string(), Align::Left).into(),
        per_unit.into(),
        (Into::<&str>::into(currency), Align::Left).into(),
    ];
    if let Some(label) = &cost.label {
        cells.push((*label, Align::Left).into())
    }
    if cost.merge {
        cells.push(("*", Align::Left).into())
    }
    Cell::Row(cells, GUTTER_MINOR)
}

pub(crate) type PostingCost<'a> =
    limabean_booking::PostingCost<limabean_booking::LimaParserBookingTypes<'a>>;

pub(crate) fn cur_posting_cost_to_cost<'a>(
    currency: parser::Currency<'a>,
    cost: PostingCost<'a>,
) -> Cost<'a> {
    Cost {
        date: cost.date,
        per_unit: cost.per_unit,
        currency,
        label: cost.label,
        merge: cost.merge,
    }
}

pub(crate) type Price<'a> = limabean_booking::Price<limabean_booking::LimaParserBookingTypes<'a>>;

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Amount<'a> {
    pub(crate) number: Decimal,
    pub(crate) currency: parser::Currency<'a>,
}

impl<'a> From<(Decimal, parser::Currency<'a>)> for Amount<'a> {
    fn from(value: (Decimal, parser::Currency<'a>)) -> Self {
        Self {
            number: value.0,
            currency: value.1,
        }
    }
}

impl<'a> From<&'a parser::Amount<'a>> for Amount<'a> {
    fn from(value: &'a parser::Amount<'a>) -> Self {
        Amount {
            number: value.number().value(),
            currency: *value.currency().item(),
        }
    }
}

impl<'a> From<Amount<'a>> for Cell<'static, 'static> {
    fn from(value: Amount) -> Self {
        Cell::Row(
            vec![
                value.number.into(),
                (value.currency.to_string(), Align::Left).into(),
            ],
            GUTTER_MINOR,
        )
    }
}

impl<'a, 'b> From<&'b Amount<'a>> for Cell<'a, 'static>
where
    'b: 'a,
{
    fn from(value: &'b Amount<'a>) -> Self {
        Cell::Row(
            vec![
                value.number.into(),
                (value.currency.as_ref(), Align::Left).into(),
            ],
            GUTTER_MINOR,
        )
    }
}

pub(crate) type Positions<'a> =
    limabean_booking::Positions<limabean_booking::LimaParserBookingTypes<'a>>;

// should be From, but both types are third-party
pub(crate) fn positions_into_cell<'a>(positions: Positions<'a>) -> Cell<'a, 'static> {
    Cell::Stack(
        positions
            .into_iter()
            .map(position_into_cell)
            .collect::<Vec<_>>(),
    )
}

pub(crate) type Position<'a> = limabean_booking::Position<LimaParserBookingTypes<'a>>;

pub(crate) fn position_into_cell<'a>(position: Position<'a>) -> Cell<'a, 'static> {
    let Position {
        units,
        currency,
        cost,
    } = position;
    let mut cells = vec![
        units.into(),
        (Into::<&str>::into(currency), Align::Left).into(),
    ];
    if let Some(cost) = cost {
        cells.push(cost_into_cell(cost))
    }
    Cell::Row(cells, GUTTER_MINOR)
}

#[derive(Clone, Debug)]
pub(crate) struct Element {
    element_type: &'static str,
}

impl Element {
    pub(crate) fn new(element_type: &'static str, span: parser::Span) -> parser::Spanned<Self> {
        parser::spanned(Element { element_type }, span)
    }
}

impl parser::ElementType for Element {
    fn element_type(&self) -> &'static str {
        self.element_type
    }
}

pub(crate) fn into_spanned_element<T>(value: &parser::Spanned<T>) -> parser::Spanned<Element>
where
    T: parser::ElementType,
{
    parser::spanned(
        Element {
            element_type: value.element_type(),
        },
        *value.span(),
    )
}

#[cfg(test)]
mod tests;
