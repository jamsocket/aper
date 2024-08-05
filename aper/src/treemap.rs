use crate::{Bytes, Mutation};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct TreeMapLayer {
    /// Parent layers are read-only.
    parent: Option<Arc<TreeMapLayer>>,
    /// Map of prefix -> (key -> value)
    maps: Mutex<BTreeMap<Vec<Bytes>, Arc<Mutex<BTreeMap<Bytes, Option<Bytes>>>>>>,
}

impl TreeMapLayer {
    pub fn get(&self, prefix: &Vec<Bytes>, key: &Bytes) -> Option<Bytes> {
        let mut layer = self;

        loop {
            if let Some(map) = layer.maps.lock().unwrap().get(prefix) {
                if let Some(value) = map.lock().unwrap().get(key) {
                    return value.clone();
                }
            }

            if let Some(parent) = &layer.parent {
                layer = parent;
            } else {
                return None;
            }
        }
    }

    fn push_overlay(self: &Arc<Self>) -> Self {
        let parent = Some(self.clone());
        let maps = Mutex::new(BTreeMap::new());

        TreeMapLayer { parent, maps }
    }

    pub fn combine(&self, other: &Self) {
        let mut maps_borrow = self.maps.lock().unwrap();
        let other_maps = other.maps.lock().unwrap();

        for (prefix, other_map) in other_maps.iter() {
            let mut map = maps_borrow
                .entry(prefix.clone())
                .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
                .lock()
                .unwrap();

            for (key, value) in other_map.lock().unwrap().iter() {
                map.insert(key.clone(), value.clone());
            }
        }
    }

    pub fn collect(&self) -> BTreeMap<Vec<Bytes>, BTreeMap<Bytes, Bytes>> {
        let mut result = if let Some(parent) = &self.parent {
            parent.collect()
        } else {
            BTreeMap::new()
        };

        for (prefix, map) in self.maps.lock().unwrap().iter() {
            let prefix_map = result.entry(prefix.clone()).or_insert_with(BTreeMap::new);

            for (key, value) in map.lock().unwrap().iter() {
                if let Some(value) = value {
                    prefix_map.insert(key.clone(), value.clone());
                } else {
                    prefix_map.remove(key);
                }
            }
        }

        result
    }

    pub fn into_mutations(&self) -> Vec<Mutation> {
        let mut mutations = vec![];

        for (prefix, map) in self.maps.lock().unwrap().iter() {
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
}

#[derive(Clone, Debug)]
pub struct TreeMapRef {
    map: Arc<TreeMapLayer>,
    prefix: Vec<Bytes>,
    reference: Arc<Mutex<BTreeMap<Bytes, Option<Bytes>>>>,
}

impl Default for TreeMapRef {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeMapRef {
    pub fn push_overlay(&self) -> Self {
        let map = self.map.push_overlay();

        let reference = {
            let mut maps_borrow = map.maps.lock().unwrap();
            maps_borrow
                .entry(self.prefix.clone())
                .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
                .clone()
        };

        Self {
            map: Arc::new(map),
            prefix: self.prefix.clone(),
            reference,
        }
    }

    pub fn mutate(&self, mutations: &Vec<Mutation>) {
        for mutation in mutations {
            let mut map_lock = self.map.maps.lock().unwrap();
            let mut map = map_lock
                .entry(mutation.prefix.clone())
                .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
                .lock()
                .unwrap();

            for (key, value) in &mutation.entries {
                match value {
                    Some(value) => map.insert(key.clone(), Some(value.clone())),
                    None => map.insert(key.clone(), None),
                };
            }
        }
    }

    pub fn into_mutations(self) -> Vec<Mutation> {
        self.map.into_mutations()
    }

    pub fn combine(&self, other: &Self) {
        self.map.combine(&other.map);
    }

    pub fn child(&self, name: &[u8]) -> Self {
        let mut prefix = self.prefix.clone();
        prefix.push(name.to_vec());

        let mut map_borrow = self.map.maps.lock().unwrap();
        let map = map_borrow
            .entry(prefix.clone())
            .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
            .clone();

        Self {
            map: self.map.clone(),
            prefix,
            reference: map,
        }
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        self.map.get(&self.prefix, key)
    }

    pub fn set(&mut self, key: Bytes, value: Bytes) {
        self.reference.lock().unwrap().insert(key, Some(value));
    }

    pub fn delete(&mut self, key: Bytes) {
        self.reference.lock().unwrap().insert(key, None);
    }

    pub fn len(&self) -> usize {
        self.reference.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.reference.lock().unwrap().is_empty()
    }

    pub fn new() -> Self {
        let root = Arc::new(Mutex::new(BTreeMap::new()));
        let mut maps = BTreeMap::new();
        maps.insert(vec![], root.clone());

        let maps = Mutex::new(maps);

        Self {
            map: Arc::new(TreeMapLayer { parent: None, maps }),
            prefix: vec![],
            reference: root,
        }
    }

    pub fn collect(&self) -> BTreeMap<Vec<Bytes>, BTreeMap<Bytes, Bytes>> {
        self.map.collect()
    }
}
