use super::PrefixMapValue;
use crate::Bytes;
use std::collections::btree_map::Iter as BTreeMapIter;
use std::collections::BinaryHeap;

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
        println!("self: {:?}, other: {:?}", self.next_value, other.next_value);
        let result =
            (self.next_value.0, self.layer_rank).cmp(&(other.next_value.0, other.layer_rank));
        println!("result: {:?}", result);
        result
    }
}

pub struct StoreIterator {
    inner: Vec<(Bytes, Bytes)>,
}

impl StoreIterator {
    pub fn new<'a>(iter: impl Iterator<Item = BTreeMapIter<'a, Bytes, PrefixMapValue>>) -> Self {
        let mut inner = Vec::new();

        let mut heap = BinaryHeap::new();
        for (layer_rank, mut iter) in iter.enumerate() {
            let next_value = iter.next_back();
            if let Some((key, value)) = next_value {
                println!("pushing... {:?}", key);
                heap.push(PeekedIterator {
                    next_value: (key, value),
                    layer_rank,
                    rest: iter,
                });
            }
        }

        let mut last_key: Option<Bytes> = None;
        while let Some(mut peeked) = heap.pop() {
            println!("aa {:?}", peeked.next_value.0);

            if last_key.as_ref() == Some(peeked.next_value.0) {
                // we have already encountered this key; skip it.
                continue;
            }

            match peeked.next_value {
                (key, PrefixMapValue::Value(value)) => {
                    inner.push((key.clone(), value.clone()));
                }
                (_key, PrefixMapValue::Deleted) => {}
            }

            last_key = Some(peeked.next_value.0.clone());

            let next_value = peeked.rest.next_back();
            if let Some(next_value) = next_value {
                heap.push(PeekedIterator {
                    next_value,
                    layer_rank: peeked.layer_rank,
                    rest: peeked.rest,
                });
            }
        }

        Self { inner }
    }
}

impl Iterator for StoreIterator {
    type Item = (Bytes, Bytes);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn no_layers() {
        let iter_inner = StoreIterator::new(Vec::new().into_iter());
        let d: Vec<(Bytes, Bytes)> = iter_inner.collect();
        assert_eq!(d, Vec::new());
    }

    #[test]
    fn multiple_empty_layers() {
        let v1 = BTreeMap::new();
        let v2 = BTreeMap::new();
        let iter_inner = StoreIterator::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(Bytes, Bytes)> = iter_inner.collect();
        assert_eq!(d, Vec::new());
    }

    #[test]
    fn one_nonempty_layer() {
        let mut v1 = BTreeMap::new();

        v1.insert(
            Bytes::from("key1"),
            PrefixMapValue::Value(Bytes::from("abc")),
        );

        let iter_inner = StoreIterator::new(vec![v1.iter()].into_iter());
        let d: Vec<(Bytes, Bytes)> = iter_inner.collect();
        assert_eq!(d, vec![(Bytes::from("key1"), Bytes::from("abc")),]);
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

        let iter_inner = StoreIterator::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(Bytes, Bytes)> = iter_inner.collect();
        assert_eq!(
            d,
            vec![
                (Bytes::from("key1"), Bytes::from("abc")),
                (Bytes::from("key3"), Bytes::from("abc")),
                (Bytes::from("key5"), Bytes::from("abc")),
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

        let iter_inner = StoreIterator::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(Bytes, Bytes)> = iter_inner.collect();
        assert_eq!(
            d,
            vec![(
                Bytes::from("overlapping-key"),
                Bytes::from("intended value")
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

        let iter_inner = StoreIterator::new(vec![v1.iter(), v2.iter()].into_iter());
        let d: Vec<(Bytes, Bytes)> = iter_inner.collect();
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

        let iter_inner = StoreIterator::new(vec![v1.iter(), v2.iter(), v3.iter()].into_iter());
        let d: Vec<(Bytes, Bytes)> = iter_inner.collect();
        assert_eq!(
            d,
            vec![(Bytes::from("deleted-key"), Bytes::from("recreated value")),]
        );
    }
}
