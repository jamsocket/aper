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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
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
    pub fn get(&self, key: &Bytes) -> Option<PrefixMapValue> {
        match self {
            PrefixMap::Children(children) => children.get(key).cloned(),
            PrefixMap::DeletedPrefixMap => Some(PrefixMapValue::Deleted),
        }
    }

    pub fn insert(&mut self, key: Bytes, value: PrefixMapValue) {
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
