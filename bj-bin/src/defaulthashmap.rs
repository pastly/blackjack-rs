//! A HashMap that returns the configured default value if an existing value does not already exist
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct DefaultHashMap<K, V>
where
    K: Hash + Eq + Copy,
    V: Clone,
{
    hm: HashMap<K, V>,
    def: V,
}

impl<K, V> DefaultHashMap<K, V>
where
    K: Hash + Eq + Copy,
    V: Clone,
{
    pub fn new(def: V) -> Self {
        Self {
            hm: HashMap::new(),
            def,
        }
    }

    /// Check if a value exists at the given key. If so, do nothing. Otherwise insert the default
    /// value.
    fn maybe_insert_default(&mut self, k: &K) {
        if !self.hm.contains_key(k) {
            self.insert(*k, self.def.clone());
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.hm.insert(k, v)
    }

    pub fn get(&mut self, k: &K) -> &V {
        self.maybe_insert_default(k);
        self.hm.get(k).unwrap()
    }

    pub fn get_mut(&mut self, k: &K) -> &mut V {
        self.maybe_insert_default(k);
        self.hm.get_mut(k).unwrap()
    }

    pub fn len(&self) -> usize {
        self.hm.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hm.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::DefaultHashMap as DHM;

    #[test]
    fn basic_func() {
        const DEF: u8 = 1;
        let mut m: DHM<u8, u8> = DHM::new(DEF);
        // starts empty
        assert!(m.is_empty());
        // fetching keys that don't exist return the default
        for k in 0..=255 {
            assert_eq!(*m.get(&k), DEF);
        }
        // size is now equal to the number of default values we had to insert
        assert_eq!(m.len(), 256);
    }

    #[test]
    fn insert() {
        let mut m: DHM<u8, u8> = DHM::new(69);
        // first insert returns None because no existing value
        assert_eq!(m.insert(1, 1), None);
        // futhrer inserts at same key return Some because existing value
        assert_eq!(m.insert(1, 2), Some(1));
        assert_eq!(m.insert(1, 3), Some(2));
        assert_eq!(m.insert(1, 4), Some(3));
    }

    #[test]
    fn serialize_identity_empty() {
        let input: DHM<u8, u8> = DHM::new(69);
        let bytes = serde_json::to_vec(&input).unwrap();
        let output: DHM<u8, u8> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn serialize_identity() {
        let mut input: DHM<u8, u8> = DHM::new(69);
        let _ = input.get(&7);
        input.insert(99, 4);
        let bytes = serde_json::to_vec(&input).unwrap();
        let output: DHM<u8, u8> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(input, output);
    }
}
