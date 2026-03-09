use beancount_parser_lima as parser;

use super::*;

impl<'a> From<&'a parser::Spanned<parser::Directive<'a>>> for Directive<'a> {
    fn from(value: &'a parser::Spanned<parser::Directive<'a>>) -> Self {
        let source = Source {
            file: 0,
            start: 0,
            end: 0,
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
            parser::Transaction(transaction) => todo!(),
            parser::Price(price) => todo!(),
            parser::Balance(balance) => todo!(),
            parser::Open(open) => Open(open.into()),
            parser::Close(close) => todo!(),
            parser::Commodity(commodity) => todo!(),
            parser::Pad(pad) => todo!(),
            parser::Document(document) => todo!(),
            parser::Note(note) => todo!(),
            parser::Event(event) => todo!(),
            parser::Query(query) => todo!(),
            parser::Custom(custom) => todo!(),
        }
    }
}

impl<'a> From<&'a parser::Open<'a>> for Open<'a> {
    fn from(value: &'a parser::Open<'a>) -> Self {
        Open {
            account: value.account().item().as_ref(),
            currencies: value
                .currencies()
                .map(|cur| cur.item().as_ref())
                .collect::<HashSet<_>>(),
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
