use crate::{listener::ListenerMap, Bytes, Mutation};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct TreeMapLayer(BTreeMap<Vec<Bytes>, Arc<Mutex<BTreeMap<Bytes, Option<Bytes>>>>>);

impl TreeMapLayer {
    pub fn prefixes(&self) -> Vec<Vec<Bytes>> {
        self.0
            .iter()
            .filter_map(|(k, v)| {
                if v.lock().unwrap().is_empty() {
                    None
                } else {
                    Some(k.clone())
                }
            })
            .collect()
    }
}

pub struct TreeMapInner {
    layers: Vec<TreeMapLayer>,
    listeners: ListenerMap,
}

impl Default for TreeMapInner {
    fn default() -> Self {
        Self {
            layers: vec![TreeMapLayer::default()],
            listeners: ListenerMap::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct TreeMap {
    inner: Arc<Mutex<TreeMapInner>>,
}

impl TreeMap {
    pub fn push_overlay(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.layers.push(TreeMapLayer::default());
    }

    pub fn pop_overlay(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.layers.pop();
    }

    pub fn top_layer_mutations(&self) -> Vec<Mutation> {
        let mut inner = self.inner.lock().unwrap();
        let top_layer = inner.layers.last().unwrap();

        let mut mutations = vec![];

        for (prefix, map) in top_layer.0.iter() {
            let mut entries = vec![];

            for (key, value) in map.lock().unwrap().iter() {
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

        for (prefix, map) in top_layer.0.iter() {
            let mut next_map = next_layer
                .0
                .entry(prefix.clone())
                .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
                .lock()
                .unwrap();

            for (key, value) in map.lock().unwrap().iter() {
                next_map.insert(key.clone(), value.clone());
            }
        }
    }

    pub fn get(&self, prefix: &Vec<Bytes>, key: &Bytes) -> Option<Bytes> {
        let inner = self.inner.lock().unwrap();

        for layer in inner.layers.iter().rev() {
            if let Some(map) = layer.0.get(prefix) {
                if let Some(value) = map.lock().unwrap().get(key) {
                    return value.clone();
                }
            }
        }

        None
    }

    pub fn mutate(&self, mutations: &Vec<Mutation>) {
        let mut inner = self.inner.lock().unwrap();
        let top_layer = inner.layers.last_mut().unwrap();

        for mutation in mutations.iter() {
            let mut map = top_layer
                .0
                .entry(mutation.prefix.clone())
                .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
                .lock()
                .unwrap();

            for (key, value) in mutation.entries.iter() {
                map.insert(key.clone(), value.clone());
            }
        }
    }
}

#[derive(Clone)]
pub struct TreeMapRef {
    map: TreeMap,
    prefix: Vec<Bytes>,
}

impl TreeMapRef {
    pub fn listen<F: Fn() -> bool + 'static + Send + Sync>(&self, listener: F) {
        let mut inner = self.map.inner.lock().unwrap();
        inner.listeners.listen(self.prefix.clone(), listener);
    }

    pub fn new_root(map: &TreeMap) -> Self {
        let prefix = vec![];
        let map = map.clone();
        Self { map, prefix }
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        self.map.get(&self.prefix, key)
    }

    pub fn set(&mut self, key: Bytes, value: Bytes) {
        // set the value in the top layer.

        let mut inner = self.map.inner.lock().unwrap();
        let mut top_layer = inner.layers.last_mut().unwrap();

        let mut map = top_layer
            .0
            .entry(self.prefix.clone())
            .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
            .lock()
            .unwrap();

        map.insert(key, Some(value));
    }

    pub fn delete(&mut self, key: Bytes) {
        // set the value in the top layer.

        let mut inner = self.map.inner.lock().unwrap();
        let mut top_layer = inner.layers.last_mut().unwrap();

        let mut map = top_layer
            .0
            .entry(self.prefix.clone())
            .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
            .lock()
            .unwrap();

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
