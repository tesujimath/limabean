use beancount_parser_lima as parser;
use hashbrown::HashMap;
use std::str::FromStr;

use crate::book::Element;

#[derive(
    PartialEq,
    Eq,
    Hash,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::IntoStaticStr,
    Clone,
    Debug,
)]
pub(crate) enum InternalPlugin {
    #[strum(to_string = "beancount.plugins.auto_accounts")]
    AutoAccounts,
    #[strum(to_string = "beancount.plugins.implicit_prices")]
    ImplicitPrices,
    #[strum(to_string = "limabean.balance_rollup")]
    BalanceRollup,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct Plugins {
    pub(crate) internal: HashMap<InternalPlugin, Option<String>>,
    pub(crate) external: Vec<(String, Option<String>)>,
}

pub(crate) fn collate_plugins<'a>(
    parsed_plugins: &[parser::Plugin<'a>],
) -> Result<Plugins, Vec<parser::Error>> {
    let mut plugin_spans = HashMap::<&'a str, parser::Spanned<Element>>::default();
    let mut internal = HashMap::<InternalPlugin, Option<String>>::default();
    let mut external = Vec::<(String, Option<String>)>::default();

    let mut errors = Vec::default();

    for plugin in parsed_plugins {
        let element = Element::new("plugin", *plugin.module_name().span());
        let module_name = *plugin.module_name().item();
        match plugin_spans.entry(module_name) {
            hashbrown::hash_map::Entry::Occupied(entry) => {
                let previous_element = *entry.get();
                let e = element
                    .error("duplicate plugin")
                    .related_to(&previous_element);
                errors.push(e);
            }
            hashbrown::hash_map::Entry::Vacant(entry) => {
                entry.insert(element);
                let plugin_config = plugin.config().map(|config| config.item().to_string());
                if let Ok(internal_plugin) = InternalPlugin::from_str(module_name) {
                    internal.insert(internal_plugin, plugin_config);
                } else {
                    external.push((plugin.module_name().item().to_string(), plugin_config));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(Plugins { internal, external })
    } else {
        Err(errors)
    }
}
