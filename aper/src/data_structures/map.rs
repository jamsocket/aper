use crate::{AperSync, TreeMapRef};
use serde::{de::DeserializeOwned, Serialize};

pub struct Map<K: Serialize + DeserializeOwned, V: AperSync> {
    map: TreeMapRef,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K: Serialize + DeserializeOwned, V: AperSync> AperSync for Map<K, V> {
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

impl<K: Serialize + DeserializeOwned, V: AperSync> Map<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        let key = bincode::serialize(key).unwrap();
        Some(V::attach(self.map.child(&key)))
    }

    pub fn get_or_create(&self, key: &K) -> V {
        let key = bincode::serialize(key).unwrap();
        V::attach(self.map.child(&key))
    }
}
