use crate::{AperSync, StoreHandle, StoreIterator};
use serde::{de::DeserializeOwned, Serialize};

pub struct AtomMap<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> {
    map: StoreHandle,
    _phantom: std::marker::PhantomData<(K, V)>,
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
            .get(&bincode::serialize(key).unwrap())
            .map(|bytes| bincode::deserialize(&bytes).unwrap())
    }

    pub fn set(&mut self, key: &K, value: &V) {
        self.map.set(
            bincode::serialize(key).unwrap(),
            bincode::serialize(value).unwrap(),
        );
    }

    pub fn delete(&mut self, key: &K) {
        self.map.delete(bincode::serialize(key).unwrap());
    }

    pub fn iter(&self) -> AtomMapIter<K, V> {
        AtomMapIter {
            iter: self.map.iter(),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct AtomMapIter<'a, K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> {
    iter: StoreIterator<'a>,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<'a, K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Iterator
    for AtomMapIter<'a, K, V>
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: wrong
        let n = self.iter.next()?;
        let key = bincode::deserialize(&n.0).unwrap();
        let value = bincode::deserialize(&n.1).unwrap();
        Some((key, value))
    }
}
