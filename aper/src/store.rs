use crate::{listener::ListenerMap, Bytes, Mutation};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct StoreLayer {
    /// Map of prefix to direct children at that prefix.
    layer: BTreeMap<Vec<Bytes>, BTreeMap<Bytes, Option<Bytes>>>,
    /// A set of prefixes that have been modified in this layer.
    dirty: HashSet<Vec<Bytes>>,
}

impl StoreLayer {
    /// Return a list of prefixes in this layer.
    pub fn prefixes(&self) -> Vec<Vec<Bytes>> {
        self.layer
            .iter()
            .filter_map(|(k, v)| if v.is_empty() { None } else { Some(k.clone()) })
            .collect()
    }
}

pub struct StoreInner {
    layers: Vec<StoreLayer>,
    listeners: ListenerMap,
}

impl Default for StoreInner {
    fn default() -> Self {
        Self {
            layers: vec![StoreLayer::default()],
            listeners: ListenerMap::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct Store {
    inner: Arc<Mutex<StoreInner>>,
}

impl Store {
    pub fn push_overlay(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.layers.push(StoreLayer::default());
    }

    pub fn pop_overlay(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.layers.pop();

        if inner.layers.is_empty() {
            tracing::error!("popped last overlay");
        }
    }

    pub fn notify_dirty(&self) {
        let mut dirty_prefixes = HashSet::new();
        let mut inner = self.inner.lock().unwrap();

        for layer in inner.layers.iter_mut() {
            let new_prefixes = std::mem::take(&mut layer.dirty);
            dirty_prefixes.extend(new_prefixes.into_iter());
        }

        for prefix in dirty_prefixes.iter() {
            inner.listeners.alert(prefix);
        }
    }

    pub fn top_layer_mutations(&self) -> Vec<Mutation> {
        let mut inner = self.inner.lock().unwrap();
        let top_layer = inner.layers.last().unwrap();

        let mut mutations = vec![];

        for (prefix, map) in top_layer.layer.iter() {
            let mut entries = vec![];

            for (key, value) in map.iter() {
                entries.push((key.clone(), value.clone()));
            }

            if entries.is_empty() {
                continue;
            }

            mutations.push(Mutation {
                prefix: prefix.clone(),
                entries,
            });
        }

        mutations
    }

    pub fn alert(&self, prefix: &Vec<Bytes>) {
        let mut inner = self.inner.lock().unwrap();
        inner.listeners.alert(prefix);
    }

    pub fn combine_down(&self) {
        let mut inner = self.inner.lock().unwrap();

        let Some(top_layer) = inner.layers.pop() else {
            return;
        };

        // Combine the top layer with the next layer.
        let Some(next_layer) = inner.layers.last_mut() else {
            return;
        };

        for (prefix, map) in top_layer.layer.iter() {
            let mut next_map = next_layer
                .layer
                .entry(prefix.clone())
                .or_insert_with(BTreeMap::new);

            for (key, value) in map.iter() {
                next_map.insert(key.clone(), value.clone());
            }
        }

        next_layer.dirty.extend(top_layer.dirty);
    }

    pub fn get(&self, prefix: &Vec<Bytes>, key: &Bytes) -> Option<Bytes> {
        let inner = self.inner.lock().unwrap();

        for layer in inner.layers.iter().rev() {
            if let Some(map) = layer.layer.get(prefix) {
                if let Some(value) = map.get(key) {
                    return value.clone();
                }
            }
        }

        None
    }

    pub fn mutate(&self, mutations: &[Mutation]) {
        let mut inner = self.inner.lock().unwrap();
        let top_layer = inner.layers.last_mut().unwrap();

        for mutation in mutations.iter() {
            let mut map = top_layer.layer.entry(mutation.prefix.clone()).or_default();

            for (key, value) in mutation.entries.iter() {
                map.insert(key.clone(), value.clone());
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
        let mut inner = self.map.inner.lock().unwrap();
        inner.listeners.listen(self.prefix.clone(), listener);
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        self.map.get(&self.prefix, key)
    }

    pub fn set(&mut self, key: Bytes, value: Bytes) {
        // set the value in the top layer.

        let mut inner = self.map.inner.lock().unwrap();
        let mut top_layer = inner.layers.last_mut().unwrap();

        let mut map = top_layer.layer.entry(self.prefix.clone()).or_default();

        top_layer.dirty.insert(self.prefix.clone());

        map.insert(key, Some(value));
    }

    pub fn delete(&mut self, key: Bytes) {
        // set the value in the top layer.

        let mut inner = self.map.inner.lock().unwrap();
        let mut top_layer = inner.layers.last_mut().unwrap();

        let mut map = top_layer.layer.entry(self.prefix.clone()).or_default();

        top_layer.dirty.insert(self.prefix.clone());

        map.insert(key, None);
    }

    pub fn child(&self, path_part: &[u8]) -> Self {
        let mut prefix = self.prefix.clone();
        prefix.push(path_part.to_vec());
        Self {
            map: self.map.clone(),
            prefix,
        }
    }
}
