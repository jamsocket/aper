use aper_stateroom::AperStateroomService;
use timer_common::Timer;
use stateroom_wasm::stateroom_wasm;

#[stateroom_wasm]
type DropFourService = AperStateroomService<Timer>;
