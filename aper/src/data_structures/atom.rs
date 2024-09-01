use crate::{AperSync, Store, StoreHandle};
use serde::{de::DeserializeOwned, Serialize};

pub struct Atom<T: Serialize + DeserializeOwned + Default> {
    map: StoreHandle,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Default> AperSync for Atom<T> {
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

impl<T: Serialize + DeserializeOwned + Default> Atom<T> {
    pub fn get(&self) -> T {
        self.map
            .get(&vec![])
            .map(|bytes| bincode::deserialize(&bytes).unwrap())
            .unwrap_or_default()
    }

    pub fn set(&mut self, value: T) {
        self.map.set(vec![], bincode::serialize(&value).unwrap());
    }
}
