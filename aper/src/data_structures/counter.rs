use crate::{NeverConflict, StateMachine};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Counter {
    value: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CounterTransition {
    Set(i64),
    Increment(i64),
    Decrement(i64),
}

impl Counter {
    pub fn new(value: i64) -> Self {
        Counter { value }
    }

    pub fn increment(by: i64) -> CounterTransition {
        CounterTransition::Increment(by)
    }

    pub fn decrement(by: i64) -> CounterTransition {
        CounterTransition::Decrement(by)
    }

    pub fn set(to: i64) -> CounterTransition {
        CounterTransition::Set(to)
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

impl StateMachine for Counter {
    type Transition = CounterTransition;
    type Conflict = NeverConflict;

    fn apply(&self, event: &CounterTransition) -> Result<Counter, NeverConflict> {
        match event {
            CounterTransition::Set(value) => Ok(Counter { value: *value }),
            CounterTransition::Increment(amount) => Ok(Counter {
                value: self.value + amount,
            }),
            CounterTransition::Decrement(amount) => Ok(Counter {
                value: self.value - amount,
            }),
        }
    }
}
