use aper_stateroom::{AperStateroomService, StateMachineContainerProgram};
use counter_common::Counter;
use stateroom_wasm::prelude::stateroom_wasm;

#[stateroom_wasm]
type DropFourService = AperStateroomService<StateMachineContainerProgram<Counter>>;
