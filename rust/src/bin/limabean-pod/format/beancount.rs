use beancount_parser_lima as parser;
use std::fmt::{self, Display, Formatter};
use time::Date;

use super::*;
use crate::{book::pad_flag, plugins::InternalPlugin};

pub(crate) fn write_booked_as_beancount<'a, W>(
    directives: &[Directive<'a>],
    _options: &parser::Options<'a>,
    internal_plugins: &hashbrown::HashMap<InternalPlugin, Option<String>>,
    mut out_w: W,
) -> Result<(), crate::Error>
where
    W: std::io::Write + Copy,
{
    for d in directives {
        writeln!(out_w, "{}", DirectiveWithPlugins(d, internal_plugins))?;
    }
    Ok(())
}

impl<'a> Display for Directive<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let no_plugins = hashbrown::HashMap::default();
        write!(f, "{}", DirectiveWithPlugins(self, &no_plugins))
    }
}

impl<'a, 'b> Display for DirectiveWithPlugins<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use crate::book::DirectiveVariant as LDV;
        use parser::DirectiveVariant as PDV;

        let DirectiveWithPlugins(dct, internal_plugins) = self;

        let dct_parsed = dct.parsed.item();
        let date = *dct_parsed.date().item();

        match (dct_parsed.variant(), &dct.loaded) {
            (PDV::Transaction(parsed), LDV::Transaction(loaded)) => {
                loaded.fmt(f, date, parsed, internal_plugins /*, &self.metadata*/)
            }
            (PDV::Pad(_parsed), LDV::Pad(loaded)) => {
                loaded.fmt(f, date, dct_parsed /*, &self.metadata*/)
            }
            _ => writeln!(f, "{}", dct_parsed),
        }
    }
}

// adapted from beancount-parser-lima

impl<'a> Transaction<'a> {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
        date: Date,
        parsed: &parser::Transaction,
        internal_plugins: &hashbrown::HashMap<InternalPlugin, Option<String>>, /*, metadata: &Metadata*/
    ) -> fmt::Result {
        if !self.auto_accounts.is_empty() {
            let mut auto_accounts = self.auto_accounts.iter().collect::<Vec<_>>();
            auto_accounts.sort();

            for account in auto_accounts {
                fmt_open(f, date, account, true)?;
            }
        }

        write!(f, "{} {}", date, parsed.flag())?;

        format(f, parsed.payee(), double_quoted, SPACE, Some(SPACE))?;
        format(f, parsed.narration(), double_quoted, SPACE, Some(SPACE))?;
        // we prefer to show tags and links inline rather then line by line in metadata
        // metadata.fmt_tags_links_inline(f)?;
        // metadata.fmt_keys_values(f)?;
        format(
            f,
            self.postings.iter(),
            plain,
            NEWLINE_INDENT,
            Some(NEWLINE_INDENT),
        )?;
        f.write_str(NEWLINE)?;

        if internal_plugins.contains_key(&InternalPlugin::ImplicitPrices) && !self.prices.is_empty()
        {
            let mut prices = self.prices.iter().collect::<Vec<_>>();
            prices.sort();
            for (cur, price) in &prices {
                fmt_price(f, date, *cur, price, true)?;
            }
        }

        Ok(())
    }
}

fn fmt_open(f: &mut Formatter<'_>, date: Date, account: &str, auto: bool) -> fmt::Result {
    write!(f, "{} open {}", date, account)?;

    if auto {
        write!(f, "{}auto: TRUE{}", NEWLINE_INDENT, DOUBLE_NEWLINE)
    } else {
        f.write_str(DOUBLE_NEWLINE)
    }
}

fn fmt_price(
    f: &mut Formatter<'_>,
    date: Date,
    cur: parser::Currency,
    price: &Price,
    implicit: bool,
) -> fmt::Result {
    write!(
        f,
        "{}{} price {} {} {}",
        NEWLINE, date, cur, price.per_unit, price.currency
    )?;

    if implicit {
        write!(f, "{}implicit: TRUE{}", NEWLINE_INDENT, NEWLINE)
    } else {
        f.write_str(NEWLINE)
    }
}

impl<'a> Pad<'a> {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
        date: Date,
        parsed: &parser::Directive, /*, metadata: &Metadata*/
    ) -> fmt::Result {
        writeln!(f, "{}\n", parsed)?;
        write!(f, "{} {}", date, pad_flag())?;
        format(
            f,
            self.postings.iter(),
            plain,
            NEWLINE_INDENT,
            Some(NEWLINE_INDENT),
        )?;
        f.write_str(NEWLINE)
    }
}

impl<'a> Display for Posting<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        simple_format(f, self.flag, None)?;

        write!(
            f,
            "{}{} {} {}",
            if self.flag.is_some() { SPACE } else { EMPTY },
            &self.account,
            &self.units,
            &self.currency
        )?;

        simple_format(f, &self.cost, Some(SPACE))?;
        simple_format(f, &self.price, Some(SPACE))?;
        // self.metadata.fmt(f)

        Ok(())
    }
}
