use crate::{Bytes, Mutation};
use std::{cell::RefCell, collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct TreeMapLayer {
    /// Parent layers are read-only.
    parent: Option<Arc<TreeMapLayer>>,
    /// Map of prefix -> (key -> value)
    maps: RefCell<BTreeMap<Vec<Bytes>, Arc<RefCell<BTreeMap<Bytes, Option<Bytes>>>>>>,
}

impl TreeMapLayer {
    pub fn get(&self, prefix: &Vec<Bytes>, key: &Bytes) -> Option<Bytes> {
        let mut layer = self;

        loop {
            if let Some(map) = layer.maps.borrow().get(prefix) {
                if let Some(value) = map.borrow().get(key) {
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
        let maps = RefCell::new(BTreeMap::new());

        TreeMapLayer { parent, maps }
    }

    pub fn combine(&self, other: &Self) {
        let mut maps_borrow = self.maps.borrow_mut();
        let other_maps = other.maps.borrow();

        for (prefix, other_map) in other_maps.iter() {
            let mut map = maps_borrow
                .entry(prefix.clone())
                .or_insert_with(|| Arc::new(RefCell::new(BTreeMap::new())))
                .borrow_mut();

            for (key, value) in other_map.borrow().iter() {
                map.insert(key.clone(), value.clone());
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeMapRef {
    map: Arc<TreeMapLayer>,
    prefix: Vec<Bytes>,
    reference: Arc<RefCell<BTreeMap<Bytes, Option<Bytes>>>>,
}

impl TreeMapRef {
    pub fn push_overlay(&self) -> Self {
        let map = self.map.push_overlay();

        let reference = {
            let mut maps_borrow = map.maps.borrow_mut();
            maps_borrow
                .entry(self.prefix.clone())
                .or_insert_with(|| Arc::new(RefCell::new(BTreeMap::new())))
                .clone()
        };

        Self {
            map: Arc::new(map),
            prefix: self.prefix.clone(),
            reference,
        }
    }

    pub fn mutate(&self, mutations: &Vec<Mutation>) {
        let mut reference = self.reference.borrow_mut();

        for mutation in mutations {
            for (key, value) in &mutation.entries {
                match value {
                    Some(value) => reference.insert(key.clone(), Some(value.clone())),
                    None => reference.insert(key.clone(), None),
                };
            }
        }
    }

    pub fn into_mutations(self) -> Vec<Mutation> {
        let mut mutations = vec![];

        let reference = self.reference.borrow();
        let mut entries = vec![];

        for (key, value) in reference.iter() {
            entries.push((key.clone(), value.clone()));
        }

        mutations.push(Mutation {
            prefix: self.prefix.clone(),
            entries,
        });

        mutations
    }

    pub fn combine(&self, other: &Self) {
        self.map.combine(&other.map);
    }

    pub fn child(&self, name: &[u8]) -> Self {
        let mut prefix = self.prefix.clone();
        prefix.push(name.to_vec());

        let mut map_borrow = self.map.maps.borrow_mut();
        let map = map_borrow
            .entry(prefix.clone())
            .or_insert_with(|| Arc::new(RefCell::new(BTreeMap::new())))
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
        self.reference.borrow_mut().insert(key, Some(value));
    }

    pub fn delete(&mut self, key: Bytes) {
        self.reference.borrow_mut().insert(key, None);
    }

    pub fn len(&self) -> usize {
        self.reference.borrow().len()
    }

    pub fn new() -> Self {
        let root = Arc::new(RefCell::new(BTreeMap::new()));
        let mut maps = BTreeMap::new();
        maps.insert(vec![], root.clone());

        let maps = RefCell::new(maps);

        Self {
            map: Arc::new(TreeMapLayer { parent: None, maps }),
            prefix: vec![],
            reference: root,
        }
    }
}
