use aper_jamsocket::{AperJamsocketService, StateMachineContainerProgram};
use counter_common::Counter;
use jamsocket_wasm::prelude::jamsocket_wasm;

#[jamsocket_wasm]
type DropFourService = AperJamsocketService<StateMachineContainerProgram<Counter>>;
