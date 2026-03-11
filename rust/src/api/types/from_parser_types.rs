use beancount_parser_lima as parser;

use super::*;

impl<'a> From<&'a parser::Spanned<parser::Directive<'a>>> for Directive<'a> {
    fn from(value: &'a parser::Spanned<parser::Directive<'a>>) -> Self {
        Directive {
            src: value.into(),
            date: *value.date().item(),
            variant: value.variant().into(),
            metadata: (!value.metadata().is_empty()).then_some(value.metadata().into()),
        }
    }
}

impl<'a> From<&'a parser::DirectiveVariant<'a>> for DirectiveVariant<'a> {
    fn from(value: &'a parser::DirectiveVariant<'a>) -> Self {
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

impl<'a> From<&'a parser::Transaction<'a>> for Transaction<'a> {
    fn from(value: &'a parser::Transaction<'a>) -> Self {
        Transaction {
            flag: from_flag(*value.flag().item()),
            payee: value.payee().map(|x| x.item().as_ref()),
            narration: value.narration().map(|x| x.item().as_ref()),
            postings: value.postings().map(|x| x.into()).collect::<Vec<_>>(),
        }
    }
}

impl<'a> From<&'a parser::Price<'a>> for PriceDct<'a> {
    fn from(value: &'a parser::Price<'a>) -> Self {
        PriceDct {
            cur: value.currency().item().as_ref(),
            price: Price {
                per_unit: value.amount().number().value(),
                total: None,
                cur: value.amount().currency().item().as_ref(),
            },
        }
    }
}

impl<'a> From<&'a parser::Balance<'a>> for Balance<'a> {
    fn from(value: &'a parser::Balance<'a>) -> Self {
        Balance {
            acc: value.account().item().as_ref(),
            units: value.atol().amount().number().value(),
            cur: value.atol().amount().currency().item().as_ref(),
            tolerance: value.atol().tolerance().map(|x| *x.item()),
        }
    }
}

impl<'a> From<&'a parser::Open<'a>> for Open<'a> {
    fn from(value: &'a parser::Open<'a>) -> Self {
        let currencies = (value.currencies().count() > 0).then(|| {
            value
                .currencies()
                .map(|cur| cur.item().as_ref())
                .collect::<HashSet<_>>()
        });
        Open {
            acc: value.account().item().as_ref(),
            currencies,
            booking: value.booking().map(|booking| (*booking.item()).into()),
        }
    }
}

impl<'a> From<&'a parser::Close<'a>> for Close<'a> {
    fn from(value: &'a parser::Close<'a>) -> Self {
        Close {
            acc: value.account().item().as_ref(),
        }
    }
}

impl<'a> From<&'a parser::Commodity<'a>> for Commodity<'a> {
    fn from(value: &'a parser::Commodity<'a>) -> Self {
        Commodity {
            cur: value.currency().item().as_ref(),
        }
    }
}

impl<'a> From<&'a parser::Pad<'a>> for Pad<'a> {
    fn from(value: &'a parser::Pad<'a>) -> Self {
        Pad {
            acc: value.account().item().as_ref(),
            source: value.source().item().as_ref(),
        }
    }
}

impl<'a> From<&'a parser::Document<'a>> for Document<'a> {
    fn from(value: &'a parser::Document<'a>) -> Self {
        Document {
            acc: value.account().item().as_ref(),
            path: value.path().item(),
        }
    }
}

impl<'a> From<&'a parser::Note<'a>> for Note<'a> {
    fn from(value: &'a parser::Note<'a>) -> Self {
        Note {
            acc: value.account().item().as_ref(),
            comment: value.comment().item(),
        }
    }
}

impl<'a> From<&'a parser::Event<'a>> for Event<'a> {
    fn from(value: &'a parser::Event<'a>) -> Self {
        Event {
            type_: value.event_type().item(),
            description: value.description().item(),
        }
    }
}

impl<'a> From<&'a parser::Query<'a>> for Query<'a> {
    fn from(value: &'a parser::Query<'a>) -> Self {
        Query {
            name: value.name().item(),
            content: value.content().item(),
        }
    }
}

impl<'a> From<&'a parser::Custom<'a>> for Custom<'a> {
    fn from(value: &'a parser::Custom<'a>) -> Self {
        Custom {
            type_: value.type_().item(),
            // TODO custom meta values
        }
    }
}

fn from_flag(flag: parser::Flag) -> Cow<'static, str> {
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

impl<'a> From<&'a parser::Spanned<parser::Posting<'a>>> for PostingSpec<'a> {
    fn from(value: &'a parser::Spanned<parser::Posting<'a>>) -> Self {
        PostingSpec {
            src: value.into(),
            flag: value.flag().map(|x| from_flag(*x.item())),
            acc: value.account().item().as_ref(),
            units: value.amount().map(|x| x.item().value()),
            cur: value.currency().map(|x| x.item().as_ref()),
            cost_spec: value.cost_spec().map(|x| x.item().into()),
            price_spec: value.price_annotation().map(|x| x.item().into()),
            // TODO posting spec metadata
            // pub(crate) metadata: Spanned<Metadata<'a>>>,
        }
    }
}

impl<'a> From<&'a parser::CostSpec<'a>> for CostSpec<'a> {
    fn from(value: &'a parser::CostSpec<'a>) -> Self {
        CostSpec {
            per_unit: value.per_unit().map(|x| x.item().value()),
            total: value.total().map(|x| x.item().value()),
            cur: value.currency().map(|x| x.item().as_ref()),
            date: value.date().map(|x| x.item()).copied(),
            label: value.label().map(|x| x.item().as_ref()),
            merge: value.merge(),
        }
    }
}

impl<'a> From<&'a parser::PriceSpec<'a>> for PriceSpec<'a> {
    fn from(value: &'a parser::PriceSpec<'a>) -> Self {
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
                cur: Some(cur.as_ref()),
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
                cur: Some(cur.as_ref()),
            },
            CurrencyAmount(Total(expr), cur) => PriceSpec {
                per_unit: None,
                total: Some(expr.value()),
                cur: Some(cur.as_ref()),
            },
        }
    }
}

impl<'a> From<&'a parser::Metadata<'a>> for Metadata<'a> {
    fn from(value: &'a parser::Metadata<'a>) -> Self {
        let key_values = value
            .key_values()
            .map(|(k, v)| (k.item().as_ref(), v.item().into()))
            .collect::<HashMap<_, _>>();
        let tags = value
            .tags()
            .map(|tag| tag.item().as_ref())
            .collect::<HashSet<_>>();
        let links = value
            .links()
            .map(|link| link.item().as_ref())
            .collect::<HashSet<_>>();
        Metadata {
            key_values: (!key_values.is_empty()).then_some(key_values),
            tags: (!tags.is_empty()).then_some(tags),
            links: (!links.is_empty()).then_some(links),
        }
    }
}

impl<'a> From<&'a parser::MetaValue<'a>> for MetaValue<'a> {
    fn from(value: &'a parser::MetaValue<'a>) -> Self {
        use MetaValue::*;
        use parser::MetaValue as p;
        use parser::SimpleValue;

        match value {
            p::Simple(SimpleValue::String(x)) => String(x),
            p::Simple(SimpleValue::Currency(x)) => Currency(x.as_ref()),
            p::Simple(SimpleValue::Account(x)) => Account(x.as_ref()),
            p::Simple(SimpleValue::Tag(x)) => Tag(x.as_ref()),
            p::Simple(SimpleValue::Link(x)) => Link(x.as_ref()),
            p::Simple(SimpleValue::Date(x)) => Date(*x),
            p::Simple(SimpleValue::Bool(x)) => Bool(*x),
            p::Simple(SimpleValue::Expr(x)) => Number(x.value()),
            p::Simple(SimpleValue::Null) => Null,
            p::Amount(amount) => Amount(amount.number().value(), amount.currency().item().as_ref()),
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

impl<'a, T> From<&'a parser::Spanned<T>> for Source {
    fn from(value: &'a parser::Spanned<T>) -> Self {
        Source {
            file: value.source_id().into(),
            start: value.span().start,
            end: value.span().end,
        }
    }
}
