use aper::StateMachineContainerProgram;
use aper::{StateMachine, Transition};
use aper_jamsocket::AperJamsocketService;
use jamsocket_wasm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Counter {
    value: i64,
}

#[derive(Transition, Serialize, Deserialize, Debug, Clone, PartialEq)]
enum CounterTransition {
    Add(i64),
    Subtract(i64),
    Reset,
}

impl StateMachine for Counter {
    type Transition = CounterTransition;

    fn apply(&mut self, event: CounterTransition) {
        match event {
            CounterTransition::Add(i) => {
                self.value += i;
            }
            CounterTransition::Subtract(i) => {
                self.value -= i;
            }
            CounterTransition::Reset => {
                self.value = 0;
            }
        }
    }
}

#[jamsocket_wasm]
type CoutnerService = AperJamsocketService<StateMachineContainerProgram<Counter>>;
