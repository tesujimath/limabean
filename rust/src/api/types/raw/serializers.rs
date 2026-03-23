use serde::{
    Deserialize, Serialize,
    de::Visitor,
    ser::{SerializeMap, SerializeTuple},
};

use crate::api::types::{fmt_iso8601date, parse_iso8601date};

use super::*;

impl<'a> Serialize for MetaValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use MetaValue::*;

        if let Null = self {
            serializer.serialize_none()
        } else {
            let map_len = if matches!(self, Amount(_, _)) { 2 } else { 1 };
            let mut map = serializer.serialize_map(Some(map_len))?;

            match self {
                Amount(units, cur) => {
                    map.serialize_entry(&MetaKey::Units, units)?;
                    map.serialize_entry(&MetaKey::Currency, cur)?;
                }
                String(x) => map.serialize_entry(&MetaKey::String, x)?,
                Currency(x) => map.serialize_entry(&MetaKey::Currency, x)?,
                Account(x) => map.serialize_entry(&MetaKey::Account, x)?,
                Tag(x) => map.serialize_entry(&MetaKey::Tag, x)?,
                Link(x) => map.serialize_entry(&MetaKey::Link, x)?,
                Date(x) => map.serialize_entry(&MetaKey::Date, &fmt_iso8601date(*x))?,
                Bool(x) => map.serialize_entry(&MetaKey::Bool, x)?,
                Number(x) => map.serialize_entry(&MetaKey::Number, x)?,
                Null => panic!("impossible, handled above"),
            }
            map.end()
        }
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for MetaValue<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MetaValueVisitor;

        impl<'de> Visitor<'de> for MetaValueVisitor {
            type Value = MetaValue<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with a single key indicating the MetaValue variant")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let key: MetaKey = map.next_key()?.ok_or_else(|| {
                    serde::de::Error::custom("empty map, expected a MetaValue key")
                })?;

                let value = match key {
                    MetaKey::Units => {
                        let units: Decimal = map.next_value()?;
                        let cur_key: MetaKey = map.next_key()?.ok_or_else(|| {
                            serde::de::Error::custom("expected currency key after units")
                        })?;
                        if cur_key != MetaKey::Currency {
                            return Err(serde::de::Error::custom(
                                "expected currency key after units",
                            ));
                        }
                        let cur: &'de str = map.next_value()?;
                        MetaValue::Amount(units, cur)
                    }
                    MetaKey::Currency => {
                        let cur: &'de str = map.next_value()?;
                        // peek at next key to decide if this is Amount or Currency variant
                        let next_key: Option<MetaKey> = map.next_key()?;
                        match next_key {
                            Some(MetaKey::Units) => {
                                let units: Decimal = map.next_value()?;
                                MetaValue::Amount(units, cur)
                            }
                            None => MetaValue::Currency(cur),
                            Some(other) => {
                                return Err(serde::de::Error::custom(format!(
                                    "unexpected key {:?} after currency",
                                    other
                                )));
                            }
                        }
                    }
                    MetaKey::String => MetaValue::String(map.next_value()?),
                    MetaKey::Account => MetaValue::Account(map.next_value()?),
                    MetaKey::Tag => MetaValue::Tag(map.next_value()?),
                    MetaKey::Link => MetaValue::Link(map.next_value()?),
                    MetaKey::Date => MetaValue::Date(
                        parse_iso8601date(map.next_value()?)
                            .map_err(|e| serde::de::Error::custom(format!("bad date: {}", &e)))?,
                    ),
                    MetaKey::Bool => MetaValue::Bool(map.next_value()?),
                    MetaKey::Number => MetaValue::Number(map.next_value()?),
                    MetaKey::Null => Err(serde::de::Error::custom(
                        "can't have metavalue map with null key",
                    ))?,
                };

                Ok(value)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MetaValue::Null)
            }
        }

        deserializer.deserialize_any(MetaValueVisitor)
    }
}

impl Serialize for Span {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element(&self.source)?;
        seq.serialize_element(&self.start)?;
        seq.serialize_element(&self.end)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Span {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SourceVisitor;

        impl<'de> Visitor<'de> for SourceVisitor {
            type Value = Span;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a tuple of 3 elements: (u32, usize, usize)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let file = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let start = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let end = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                Ok(Span {
                    source: file,
                    start,
                    end,
                })
            }
        }

        deserializer.deserialize_tuple(3, SourceVisitor)
    }
}

// metadata keywords when encoded into HashMap
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
enum MetaKey {
    #[serde(rename = "acc")]
    Account,
    Bool,
    #[serde(rename = "cur")]
    Currency,
    Date,
    Link,
    Null,
    Number,
    String,
    Tag,
    Units,
}
