use super::{core::StoreLayer, PrefixMap, PrefixMapValue};
use crate::Bytes;
use self_cell::self_cell;
use std::collections::btree_map::Iter as BTreeMapIter;
use std::collections::BinaryHeap;
use std::sync::MutexGuard;

struct PeekedIterator<'a> {
    next_value: (&'a Bytes, &'a PrefixMapValue),
    layer_rank: usize,
    rest: BTreeMapIter<'a, Bytes, PrefixMapValue>,
}

impl<'a> PartialEq for PeekedIterator<'a> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<'a> PartialOrd for PeekedIterator<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Eq for PeekedIterator<'a> {}

impl<'a> Ord for PeekedIterator<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // NOTE: we invert the order of next_value here, because we want the smallest value to be at the top of the heap.
        // If two layers have the same key, we break the tie by layer rank.
        (other.next_value.0, self.layer_rank).cmp(&(self.next_value.0, other.layer_rank))
    }
}

struct StoreIteratorInner<'a> {
    iters: BinaryHeap<PeekedIterator<'a>>,
    last_key: Option<Bytes>,
}

impl<'a> StoreIteratorInner<'a> {
    fn new(iters: impl Iterator<Item = BTreeMapIter<'a, Bytes, PrefixMapValue>>) -> Self {
        let mut heap = BinaryHeap::new();

        for (layer_rank, mut iter) in iters.enumerate() {
            if let Some(next) = iter.next() {
                heap.push(PeekedIterator {
                    next_value: next.clone(),
                    layer_rank,
                    rest: iter,
                });
            }
        }

        StoreIteratorInner {
            iters: heap,
            last_key: None,
        }
    }
}

impl<'a> Iterator for StoreIteratorInner<'a> {
    type Item = (&'a Bytes, &'a Bytes);

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = loop {
            let PeekedIterator {
                next_value,
                layer_rank,
                mut rest,
            } = self.iters.pop()?;

            if let Some(next) = rest.next() {
                self.iters.push(PeekedIterator {
                    next_value: next.clone(),
                    layer_rank,
                    rest,
                });
            };

            if self.last_key == Some(next_value.0.clone()) {
                continue;
            }

            self.last_key = Some(next_value.0.clone());

            if let PrefixMapValue::Value(value) = next_value.1 {
                break (next_value.0, value);
            }
        };

        Some(next_value)
    }
}

self_cell! {
    pub struct StoreIterator<'a> {
        owner: MutexGuard<'a, Vec<StoreLayer>>,

        #[covariant]
        dependent: StoreIteratorInner,
    }
}

impl<'a> Iterator for StoreIterator<'a> {
    type Item = (&'a Bytes, &'a Bytes);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> StoreIterator<'a> {
    pub fn from_guard(prefix: Vec<Bytes>, guard: MutexGuard<'a, Vec<StoreLayer>>) -> Self {
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
        let d: Vec<(&Bytes, &Bytes)> = iter_inner.collect();
        assert_eq!(d, Vec::new());
    }

    #[test]
    fn multiple_empty_layers() {
        let v1 = BTreeMap::new();
        let v2 = BTreeMap::new();
        let iter_inner = StoreIteratorInner::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(&Bytes, &Bytes)> = iter_inner.collect();
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
        let d: Vec<(&Bytes, &Bytes)> = iter_inner.collect();
        assert_eq!(d, vec![(&Bytes::from("key1"), &Bytes::from("abc")),]);
    }

    #[test]
    fn two_nonempty_layers_no_overlap() {
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
        let d: Vec<(&Bytes, &Bytes)> = iter_inner.collect();
        assert_eq!(
            d,
            vec![
                (&Bytes::from("key1"), &Bytes::from("abc")),
                (&Bytes::from("key3"), &Bytes::from("abc")),
                (&Bytes::from("key5"), &Bytes::from("abc")),
            ]
        );
    }

    #[test]
    fn two_nonempty_layers_overlap() {
        let mut v1 = BTreeMap::new();

        v1.insert(
            Bytes::from("overlapping-key"),
            PrefixMapValue::Value(Bytes::from("erased value")),
        );

        let mut v2 = BTreeMap::new();

        v2.insert(
            Bytes::from("overlapping-key"),
            PrefixMapValue::Value(Bytes::from("intended value")),
        );

        let iter_inner = StoreIteratorInner::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(&Bytes, &Bytes)> = iter_inner.collect();
        assert_eq!(
            d,
            vec![(
                &Bytes::from("overlapping-key"),
                &Bytes::from("intended value")
            ),]
        );
    }

    #[test]
    fn two_nonempty_layers_deletion() {
        let mut v1 = BTreeMap::new();

        v1.insert(
            Bytes::from("deleted-key"),
            PrefixMapValue::Value(Bytes::from("erased value")),
        );

        let mut v2 = BTreeMap::new();

        v2.insert(Bytes::from("deleted-key"), PrefixMapValue::Deleted);

        let iter_inner = StoreIteratorInner::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(&Bytes, &Bytes)> = iter_inner.collect();
        assert_eq!(d, vec![]);
    }

    #[test]
    fn undeleted_key() {
        let mut v1 = BTreeMap::new();
        v1.insert(
            Bytes::from("deleted-key"),
            PrefixMapValue::Value(Bytes::from("erased value")),
        );

        let mut v2 = BTreeMap::new();
        v2.insert(Bytes::from("deleted-key"), PrefixMapValue::Deleted);

        let mut v3 = BTreeMap::new();
        v3.insert(
            Bytes::from("deleted-key"),
            PrefixMapValue::Value(Bytes::from("recreated value")),
        );

        let iter_inner = StoreIteratorInner::new(vec![v1.iter(), v2.iter(), v3.iter()].into_iter());
        let d: Vec<(&Bytes, &Bytes)> = iter_inner.collect();
        assert_eq!(
            d,
            vec![(&Bytes::from("deleted-key"), &Bytes::from("recreated value")),]
        );
    }
}
