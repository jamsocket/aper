use aper::{NeverConflict, StateMachine};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Counter {
    value: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CounterTransition {
    Add(i64),
    Subtract(i64),
    Reset,
}

impl Counter {
    pub fn value(&self) -> i64 {
        self.value
    }
}

impl StateMachine for Counter {
    type Transition = CounterTransition;
    type Conflict = NeverConflict;

    fn apply(&self, event: CounterTransition) -> Result<Self, NeverConflict> {
        let mut new_self = self.clone();
        match event {
            CounterTransition::Add(i) => {
                new_self.value += i;
            }
            CounterTransition::Subtract(i) => {
                new_self.value -= i;
            }
            CounterTransition::Reset => {
                new_self.value = 0;
            }
        }

        Ok(new_self)
    }
}
