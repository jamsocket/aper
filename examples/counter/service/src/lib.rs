use aper::{NeverConflict, StateMachine, Transition};
use aper_jamsocket::{
    AperJamsocketService, StateMachineContainerProgram, StateProgram, TransitionEvent,
};
use counter_common::{Counter, CounterTransition};
use jamsocket_wasm::prelude::{jamsocket_wasm, SimpleJamsocketService};

#[jamsocket_wasm]
type DropFourService = AperJamsocketService<StateMachineContainerProgram<Counter>>;
