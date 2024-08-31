use crate::{AperSync, TreeMapRef};
use serde::{de::DeserializeOwned, Serialize};

pub struct AtomMap<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> {
    map: TreeMapRef,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> AperSync for AtomMap<K, V> {
    fn attach(map: TreeMapRef) -> Self {
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
}
