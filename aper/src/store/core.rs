use super::{
    handle::StoreHandle,
    prefix_map::{PrefixMap, PrefixMapValue},
};
use crate::{listener::ListenerMap, Bytes, Mutation};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};

#[derive(Default)]
pub struct StoreLayer {
    /// Map of prefix to direct children at that prefix.
    pub(crate) layer: BTreeMap<Vec<Bytes>, PrefixMap>,
    /// A set of prefixes that have been modified in this layer.
    pub(crate) dirty: HashSet<Vec<Bytes>>,
}

pub struct StoreInner {
    pub(crate) layers: RwLock<Vec<StoreLayer>>,
    pub(crate) listeners: Mutex<ListenerMap>,
}

impl Default for StoreInner {
    fn default() -> Self {
        Self {
            layers: RwLock::new(vec![StoreLayer::default()]),
            listeners: Mutex::new(ListenerMap::default()),
        }
    }
}

#[derive(Clone, Default)]
pub struct Store {
    pub(crate) inner: Arc<StoreInner>,
}

impl Store {
    pub fn prefixes(&self) -> Vec<Vec<Bytes>> {
        let mut result = std::collections::BTreeSet::new();
        let layers = self.inner.layers.read().unwrap();

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
        let mut layers = self.inner.layers.write().unwrap();
        let layer = layers.last_mut().unwrap();

        layer.layer.entry(prefix.to_vec()).or_default();
    }

    pub fn push_overlay(&self) {
        let mut layers = self.inner.layers.write().unwrap();
        layers.push(StoreLayer::default());
    }

    pub fn pop_overlay(&self) {
        let mut layers = self.inner.layers.write().unwrap();
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
            let mut layers = self.inner.layers.write().unwrap();
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
        let layers = self.inner.layers.read().unwrap();
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
        let mut layers = self.inner.layers.write().unwrap();

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
        let layers = self.inner.layers.read().unwrap();

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
        let mut layers = self.inner.layers.write().unwrap();
        let top_layer = layers.last_mut().unwrap();

        for mutation in mutations.iter() {
            match &mutation.entries {
                PrefixMap::DeletedPrefixMap => {
                    let map = top_layer.layer.entry(mutation.prefix.clone()).or_default();
                    *map = PrefixMap::DeletedPrefixMap;
                }
                PrefixMap::Children(children) => {
                    let map = top_layer.layer.entry(mutation.prefix.clone()).or_default();

                    for (key, value) in children.iter() {
                        map.insert(key.clone(), value.clone());
                    }
                }
            }

            top_layer.dirty.insert(mutation.prefix.clone());
        }
    }

    pub fn handle(&self) -> StoreHandle {
        StoreHandle::new(self.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn child_creates_prefix() {
        let store = Store::default();
        let mut handle = store.handle();

        let mut child_handle = handle.child(Bytes::from_static(b"foo"));
        let _ = child_handle.child(Bytes::from_static(b"bar"));

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

        let mut child_handle = handle.child(Bytes::from_static(b"foo"));
        let _ = child_handle.child(Bytes::from_static(b"bar"));

        handle.delete_child(Bytes::from_static(b"foo"));

        assert_eq!(store.prefixes(), vec![] as Vec<Vec<Bytes>>);
    }
}
