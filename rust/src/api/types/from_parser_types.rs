use beancount_parser_lima as parser;

use super::*;

impl<'a> From<&'a parser::Spanned<parser::Directive<'a>>> for Directive<'a> {
    fn from(value: &'a parser::Spanned<parser::Directive<'a>>) -> Self {
        Directive {
            src: value.into(),
            date: *value.date().item(),
            variant: value.variant().into(),
        }
    }
}

impl<'a> From<&'a parser::DirectiveVariant<'a>> for DirectiveVariant<'a> {
    fn from(value: &'a parser::DirectiveVariant<'a>) -> Self {
        use DirectiveVariant::*;
        use parser::DirectiveVariant as parser;

        match value {
            parser::Transaction(transaction) => Transaction(transaction.into()),
            parser::Price(_price) => todo!(),
            parser::Balance(_balance) => todo!(),
            parser::Open(open) => Open(open.into()),
            parser::Close(_close) => todo!(),
            parser::Commodity(_commodity) => todo!(),
            parser::Pad(_pad) => todo!(),
            parser::Document(_document) => todo!(),
            parser::Note(_note) => todo!(),
            parser::Event(_event) => todo!(),
            parser::Query(_query) => todo!(),
            parser::Custom(_custom) => todo!(),
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
