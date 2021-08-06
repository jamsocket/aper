mod state;

pub use state::{Board, DropFourGame, GameTransition, PlayerColor, PlayState, BOARD_COLS, BOARD_ROWS};
use aper_jamsocket::AperJamsocketService;
use jamsocket_wasm::jamsocket_wasm;

#[jamsocket_wasm]
type DropFourService = AperJamsocketService<DropFourGame>;
