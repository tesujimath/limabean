use beancount_parser_lima as parser;

use super::*;

impl<'a> From<&'a parser::Spanned<parser::Directive<'a>>> for Directive<'a> {
    fn from(value: &'a parser::Spanned<parser::Directive<'a>>) -> Self {
        let source = Source {
            file: value.source_id().into(),
            start: value.span().start,
            end: value.span().end,
        };
        Directive {
            source,
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
            parser::Transaction(_transaction) => todo!(),
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

impl<'a> From<&'a parser::Open<'a>> for Open<'a> {
    fn from(value: &'a parser::Open<'a>) -> Self {
        let currencies = (value.currencies().count() > 0).then(|| {
            value
                .currencies()
                .map(|cur| cur.item().as_ref())
                .collect::<HashSet<_>>()
        });
        Open {
            account: value.account().item().as_ref(),
            currencies,
            booking: value.booking().map(|booking| (*booking.item()).into()),
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
