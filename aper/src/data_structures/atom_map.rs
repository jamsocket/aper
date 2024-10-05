use crate::{AperSync, StoreHandle, StoreIterator};
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

pub struct AtomMap<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> {
    map: StoreHandle,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> Clone for AtomMap<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> AperSync for AtomMap<K, V> {
    fn attach(map: StoreHandle) -> Self {
        Self {
            map,
            _phantom: std::marker::PhantomData,
        }
    }

    fn listen<F: Fn() -> bool + 'static + Send + Sync>(&self, listener: F) {
        self.map.listen(listener)
    }
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> AtomMap<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        self.map
            .get(&Bytes::from(bincode::serialize(key).unwrap()))
            .map(|bytes| bincode::deserialize(&bytes).unwrap())
    }

    pub fn set(&mut self, key: &K, value: &V) {
        self.map.set(
            Bytes::from(bincode::serialize(key).unwrap()),
            Bytes::from(bincode::serialize(value).unwrap()),
        );
    }

    pub fn delete(&mut self, key: &K) {
        self.map
            .delete(Bytes::from(bincode::serialize(key).unwrap()));
    }

    pub fn iter(&self) -> AtomMapIter<K, V> {
        AtomMapIter {
            iter: self.map.iter(),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct AtomMapIter<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> {
    iter: StoreIterator,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Iterator
    for AtomMapIter<K, V>
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.iter.next()?;
        let key = bincode::deserialize(&n.0).unwrap();
        let value = bincode::deserialize(&n.1).unwrap();
        Some((key, value))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn atom_map_iter() {
        let store = crate::Store::default();
        let mut map = AtomMap::<String, String>::attach(store.handle());

        map.set(&"h-insert".to_string(), &"b".to_string());
        map.set(&"a-insert".to_string(), &"a".to_string());
        map.set(&"z-insert".to_string(), &"c".to_string());
        map.set(&"f-insert".to_string(), &"d".to_string());

        let mut iter = map.iter();

        assert_eq!(iter.next(), Some(("a-insert".to_string(), "a".to_string())));
        assert_eq!(iter.next(), Some(("f-insert".to_string(), "d".to_string())));
        assert_eq!(iter.next(), Some(("h-insert".to_string(), "b".to_string())));
        assert_eq!(iter.next(), Some(("z-insert".to_string(), "c".to_string())));
    }
}
