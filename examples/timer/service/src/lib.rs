use aper_jamsocket::AperJamsocketService;
use timer_common::Timer;
use jamsocket_wasm::jamsocket_wasm;

#[jamsocket_wasm]
type DropFourService = AperJamsocketService<Timer>;
