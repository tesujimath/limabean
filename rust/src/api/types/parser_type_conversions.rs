use beancount_parser_lima as parser;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use super::{Report, raw::*};

impl<'a> From<&'_ parser::Spanned<parser::Directive<'a>>> for Directive<'a> {
    fn from(value: &'_ parser::Spanned<parser::Directive<'a>>) -> Self {
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

impl<'a> From<&'_ parser::DirectiveVariant<'a>> for DirectiveVariant<'a> {
    fn from(value: &'_ parser::DirectiveVariant<'a>) -> Self {
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

impl<'a> From<&'_ parser::Transaction<'a>> for Transaction<'a> {
    fn from(value: &'_ parser::Transaction<'a>) -> Self {
        Transaction {
            flag: from_flag(*value.flag().item()),
            payee: value.payee().map(|x| *x.item()),
            narration: value.narration().map(|x| *x.item()),
            postings: value.postings().map(|x| x.into()).collect::<Vec<_>>(),
        }
    }
}

impl<'a> From<&'_ parser::Price<'a>> for PriceDct<'a> {
    fn from(value: &'_ parser::Price<'a>) -> Self {
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

impl<'a> From<&'_ parser::Balance<'a>> for Balance<'a> {
    fn from(value: &'_ parser::Balance<'a>) -> Self {
        Balance {
            acc: value.account().item().into(),
            units: value.atol().amount().number().value(),
            cur: value.atol().amount().currency().item().into(),
            tolerance: value.atol().tolerance().map(|x| *x.item()),
        }
    }
}

impl<'a> From<&'_ parser::Open<'a>> for Open<'a> {
    fn from(value: &'_ parser::Open<'a>) -> Self {
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

impl<'a> From<&'_ parser::Close<'a>> for Close<'a> {
    fn from(value: &'_ parser::Close<'a>) -> Self {
        Close {
            acc: (value.account().item()).into(),
        }
    }
}

impl<'a> From<&'_ parser::Commodity<'a>> for Commodity<'a> {
    fn from(value: &'_ parser::Commodity<'a>) -> Self {
        Commodity {
            cur: value.currency().item().into(),
        }
    }
}

impl<'a> From<&'_ parser::Pad<'a>> for Pad<'a> {
    fn from(value: &'_ parser::Pad<'a>) -> Self {
        Pad {
            acc: value.account().item().into(),
            source: value.source().item().into(),
        }
    }
}

impl<'a> From<&'_ parser::Document<'a>> for Document<'a> {
    fn from(value: &'_ parser::Document<'a>) -> Self {
        Document {
            acc: value.account().item().into(),
            path: value.path().item(),
        }
    }
}

impl<'a> From<&'_ parser::Note<'a>> for Note<'a> {
    fn from(value: &'_ parser::Note<'a>) -> Self {
        Note {
            acc: value.account().item().into(),
            comment: value.comment().item(),
        }
    }
}

impl<'a> From<&'_ parser::Event<'a>> for Event<'a> {
    fn from(value: &'_ parser::Event<'a>) -> Self {
        Event {
            type_: value.event_type().item(),
            description: value.description().item(),
        }
    }
}

impl<'a> From<&'_ parser::Query<'a>> for Query<'a> {
    fn from(value: &'_ parser::Query<'a>) -> Self {
        Query {
            name: value.name().item(),
            content: value.content().item(),
        }
    }
}

impl<'a> From<&'_ parser::Custom<'a>> for Custom<'a> {
    fn from(value: &'_ parser::Custom<'a>) -> Self {
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

impl<'a> From<&'_ parser::Spanned<parser::Posting<'a>>> for PostingSpec<'a> {
    fn from(value: &'_ parser::Spanned<parser::Posting<'a>>) -> Self {
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

impl<'a> From<&'_ parser::CostSpec<'a>> for CostSpec<'a> {
    fn from(value: &'_ parser::CostSpec<'a>) -> Self {
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

impl<'a> From<&'_ parser::PriceSpec<'a>> for PriceSpec<'a> {
    fn from(value: &'_ parser::PriceSpec<'a>) -> Self {
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
    'a: 'b,
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
    'a: 'b,
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
    'a: 'b,
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

impl<'a> From<&'_ parser::MetaValue<'a>> for MetaValue<'a> {
    fn from(value: &'_ parser::MetaValue<'a>) -> Self {
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

impl<'a> parser::Report for Report<'a> {
    fn message(&self) -> &str {
        self.message.as_ref()
    }
    fn reason(&self) -> &str {
        self.reason.as_ref()
    }
    fn span(&self) -> parser::Span {
        self.span.into()
    }
    fn contexts(&self) -> impl Iterator<Item = (&str, parser::Span)> {
        self.contexts
            .iter()
            .flatten()
            .map(|(label, span)| (label.as_ref(), span.into()))
    }
    fn related(&self) -> impl Iterator<Item = (&str, parser::Span)> {
        self.related
            .iter()
            .flatten()
            .map(|(label, span)| (label.as_ref(), span.into()))
    }
}

fn from_error_or_warning<'a, K>(
    eow: &'a parser::ErrorOrWarning<K>,
    annotation: Option<Cow<'a, str>>,
) -> Report<'a>
where
    K: parser::ErrorOrWarningKind,
{
    Report {
        message: Cow::Borrowed(eow.message()),
        reason: Cow::Borrowed(eow.reason()),
        span: eow.span().into(),
        contexts: eow.contexts().map(|contexts| {
            contexts
                .map(|(ctx, span)| (Cow::Borrowed(ctx), span.into()))
                .collect::<Vec<_>>()
        }),
        related: eow.related().map(|related| {
            related
                .map(|(rel, span)| (Cow::Borrowed(rel), span.into()))
                .collect::<Vec<_>>()
        }),
        annotation,
    }
}

pub(crate) fn from_errors_or_warnings<'a, K>(
    errors_or_warnings: &'a [parser::ErrorOrWarning<K>],
) -> Vec<Report<'a>>
where
    K: parser::ErrorOrWarningKind,
{
    errors_or_warnings
        .iter()
        .map(|eow| from_error_or_warning(eow, None))
        .collect::<Vec<_>>()
}

pub(crate) fn from_annotated_errors_or_warnings<'a, K>(
    errors_or_warnings: &'a [parser::AnnotatedErrorOrWarning<K>],
) -> Vec<Report<'a>>
where
    K: parser::ErrorOrWarningKind,
{
    errors_or_warnings
        .iter()
        .map(|eow| from_error_or_warning(eow, eow.annotation().map(Cow::Borrowed)))
        .collect::<Vec<_>>()
}

impl<'a> From<parser::SpannedSource<'a>> for SpannedSource<'a> {
    fn from(value: parser::SpannedSource<'a>) -> Self {
        SpannedSource {
            file_name: value.file_name,
            start_line: value.start_line,
            end_line: value.end_line,
            content: value.content,
        }
    }
}

impl<T> From<&parser::Spanned<T>> for Span {
    fn from(value: &parser::Spanned<T>) -> Self {
        let span = value.span();
        Span {
            source: span.source,
            start: span.start,
            end: span.end,
        }
    }
}

impl From<&parser::Span> for Span {
    fn from(value: &parser::Span) -> Self {
        Span {
            source: value.source,
            start: value.start,
            end: value.end,
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

impl From<Span> for parser::Span {
    fn from(value: Span) -> Self {
        parser::Span::from(&value)
    }
}
