use super::{core::StoreLayer, PrefixMap, PrefixMapValue};
use crate::Bytes;
use self_cell::self_cell;
use std::collections::BinaryHeap;
use std::{marker::PhantomData, sync::MutexGuard};
use std::collections::btree_map::Iter as BTreeMapIter;

struct PeekedIterator<'a> {
    next_value: (&'a Bytes, &'a PrefixMapValue),
    rest: BTreeMapIter<'a, Bytes, PrefixMapValue>,
}

impl<'a> PartialEq for PeekedIterator<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.next_value == other.next_value
    }
}

impl<'a> PartialOrd for PeekedIterator<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // NOTE: we invert the order here, because we want the smallest value to be at the top of the heap.

        other.next_value.partial_cmp(&self.next_value)
    }
}

impl<'a> Eq for PeekedIterator<'a> {}

impl<'a> Ord for PeekedIterator<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.next_value.cmp(&self.next_value)
    }
}

struct StoreIteratorInner<'a> {
    iters: BinaryHeap<PeekedIterator<'a>>,
}

impl<'a> StoreIteratorInner<'a> {
    fn new(iters: impl Iterator<Item = BTreeMapIter<'a, Bytes, PrefixMapValue>>) -> Self {
        let mut heap = BinaryHeap::new();

        for mut iter in iters {
            if let Some(next) = iter.next() {
                heap.push(PeekedIterator {
                    next_value: next.clone(),
                    rest: iter,
                });
            }
        }

        StoreIteratorInner { iters: heap }
    }
}

impl<'a> Iterator for StoreIteratorInner<'a> {
    type Item = (&'a Bytes, &'a PrefixMapValue);

    fn next(&mut self) -> Option<Self::Item> {
        let PeekedIterator { next_value, mut rest } = self.iters.pop()?;

        if let Some(next) = rest.next() {
            self.iters.push(PeekedIterator {
                next_value: next.clone(),
                rest,
            });
        };

        Some(next_value)
    }
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

            StoreIteratorInner::new(iters.into_iter())
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn no_layers() {
        let iter_inner = StoreIteratorInner::new(Vec::new().into_iter());
        let d: Vec<(&Bytes, &PrefixMapValue)> = iter_inner.collect();
        assert_eq!(d, Vec::new());
    }

    #[test]
    fn multiple_empty_layers() {
        let v1 = BTreeMap::new();
        let v2 = BTreeMap::new();
        let iter_inner = StoreIteratorInner::new(
            vec![v1.iter(), v2.iter()].into_iter(),
        );
        let d: Vec<(&Bytes, &PrefixMapValue)> = iter_inner.collect();
        assert_eq!(d, Vec::new());
    }

    #[test]
    fn one_nonempty_layer() {
        let mut v1 = BTreeMap::new();

        v1.insert(
            Bytes::from("key1"),
            PrefixMapValue::Value(Bytes::from("abc")),
        );

        let iter_inner = StoreIteratorInner::new(vec![v1.iter()].into_iter());
        let d: Vec<(&Bytes, &PrefixMapValue)> = iter_inner.collect();
        assert_eq!(d, vec![
            (&Bytes::from("key1"), &PrefixMapValue::Value(Bytes::from("abc"))),
        ]);
    }

    #[test]
    fn no_nonempty_layers_no_overlap() {
        let mut v1 = BTreeMap::new();

        v1.insert(
            Bytes::from("key1"),
            PrefixMapValue::Value(Bytes::from("abc")),
        );
        v1.insert(
            Bytes::from("key5"),
            PrefixMapValue::Value(Bytes::from("abc")),
        );

        let mut v2 = BTreeMap::new();

        v2.insert(
            Bytes::from("key3"),
            PrefixMapValue::Value(Bytes::from("abc")),
        );

        let iter_inner = StoreIteratorInner::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(&Bytes, &PrefixMapValue)> = iter_inner.collect();
        assert_eq!(d, vec![
            (&Bytes::from("key1"), &PrefixMapValue::Value(Bytes::from("abc"))),
            (&Bytes::from("key3"), &PrefixMapValue::Value(Bytes::from("abc"))),
            (&Bytes::from("key5"), &PrefixMapValue::Value(Bytes::from("abc"))),
        ]);
    }
}
