use beancount_parser_lima as parser;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use super::{ReportKind, raw::*};

impl<'a, 'b> From<&'b parser::Spanned<parser::Directive<'a>>> for Directive<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Spanned<parser::Directive<'a>>) -> Self {
        Directive {
            span: value.into(),
            date: *value.date().item(),
            variant: value.variant().into(),
            tags: from_tags(value.metadata().tags()),
            links: from_links(value.metadata().links()),
            metadata: from_key_values(value.metadata().key_values()),
        }
    }
}

impl<'a, 'b> From<&'b parser::DirectiveVariant<'a>> for DirectiveVariant<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::DirectiveVariant<'a>) -> Self {
        use DirectiveVariant::*;
        use parser::DirectiveVariant as parser;

        match value {
            parser::Transaction(transaction) => Transaction(transaction.into()),
            parser::Price(price) => Price(price.into()),
            parser::Balance(balance) => Balance(balance.into()),
            parser::Open(open) => Open(open.into()),
            parser::Close(close) => Close(close.into()),
            parser::Commodity(commodity) => Commodity(commodity.into()),
            parser::Pad(pad) => Pad(pad.into()),
            parser::Document(document) => Document(document.into()),
            parser::Note(note) => Note(note.into()),
            parser::Event(event) => Event(event.into()),
            parser::Query(query) => Query(query.into()),
            parser::Custom(custom) => Custom(custom.into()),
        }
    }
}

impl<'a, 'b> From<&'b parser::Transaction<'a>> for Transaction<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Transaction<'a>) -> Self {
        Transaction {
            flag: from_flag(*value.flag().item()),
            payee: value.payee().map(|x| *x.item()),
            narration: value.narration().map(|x| *x.item()),
            postings: value.postings().map(|x| x.into()).collect::<Vec<_>>(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Price<'a>> for PriceDct<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Price<'a>) -> Self {
        PriceDct {
            cur: value.currency().item().into(),
            price: Price {
                per_unit: value.amount().number().value(),
                total: None,
                cur: value.amount().currency().item().into(),
            },
        }
    }
}

impl<'a, 'b> From<&'b parser::Balance<'a>> for Balance<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Balance<'a>) -> Self {
        Balance {
            acc: value.account().item().into(),
            units: value.atol().amount().number().value(),
            cur: value.atol().amount().currency().item().into(),
            tolerance: value.atol().tolerance().map(|x| *x.item()),
        }
    }
}

impl<'a, 'b> From<&'b parser::Open<'a>> for Open<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Open<'a>) -> Self {
        let currencies = (value.currencies().count() > 0).then(|| {
            value
                .currencies()
                .map(|cur| cur.item().into())
                .collect::<HashSet<_>>()
        });
        Open {
            acc: value.account().item().into(),
            currencies,
            booking: value.booking().map(|booking| booking.item().into()),
        }
    }
}

impl<'a, 'b> From<&'b parser::Close<'a>> for Close<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Close<'a>) -> Self {
        Close {
            acc: (value.account().item()).into(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Commodity<'a>> for Commodity<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Commodity<'a>) -> Self {
        Commodity {
            cur: value.currency().item().into(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Pad<'a>> for Pad<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Pad<'a>) -> Self {
        Pad {
            acc: value.account().item().into(),
            source: value.source().item().into(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Document<'a>> for Document<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Document<'a>) -> Self {
        Document {
            acc: value.account().item().into(),
            path: value.path().item(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Note<'a>> for Note<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Note<'a>) -> Self {
        Note {
            acc: value.account().item().into(),
            comment: value.comment().item(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Event<'a>> for Event<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Event<'a>) -> Self {
        Event {
            type_: value.event_type().item(),
            description: value.description().item(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Query<'a>> for Query<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Query<'a>) -> Self {
        Query {
            name: value.name().item(),
            content: value.content().item(),
        }
    }
}

impl<'a, 'b> From<&'b parser::Custom<'a>> for Custom<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Custom<'a>) -> Self {
        Custom {
            type_: value.type_().item(),
            // TODO custom meta values
        }
    }
}

pub(crate) fn from_flag(flag: parser::Flag) -> Cow<'static, str> {
    use beancount_parser_lima::Flag::*;

    match flag {
        Asterisk => Cow::Borrowed("*"),
        Exclamation => Cow::Borrowed("!"),
        Ampersand => Cow::Borrowed("&"),
        Hash => Cow::Borrowed("#"),
        Question => Cow::Borrowed("?"),
        Percent => Cow::Borrowed("%"),
        Letter(_) => Cow::Owned(flag.to_string()),
    }
}

impl<'a, 'b> From<&'b parser::Spanned<parser::Posting<'a>>> for PostingSpec<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::Spanned<parser::Posting<'a>>) -> Self {
        PostingSpec {
            span: value.into(),
            flag: value.flag().map(|x| from_flag(*x.item())),
            acc: value.account().item().into(),
            units: value.amount().map(|x| x.item().value()),
            cur: value.currency().map(|x| x.item().into()),
            cost_spec: value.cost_spec().map(|x| x.item().into()),
            price_spec: value.price_annotation().map(|x| x.item().into()),
            tags: from_tags(value.metadata().tags()),
            links: from_links(value.metadata().links()),
            metadata: from_key_values(value.metadata().key_values()),
        }
    }
}

impl<'a, 'b> From<&'b parser::CostSpec<'a>> for CostSpec<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::CostSpec<'a>) -> Self {
        CostSpec {
            per_unit: value.per_unit().map(|x| x.item().value()),
            total: value.total().map(|x| x.item().value()),
            cur: value.currency().map(|x| x.item().into()),
            date: value.date().map(|x| x.item()).copied(),
            label: value.label().map(|x| *x.item()),
            merge: value.merge(),
        }
    }
}

impl<'a, 'b> From<&'b parser::PriceSpec<'a>> for PriceSpec<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::PriceSpec<'a>) -> Self {
        use beancount_parser_lima::PriceSpec::*;
        use beancount_parser_lima::ScopedExprValue::*;

        match value {
            Unspecified => PriceSpec {
                per_unit: None,
                total: None,
                cur: None,
            },
            BareCurrency(cur) => PriceSpec {
                per_unit: None,
                total: None,
                cur: Some(cur.into()),
            },
            BareAmount(PerUnit(expr)) => PriceSpec {
                per_unit: Some(expr.value()),
                total: None,
                cur: None,
            },
            BareAmount(Total(expr)) => PriceSpec {
                per_unit: None,
                total: Some(expr.value()),
                cur: None,
            },
            CurrencyAmount(PerUnit(expr), cur) => PriceSpec {
                per_unit: Some(expr.value()),
                total: None,
                cur: Some(cur.into()),
            },
            CurrencyAmount(Total(expr), cur) => PriceSpec {
                per_unit: None,
                total: Some(expr.value()),
                cur: Some(cur.into()),
            },
        }
    }
}

pub(crate) fn from_tags<'a, 'b>(
    tags: impl ExactSizeIterator<Item = &'b parser::Spanned<parser::Tag<'a>>>,
) -> Option<HashSet<&'a str>>
where
    'b: 'a,
{
    let tags = tags
        .map(|tag: &'b parser::Spanned<parser::Tag<'a>>| tag.item().into())
        .collect::<HashSet<_>>();

    (!tags.is_empty()).then_some(tags)
}

pub(crate) fn from_links<'a, 'b>(
    links: impl ExactSizeIterator<Item = &'b parser::Spanned<parser::Link<'a>>>,
) -> Option<HashSet<&'a str>>
where
    'b: 'a,
{
    let links = links
        .map(|link: &'b parser::Spanned<parser::Link<'a>>| link.item().into())
        .collect::<HashSet<_>>();

    (!links.is_empty()).then_some(links)
}

pub(crate) fn from_key_values<'a, 'b>(
    key_values: impl ExactSizeIterator<
        Item = (
            &'b parser::Spanned<parser::Key<'a>>,
            &'b parser::Spanned<parser::MetaValue<'a>>,
        ),
    >,
) -> Option<HashMap<&'a str, MetaValue<'a>>>
where
    'b: 'a,
{
    let key_values = key_values
        .map(
            |(k, v): (
                &'b parser::Spanned<parser::Key<'a>>,
                &'b parser::Spanned<parser::MetaValue<'a>>,
            )| (k.item().into(), v.item().into()),
        )
        .collect::<HashMap<_, _>>();

    (!key_values.is_empty()).then_some(key_values)
}

impl<'a, 'b> From<&'b parser::MetaValue<'a>> for MetaValue<'a>
where
    'b: 'a,
{
    fn from(value: &'b parser::MetaValue<'a>) -> Self {
        use MetaValue::*;
        use parser::MetaValue as pmv;
        use parser::SimpleValue;

        match value {
            pmv::Simple(SimpleValue::String(x)) => String(x),
            pmv::Simple(SimpleValue::Currency(x)) => Currency(x.into()),
            pmv::Simple(SimpleValue::Account(x)) => Account(x.into()),
            pmv::Simple(SimpleValue::Tag(x)) => Tag(x.into()),
            pmv::Simple(SimpleValue::Link(x)) => Link(x.into()),
            pmv::Simple(SimpleValue::Date(x)) => Date(*x),
            pmv::Simple(SimpleValue::Bool(x)) => Bool(*x),
            pmv::Simple(SimpleValue::Expr(x)) => Number(x.value()),
            pmv::Simple(SimpleValue::Null) => Null,
            pmv::Amount(amount) => Amount(amount.number().value(), amount.currency().item().into()),
        }
    }
}

impl From<parser::Booking> for Booking {
    fn from(value: parser::Booking) -> Self {
        use Booking::*;
        use parser::Booking as parser;

        match value {
            parser::Strict => Strict,
            parser::StrictWithSize => StrictWithSize,
            parser::None => None,
            parser::Average => Average,
            parser::Fifo => Fifo,
            parser::Lifo => Lifo,
            parser::Hifo => Hifo,
        }
    }
}

impl From<&parser::Booking> for Booking {
    fn from(value: &parser::Booking) -> Self {
        Self::from(*value)
    }
}

impl From<ReportKind> for parser::ReportKind {
    fn from(value: ReportKind) -> Self {
        use ReportKind::*;
        use parser::ReportKind as parser;
        match value {
            Error => parser::Error,
            Warning => parser::Warning,
        }
    }
}

impl<'a, 'b, T> From<&'b parser::Spanned<T>> for Span {
    fn from(value: &'b parser::Spanned<T>) -> Self {
        let span = value.span();
        Span {
            source: span.source,
            start: span.start,
            end: span.end,
        }
    }
}

impl From<&Span> for parser::Span {
    fn from(value: &Span) -> Self {
        parser::Span {
            source: value.source,
            start: value.start,
            end: value.end,
        }
    }
}
