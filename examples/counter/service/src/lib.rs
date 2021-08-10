use aper_jamsocket::AperJamsocketService;
use counter_common::Counter;
use jamsocket_wasm::jamsocket_wasm;
use aper::StateMachineContainerProgram;

#[jamsocket_wasm]
type DropFourService = AperJamsocketService<StateMachineContainerProgram<Counter>>;
