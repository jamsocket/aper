use crate::Bytes;
use std::collections::HashMap;

// A listener returns `false` if it should be removed.
type Listener = Box<dyn Fn() -> bool + Send + Sync>;

#[derive(Default)]
pub struct ListenerMap {
    listeners: HashMap<Vec<Bytes>, Vec<Listener>>,
}

impl ListenerMap {
    pub fn listen<F: Fn() -> bool + 'static + Send + Sync>(
        &mut self,
        prefix: Vec<Bytes>,
        listener: F,
    ) {
        self.listeners
            .entry(prefix)
            .or_default()
            .push(Box::new(listener))
    }

    pub fn alert(&mut self, prefix: &Vec<Bytes>) {
        let Some(listeners) = self.listeners.get_mut(prefix) else {
            return;
        };

        listeners.retain(|listener| (listener)());

        if listeners.is_empty() {
            self.listeners.remove(prefix);
        }
    }
}
