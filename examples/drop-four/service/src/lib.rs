use aper_stateroom::AperStateroomService;
pub use drop_four_common::{
    Board, DropFourGame, GameTransition, PlayState, PlayerColor, BOARD_COLS, BOARD_ROWS,
};
use stateroom_wasm::stateroom_wasm;

#[stateroom_wasm]
type DropFourService = AperStateroomService<DropFourGame>;
