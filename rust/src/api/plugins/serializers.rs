use serde::{Serialize, ser::SerializeMap};

use super::*;

impl Serialize for Plugins {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut internal_names = self.internal.keys().collect::<Vec<_>>();
        internal_names.sort();
        let internal = internal_names
            .into_iter()
            .map(|name| Plugin {
                name: name.into(),
                config: self
                    .internal
                    .get(name)
                    .unwrap()
                    .as_ref()
                    .map(|c| c.as_str()),
            })
            .collect::<Vec<_>>();

        let external = self
            .external
            .iter()
            .map(|(name, config)| Plugin {
                name: name.as_str(),
                config: config.as_ref().map(|config| config.as_str()),
            })
            .collect::<Vec<_>>();

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(INTERNAL_KEY, &internal)?;
        map.serialize_entry(EXTERNAL_KEY, &external)?;
        map.end()
    }
}

/// A plugin, either internal or external
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct Plugin<'n, 'c> {
    name: &'n str,
    config: Option<&'c str>,
}

const EXTERNAL_KEY: &str = "external";
const INTERNAL_KEY: &str = "internal";
