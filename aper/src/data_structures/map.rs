use crate::{Attach, TreeMapRef};
use serde::{de::DeserializeOwned, Serialize};

pub struct Map<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> {
    inner: TreeMapRef,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Attach for Map<K, V> {
    fn attach(map: TreeMapRef) -> Self {
        Self {
            inner: map,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Map<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner
            .get(&bincode::serialize(key).unwrap())
            .map(|bytes| bincode::deserialize(&bytes).unwrap())
    }

    pub fn set(&mut self, key: &K, value: &V) {
        self.inner.set(
            bincode::serialize(key).unwrap(),
            bincode::serialize(value).unwrap(),
        );
    }

    pub fn delete(&mut self, key: &K) {
        self.inner.delete(bincode::serialize(key).unwrap());
    }
}
