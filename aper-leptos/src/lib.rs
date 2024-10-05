use aper::{data_structures::Atom, AperSync};
use leptos::{create_signal, SignalGet, SignalSet};
use serde::{de::DeserializeOwned, Serialize};

pub mod init_tracing;

pub trait Watch<T> {
    fn watch(&self) -> Box<dyn Fn() -> T>;
}

impl<T> Watch<T> for Atom<T>
where
    T: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static,
{
    fn watch(&self) -> Box<dyn Fn() -> T> {
        let (signal, set_signal) = create_signal(self.get());

        let self_clone = self.clone();
        self.listen(move || {
            set_signal.set(self_clone.get());
            true
        });

        Box::new(move || signal.get())
    }
}
