use aper::{StateMachine, Transition};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Counter(pub i64);

#[derive(Transition, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IncrementCounter;

impl StateMachine for Counter {
    type Transition = IncrementCounter;

    fn apply(&mut self, _event: IncrementCounter) {
        self.0 += 1;
    }
}
