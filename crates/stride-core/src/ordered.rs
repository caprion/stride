//! A map that keeps its insertion order.
//!
//! The contract says the golden files' keys are "in the order given — never
//! sorted: the order is part of the contract". The zone distribution and the
//! verdict counts are built in a meaningful order, so they need a map that
//! keeps it. `BTreeMap` would sort them and `HashMap` would scramble them.

use std::fmt;

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An insertion-ordered string-keyed map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrderedMap<V> {
    entries: Vec<(String, V)>,
}

impl<V> OrderedMap<V> {
    /// An empty map.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append an entry. An existing key is *not* replaced — use
    /// [`OrderedMap::get_mut`] for that.
    pub fn push(&mut self, key: impl Into<String>, value: V) {
        self.entries.push((key.into(), value));
    }

    /// Insert a value, replacing any value already stored under `key`.
    ///
    /// The entry keeps its original position, the way a Python dictionary
    /// does.
    pub fn insert(&mut self, key: impl Into<String>, value: V) {
        let key = key.into();
        match self.entries.iter_mut().find(|(name, _)| name == &key) {
            Some(entry) => entry.1 = value,
            None => self.entries.push((key, value)),
        }
    }

    /// The value stored under `key`.
    pub fn get(&self, key: &str) -> Option<&V> {
        self.entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// The value stored under `key`, mutably.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.entries
            .iter_mut()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// Whether the map holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The entries, in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.entries
            .iter()
            .map(|(name, value)| (name.as_str(), value))
    }
}

impl<V: Serialize> Serialize for OrderedMap<V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.entries.len()))?;
        for (key, value) in &self.entries {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for OrderedMap<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor<V>(std::marker::PhantomData<V>);

        impl<'de, V: Deserialize<'de>> Visitor<'de> for MapVisitor<V> {
            type Value = OrderedMap<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut map = OrderedMap::new();
                while let Some((key, value)) = access.next_entry::<String, V>()? {
                    map.push(key, value);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::OrderedMap;

    #[test]
    fn keeps_insertion_order() {
        let mut map: OrderedMap<f64> = OrderedMap::new();
        map.push("4", 75.0);
        map.push("5", 63.0);
        let keys: Vec<&str> = map.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, vec!["4", "5"]);
        assert_eq!(map.get("5"), Some(&63.0));
    }
}
