use aper_stateroom::AperStateroomService;
use stateroom_wasm::stateroom_wasm;
use timer_common::Timer;

#[stateroom_wasm]
type DropFourService = AperStateroomService<Timer>;
