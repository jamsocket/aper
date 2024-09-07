use aper_stateroom::AperStateroomService;
use counter_common::Counter;
use stateroom_wasm::stateroom_wasm;

#[stateroom_wasm]
type CounterService = AperStateroomService<Counter>;
