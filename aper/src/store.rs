use crate::{
    listener::{self, ListenerMap},
    Bytes, Mutation,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    fmt::{Debug, Formatter},
    sync::{Arc, Mutex},
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum PrefixMapValue {
    Value(Bytes),
    Deleted,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PrefixMap {
    Children(BTreeMap<Bytes, PrefixMapValue>),
    DeletedPrefixMap,
}

impl PrefixMap {
    fn get(&self, key: &Bytes) -> Option<PrefixMapValue> {
        match self {
            PrefixMap::Children(children) => children.get(key).cloned(),
            PrefixMap::DeletedPrefixMap => Some(PrefixMapValue::Deleted),
        }
    }

    fn insert(&mut self, key: Bytes, value: PrefixMapValue) {
        match self {
            PrefixMap::Children(children) => {
                children.insert(key, value);
            }
            PrefixMap::DeletedPrefixMap => {
                if value == PrefixMapValue::Deleted {
                    // the prefix map is deleted, so we don't need to delete the value.
                    return;
                }

                let mut new_children = BTreeMap::new();
                new_children.insert(key, value);
                *self = PrefixMap::Children(new_children);
            }
        }
    }
}

impl Default for PrefixMap {
    fn default() -> Self {
        Self::Children(BTreeMap::new())
    }
}

#[derive(Default)]
pub struct StoreLayer {
    /// Map of prefix to direct children at that prefix.
    layer: BTreeMap<Vec<Bytes>, PrefixMap>,
    /// A set of prefixes that have been modified in this layer.
    dirty: HashSet<Vec<Bytes>>,
}

pub struct StoreInner {
    layers: Mutex<Vec<StoreLayer>>,
    listeners: Mutex<ListenerMap>,
}

impl Default for StoreInner {
    fn default() -> Self {
        Self {
            layers: Mutex::new(vec![StoreLayer::default()]),
            listeners: Mutex::new(ListenerMap::default()),
        }
    }
}

#[derive(Clone, Default)]
pub struct Store {
    inner: Arc<StoreInner>,
}

impl Store {
    pub fn prefixes(&self) -> Vec<Vec<Bytes>> {
        let mut result = std::collections::BTreeSet::new();
        let layers = self.inner.layers.lock().unwrap();

        for layer in layers.iter() {
            for (prefix, value) in layer.layer.iter() {
                match value {
                    PrefixMap::Children(_) => {
                        result.insert(prefix.clone());
                    }
                    PrefixMap::DeletedPrefixMap => {
                        result.remove(prefix);
                    }
                }
            }
        }

        result.into_iter().collect()
    }

    /// Ensure that a prefix exists (even if it is empty) in the store.
    pub fn ensure(&self, prefix: &[Bytes]) {
        let mut layers = self.inner.layers.lock().unwrap();
        let mut layer = layers.last_mut().unwrap();

        layer.layer.entry(prefix.to_vec()).or_default();
    }

    pub fn push_overlay(&self) {
        let mut layers = self.inner.layers.lock().unwrap();
        layers.push(StoreLayer::default());
    }

    pub fn pop_overlay(&self) {
        let mut layers = self.inner.layers.lock().unwrap();
        layers.pop();

        if layers.is_empty() {
            tracing::error!("popped last overlay");
        }
    }

    pub fn notify_dirty(&self) {
        let mut dirty_prefixes = HashSet::new();

        {
            // Collect dirty prefixes in an anonymous scope, so that the lock is released before
            // listeners are alerted.
            let mut layers = self.inner.layers.lock().unwrap();
            for layer in layers.iter_mut() {
                let new_prefixes = std::mem::take(&mut layer.dirty);
                dirty_prefixes.extend(new_prefixes.into_iter());
            }
        }

        let mut listeners = self.inner.listeners.lock().unwrap();
        for prefix in dirty_prefixes.iter() {
            listeners.alert(prefix);
        }
    }

    pub fn top_layer_mutations(&self) -> Vec<Mutation> {
        let mut layers = self.inner.layers.lock().unwrap();
        let top_layer = layers.last().unwrap();

        let mut mutations = vec![];

        for (prefix, entries) in top_layer.layer.iter() {
            mutations.push(Mutation {
                prefix: prefix.clone(),
                entries: entries.clone(),
            });
        }

        mutations
    }

    pub fn alert(&self, prefix: &Vec<Bytes>) {
        let mut listeners = self.inner.listeners.lock().unwrap();
        listeners.alert(prefix);
    }

    pub fn combine_down(&self) {
        let mut layers = self.inner.layers.lock().unwrap();

        let Some(top_layer) = layers.pop() else {
            return;
        };

        // Combine the top layer with the next layer.
        let Some(next_layer) = layers.last_mut() else {
            return;
        };

        for (prefix, map) in top_layer.layer.iter() {
            match map {
                PrefixMap::Children(children) => {
                    let entry = next_layer
                        .layer
                        .entry(prefix.clone())
                        .or_insert_with(|| PrefixMap::Children(BTreeMap::new()));

                    match entry {
                        PrefixMap::Children(next_children) => {
                            for (key, value) in children.iter() {
                                next_children.insert(key.clone(), value.clone());
                            }
                        }
                        PrefixMap::DeletedPrefixMap => {
                            next_layer
                                .layer
                                .insert(prefix.clone(), PrefixMap::Children(children.clone()));
                        }
                    }
                }
                PrefixMap::DeletedPrefixMap => {
                    next_layer
                        .layer
                        .insert(prefix.clone(), PrefixMap::DeletedPrefixMap);
                }
            }
        }

        next_layer.dirty.extend(top_layer.dirty);
    }

    pub fn get(&self, prefix: &Vec<Bytes>, key: &Bytes) -> Option<Bytes> {
        let layers = self.inner.layers.lock().unwrap();

        for layer in layers.iter().rev() {
            if let Some(map) = layer.layer.get(prefix) {
                if let Some(value) = map.get(key) {
                    match value {
                        PrefixMapValue::Value(value) => return Some(value.clone()),
                        PrefixMapValue::Deleted => return None,
                    }
                }
            }
        }

        None
    }

    pub fn mutate(&self, mutations: &[Mutation]) {
        let mut layers = self.inner.layers.lock().unwrap();
        let top_layer = layers.last_mut().unwrap();

        for mutation in mutations.iter() {
            match &mutation.entries {
                PrefixMap::DeletedPrefixMap => {
                    let mut map = top_layer.layer.entry(mutation.prefix.clone()).or_default();
                    *map = PrefixMap::DeletedPrefixMap;
                }
                PrefixMap::Children(children) => {
                    let mut map = top_layer.layer.entry(mutation.prefix.clone()).or_default();

                    for (key, value) in children.iter() {
                        map.insert(key.clone(), value.clone());
                    }
                }
            }

            top_layer.dirty.insert(mutation.prefix.clone());
        }
    }

    pub fn handle(&self) -> StoreHandle {
        StoreHandle {
            map: self.clone(),
            prefix: vec![],
        }
    }
}

#[derive(Clone)]
pub struct StoreHandle {
    map: Store,
    prefix: Vec<Bytes>,
}

impl StoreHandle {
    pub fn listen<F: Fn() -> bool + 'static + Send + Sync>(&self, listener: F) {
        let mut listeners = self.map.inner.listeners.lock().unwrap();
        listeners.listen(self.prefix.clone(), listener);
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        self.map.get(&self.prefix, key)
    }

    pub fn set(&mut self, key: Bytes, value: Bytes) {
        // set the value in the top layer.

        let mut layers = self.map.inner.layers.lock().unwrap();
        let mut top_layer = layers.last_mut().unwrap();

        let mut map = top_layer.layer.entry(self.prefix.clone()).or_default();

        top_layer.dirty.insert(self.prefix.clone());

        map.insert(key, PrefixMapValue::Value(value));
    }

    pub fn delete(&mut self, key: Bytes) {
        // delete the value in the top layer.

        let mut layers = self.map.inner.layers.lock().unwrap();
        let mut top_layer = layers.last_mut().unwrap();

        let mut map = top_layer.layer.entry(self.prefix.clone()).or_default();

        top_layer.dirty.insert(self.prefix.clone());

        map.insert(key, PrefixMapValue::Deleted);
    }

    pub fn child(&mut self, path_part: &[u8]) -> Self {
        let mut prefix = self.prefix.clone();
        prefix.push(path_part.to_vec());
        self.map.ensure(&prefix);
        Self {
            map: self.map.clone(),
            prefix,
        }
    }

    pub fn delete_child(&mut self, path_part: &[u8]) {
        let mut prefix = self.prefix.clone();
        prefix.push(path_part.to_vec());

        let mut layers = self.map.inner.layers.lock().unwrap();
        let mut top_layer = layers.last_mut().unwrap();

        // When we delete a prefix, we delete not only that prefix but all of the prefixes under it.
        // TODO: This is a bit expensive, in order to make a trade-off that reads are faster. Is the balance optimal?

        let mut prefixes_to_delete = HashSet::new();

        for layer in layers.iter() {
            for (pfx, val) in layer.layer.iter() {
                if pfx.starts_with(&prefix) {
                    prefixes_to_delete.insert(pfx.clone());
                }
            }
        }

        let mut top_layer = layers.last_mut().unwrap();

        for pfx in prefixes_to_delete.iter() {
            top_layer
                .layer
                .insert(pfx.clone(), PrefixMap::DeletedPrefixMap);
            top_layer.dirty.insert(pfx.clone());
        }
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let layers = self.inner.layers.lock().unwrap();

        for (i, layer) in layers.iter().enumerate() {
            writeln!(f, "Layer {}", i)?;
            for (prefix, map) in layer.layer.iter() {
                writeln!(f, "  {:?} -> {:?}", prefix, map)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn child_creates_prefix() {
        let store = Store::default();
        let mut handle = store.handle();

        let mut child_handle = handle.child(b"foo");
        let _ = child_handle.child(b"bar");

        assert_eq!(
            store.prefixes(),
            vec![
                vec![b"foo".to_vec()],
                vec![b"foo".to_vec(), b"bar".to_vec()],
            ]
        );
    }

    #[test]
    fn deleting_parent_deletes_child() {
        let store = Store::default();
        let mut handle = store.handle();

        let mut child_handle = handle.child(b"foo");
        let _ = child_handle.child(b"bar");

        handle.delete_child(b"foo".as_slice());

        assert_eq!(store.prefixes(), vec![] as Vec<Vec<Bytes>>);
    }
}
