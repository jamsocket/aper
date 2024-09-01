use aper::{data_structures::atom::Atom, Aper, AperSync};
use serde::{Deserialize, Serialize};

#[derive(AperSync)]
pub struct Counter {
    pub value: Atom<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CounterIntent {
    Add(i64),
    Subtract(i64),
    Reset,
}

impl Counter {
    pub fn value(&self) -> i64 {
        self.value.get()
    }
}

impl Aper for Counter {
    type Intent = CounterIntent;
    type Error = ();

    fn apply(&mut self, event: &CounterIntent) -> Result<(), ()> {
        let value = self.value.get();

        match event {
            CounterIntent::Add(i) => {
                self.value.set(value + i);
            }
            CounterIntent::Subtract(i) => {
                self.value.set(value - i);
            }
            CounterIntent::Reset => {
                self.value.set(0);
            }
        }

        Ok(())
    }
}
