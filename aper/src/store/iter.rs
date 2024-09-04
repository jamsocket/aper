use super::{core::StoreLayer, PrefixMap, PrefixMapValue};
use crate::Bytes;
use self_cell::self_cell;
use std::{marker::PhantomData, sync::MutexGuard};

struct StoreIteratorInner<'a> {
    iters: Vec<std::collections::btree_map::Iter<'a, Bytes, PrefixMapValue>>,
}

self_cell! {
    struct StoreIterator<'a> {
        owner: MutexGuard<'a, Vec<StoreLayer>>,

        #[covariant]
        dependent: StoreIteratorInner,
    }
}

impl<'a> StoreIterator<'a> {
    fn from_guard(prefix: Vec<Bytes>, guard: MutexGuard<'a, Vec<StoreLayer>>) -> Self {
        StoreIterator::new(guard, |guard| {
            let mut iters = Vec::new();

            for layer in guard.iter() {
                match layer.layer.get(&prefix) {
                    None => continue,
                    Some(PrefixMap::DeletedPrefixMap) => {
                        iters.clear();
                        continue;
                    }
                    Some(PrefixMap::Children(map)) => {
                        iters.push(map.iter());
                    }
                }
            }

            StoreIteratorInner { iters }
        })
    }
}
