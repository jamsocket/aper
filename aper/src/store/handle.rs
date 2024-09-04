use super::{
    core::Store,
    iter::StoreIterator,
    prefix_map::{PrefixMap, PrefixMapValue},
};
use crate::Bytes;
use std::{
    collections::HashSet,
    fmt::{Debug, Formatter},
};

#[derive(Clone)]
pub struct StoreHandle {
    map: Store,
    prefix: Vec<Bytes>,
}

impl StoreHandle {
    pub fn new(map: Store) -> Self {
        Self {
            map,
            prefix: vec![],
        }
    }

    pub fn listen<F: Fn() -> bool + 'static + Send + Sync>(&self, listener: F) {
        let mut listeners = self.map.inner.listeners.lock().unwrap();
        listeners.listen(self.prefix.clone(), listener);
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        self.map.get(&self.prefix, key)
    }

    pub fn set(&mut self, key: Bytes, value: Bytes) {
        // set the value in the top layer.

        let mut layers = self.map.inner.layers.write().unwrap();
        let top_layer = layers.last_mut().unwrap();

        let map = top_layer.layer.entry(self.prefix.clone()).or_default();

        top_layer.dirty.insert(self.prefix.clone());

        map.insert(key, PrefixMapValue::Value(value));
    }

    pub fn delete(&mut self, key: Bytes) {
        // delete the value in the top layer.

        let mut layers = self.map.inner.layers.write().unwrap();
        let top_layer = layers.last_mut().unwrap();

        let map = top_layer.layer.entry(self.prefix.clone()).or_default();

        top_layer.dirty.insert(self.prefix.clone());

        map.insert(key, PrefixMapValue::Deleted);
    }

    pub fn child(&mut self, path_part: Bytes) -> Self {
        let mut prefix = self.prefix.clone();
        prefix.push(path_part);
        self.map.ensure(&prefix);
        Self {
            map: self.map.clone(),
            prefix,
        }
    }

    pub fn delete_child(&mut self, path_part: Bytes) {
        let mut prefix = self.prefix.clone();
        prefix.push(path_part);

        let mut layers = self.map.inner.layers.write().unwrap();

        // When we delete a prefix, we delete not only that prefix but all of the prefixes under it.
        // TODO: This is a bit expensive, in order to make a trade-off that reads are faster. Is the balance optimal?

        let mut prefixes_to_delete = HashSet::new();

        for layer in layers.iter() {
            for (pfx, _) in layer.layer.iter() {
                if pfx.starts_with(&prefix) {
                    prefixes_to_delete.insert(pfx.clone());
                }
            }
        }

        let top_layer = layers.last_mut().unwrap();

        for pfx in prefixes_to_delete.iter() {
            top_layer
                .layer
                .insert(pfx.clone(), PrefixMap::DeletedPrefixMap);
            top_layer.dirty.insert(pfx.clone());
        }
    }

    pub fn iter(&self) -> StoreIterator {
        StoreIterator::from_guard(self.prefix.clone(), self.map.inner.layers.read().unwrap())
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let layers = self.inner.layers.read().unwrap();

        for (i, layer) in layers.iter().enumerate() {
            writeln!(f, "Layer {}", i)?;
            for (prefix, map) in layer.layer.iter() {
                writeln!(f, "  {:?} -> {:?}", prefix, map)?;
            }
        }

        Ok(())
    }
}
