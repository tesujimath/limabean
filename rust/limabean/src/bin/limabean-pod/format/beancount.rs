use beancount_parser_lima as parser;
use std::fmt::{self, Display, Formatter};
use time::Date;

use super::*;
use crate::{book::pad_flag, plugins::InternalPlugins};

pub(crate) fn write_booked_as_beancount<'a, W>(
    directives: &[Directive<'a>],
    _options: &parser::Options<'a>,
    plugins: &InternalPlugins,
    mut out_w: W,
) -> Result<(), crate::Error>
where
    W: std::io::Write + Copy,
{
    for d in directives {
        writeln!(out_w, "{}", DirectiveWithPlugins(d, plugins))?;
    }
    Ok(())
}

impl<'a> Display for Directive<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let no_plugins = InternalPlugins::default();
        write!(f, "{}", DirectiveWithPlugins(self, &no_plugins))
    }
}

impl<'a, 'b> Display for DirectiveWithPlugins<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use crate::book::DirectiveVariant as LDV;
        use parser::DirectiveVariant as PDV;

        let DirectiveWithPlugins(dct, plugins) = self;

        let dct_parsed = dct.parsed.item();
        let date = *dct_parsed.date().item();

        match (dct_parsed.variant(), &dct.loaded) {
            (PDV::Transaction(parsed), LDV::Transaction(loaded)) => {
                loaded.fmt(f, date, parsed, plugins /*, &self.metadata*/)
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
        plugins: &InternalPlugins, /*, metadata: &Metadata*/
    ) -> fmt::Result {
        if !self.auto_accounts.is_empty() {
            let mut auto_accounts = self.auto_accounts.iter().collect::<Vec<_>>();
            auto_accounts.sort();

            // ugh
            for account in auto_accounts {
                write!(
                    f,
                    "{} open {}{}auto: TRUE{}{}",
                    date, account, NEWLINE_INDENT, NEWLINE, NEWLINE
                )?;
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

        if plugins.implicit_prices && !self.prices.is_empty() {
            let mut prices = self.prices.iter().collect::<Vec<_>>();
            prices.sort();
            for (cur, price_cur, price_per_unit) in &prices {
                write!(
                    f,
                    "{}{} price {} {} {}{}implicit: TRUE{}",
                    NEWLINE, date, cur, price_per_unit, price_cur, NEWLINE_INDENT, NEWLINE
                )?;
            }
        }

        Ok(())
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
