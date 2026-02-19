use beancount_parser_lima as parser;

use crate::book::Element;

#[derive(Clone, Default, Debug)]
pub(crate) struct InternalPlugins {
    // OG Beancount
    pub(crate) auto_accounts: bool,
    pub(crate) implicit_prices: bool,

    // limabean specific
    pub(crate) balance_rollup: bool, // whether balance directives apply to the rollup of all subaccounts

    pub(crate) unknown: Vec<parser::Spanned<Element>>,
}

impl<'a> FromIterator<&'a parser::Plugin<'a>> for InternalPlugins {
    fn from_iter<T: IntoIterator<Item = &'a parser::Plugin<'a>>>(iter: T) -> Self {
        let mut internal_plugins = Self::default();
        for plugin in iter {
            match *plugin.module_name().item() {
                "beancount.plugins.auto_accounts" => {
                    internal_plugins.auto_accounts = true;
                }

                "beancount.plugins.implicit_prices" => {
                    internal_plugins.implicit_prices = true;
                }

                "limabean.balance_rollup" => {
                    internal_plugins.balance_rollup = true;
                }
                _ => internal_plugins
                    .unknown
                    .push(Element::new("plugin", *plugin.module_name().span())),
            }
        }
        internal_plugins
    }
}
