use aper_jamsocket::AperJamsocketService;
pub use drop_four_common::{
    Board, DropFourGame, GameTransition, PlayState, PlayerColor, BOARD_COLS, BOARD_ROWS,
};
use jamsocket_wasm::jamsocket_wasm;

#[jamsocket_wasm]
type DropFourService = AperJamsocketService<DropFourGame>;
