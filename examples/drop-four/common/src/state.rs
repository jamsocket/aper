use aper::{
    data_structures::{atom::Atom, fixed_array::FixedArray},
    Aper, Attach, TreeMapRef,
};
use aper_stateroom::{ClientId, IntentEvent, StateProgram};
use serde::{Deserialize, Serialize};

pub const BOARD_ROWS: u32 = 6;
pub const BOARD_COLS: u32 = 7;
pub const BOARD_SIZE: u32 = BOARD_ROWS * BOARD_COLS;

#[derive(Attach, Clone)]
pub struct Board {
    grid: FixedArray<BOARD_SIZE, Option<PlayerColor>>,
}

const NEEDED_IN_A_ROW: usize = 4;

impl Board {
    pub fn get(&self, row: u32, col: u32) -> Option<PlayerColor> {
        self.grid.get(row * BOARD_COLS + col)
    }

    fn set(&mut self, row: u32, col: u32, value: Option<PlayerColor>) {
        self.grid.set(row * BOARD_COLS + col, value);
    }

    fn clear(&mut self) {
        for i in 0..BOARD_SIZE {
            if self.grid.get(i).is_some() {
                self.grid.set(i, None);
            }
        }
    }

    fn lowest_open_row(&self, col: u32) -> Option<u32> {
        for r in (0..BOARD_ROWS).rev() {
            if self.get(r, col).is_none() {
                return Some(r);
            }
        }

        None
    }

    fn count_same_from(&self, row: i32, col: i32, row_d: i32, col_d: i32) -> usize {
        let val = self.get(row as u32, col as u32);
        if val.is_none() {
            return 0;
        }

        for i in 1..(NEEDED_IN_A_ROW as i32) {
            let rr = row + i * row_d;
            let cc = col + i * col_d;

            if rr < 0
                || rr >= BOARD_ROWS as i32
                || cc < 0
                || cc >= BOARD_COLS as i32
                || self.get(rr as u32, cc as u32) != val
            {
                return i as usize - 1;
            }
        }
        
        NEEDED_IN_A_ROW - 1
    }

    fn count_same_bidirectional(&self, row: i32, col: i32, row_d: i32, col_d: i32) -> usize {
        1 + self.count_same_from(row, col, row_d, col_d)
            + self.count_same_from(row, col, -row_d, -col_d)
    }

    fn check_winner_at(&self, row: i32, col: i32) -> Option<PlayerColor> {
        let Some(player) = self.get(row as u32, col as u32) else {
            return None;
        };

        if self.count_same_bidirectional(row, col, 1, 0) >= NEEDED_IN_A_ROW
            || self.count_same_bidirectional(row, col, 0, 1) >= NEEDED_IN_A_ROW
            || self.count_same_bidirectional(row, col, 1, 1) >= NEEDED_IN_A_ROW
            || self.count_same_bidirectional(row, col, 1, -1) >= NEEDED_IN_A_ROW
        {
            Some(player)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum PlayerColor {
    Brown,
    Teal,
}

impl Default for PlayerColor {
    fn default() -> PlayerColor {
        PlayerColor::Teal
    }
}

impl PlayerColor {
    pub fn name(&self) -> &'static str {
        match self {
            PlayerColor::Teal => "Teal",
            PlayerColor::Brown => "Brown",
        }
    }

    pub fn other(&self) -> PlayerColor {
        match self {
            PlayerColor::Brown => PlayerColor::Teal,
            PlayerColor::Teal => PlayerColor::Brown,
        }
    }
}

#[derive(Attach)]
pub struct PlayerMap {
    pub teal_player: Atom<Option<ClientId>>,
    pub brown_player: Atom<Option<ClientId>>,
}

impl PlayerMap {
    fn id_of_color(&self, color: PlayerColor) -> Option<ClientId> {
        match color {
            PlayerColor::Brown => self.brown_player.get(),
            PlayerColor::Teal => self.teal_player.get(),
        }
    }

    pub fn color_of_player(&self, player_id: ClientId) -> Option<PlayerColor> {
        if self.brown_player.get() == Some(player_id) {
            Some(PlayerColor::Brown)
        } else if self.teal_player.get() == Some(player_id) {
            Some(PlayerColor::Teal)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum PlayState {
    Waiting,
    Playing,
}

impl Default for PlayState {
    fn default() -> PlayState {
        PlayState::Waiting
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum GameTransition {
    Join,
    Drop(usize),
    Reset,
}

#[derive(Attach)]
pub struct DropFourGame {
    play_state: Atom<PlayState>,
    pub next_player: Atom<PlayerColor>,
    pub board: Board,
    pub player_map: PlayerMap,
    pub winner: Atom<Option<PlayerColor>>,
}

impl DropFourGame {
    pub fn state(&self) -> PlayState {
        self.play_state.get()
    }
}

impl Aper for DropFourGame {
    type Intent = IntentEvent<GameTransition>;
    type Error = ();

    fn apply(&mut self, event: &Self::Intent) -> Result<(), ()> {
        println!("Applying event: {:?}", event);

        match event.intent {
            GameTransition::Join => {
                if PlayState::Waiting == self.state() {
                    if self.player_map.teal_player.get().is_none() {
                        self.player_map.teal_player.set(event.client);
                    } else if self.player_map.brown_player.get().is_none() {
                        self.player_map.brown_player.set(event.client);
                        self.play_state.set(PlayState::Playing);
                    }
                }
            }
            GameTransition::Drop(c) => {
                if PlayState::Playing == self.state() {
                    if self.winner.get().is_some() {
                        return Ok(());
                    } // Someone has already won.
                    if self.player_map.id_of_color(self.next_player.get()) != event.client {
                        return Ok(());
                    } // Play out of turn.

                    if let Some(insert_row) = self.board.lowest_open_row(c as u32) {
                        self.board
                            .set(insert_row as u32, c as u32, Some(self.next_player.get()));
                        
                        let winner = self.board.check_winner_at(insert_row as i32, c as i32);
                        
                        self.winner
                            .set(winner);
                        self.next_player.set(self.next_player.get().other());
                    }
                }
            }
            GameTransition::Reset => {
                self.board.clear();
                self.winner.set(None);
                self.next_player.set(PlayerColor::Teal);
            }
        }

        Ok(())
    }
}

impl StateProgram for DropFourGame {
    type T = GameTransition;

    fn new() -> Self {
        let treemapref = TreeMapRef::new();
        Self::attach(treemapref)
    }
}

#[cfg(test)]
mod tests {
    use super::GameTransition::{Drop, Join, Reset};
    use super::PlayState::{Playing, Waiting};
    use super::PlayerColor::{Brown, Teal};
    use aper_stateroom::ClientId;
    use chrono::{TimeZone, Utc};

    use super::*;

    fn expect_disc(game: &DropFourGame, row: usize, col: usize, value: PlayerColor) {
        assert_eq!(Some(value), game.board.get(row as u32, col as u32));
    }

    #[test]
    fn test_game() {
        let mut game = DropFourGame::new();
        let dummy_timestamp = Utc.timestamp_millis_opt(0).unwrap();
        let player1: ClientId = 1.into();
        let player2: ClientId = 2.into();

        assert_eq!(
            Waiting,
            game.state()
        );
        
        game
            .apply(&IntentEvent::new(Some(player1), dummy_timestamp, Join))
            .unwrap();

        assert_eq!(
            Waiting,
            game.state()
        );

        assert_eq!(
            Some(player1),
            game.player_map.teal_player.get(),
        );

        game
            .apply(&IntentEvent::new(Some(player2), dummy_timestamp, Join))
            .unwrap();

        assert_eq!(
            game.state(),
            Playing,
        );
        assert_eq!(
            Some(player2),
            game.player_map.brown_player.get(),
        );
        assert_eq!(
            Teal,
            game.next_player.get(),
        );

        game
            .apply(&IntentEvent::new(
                Some(player1),
                dummy_timestamp,
                Drop(4),
            ))
            .unwrap();

        expect_disc(&game, 5, 4, Teal);
        assert_eq!(
            Brown,
            game.next_player.get(),
        );

        //     v
        // .......
        // .......
        // .......
        // .......
        // .......
        // ....T..

        game
            .apply(&IntentEvent::new(
                Some(player2),
                dummy_timestamp,
                Drop(4),
            ))
            .unwrap();

        assert_eq!(
            Teal,
            game.next_player.get(),
        );
        expect_disc(&game, 4, 4, Brown);

        //     v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ....T..

        game
            .apply(&IntentEvent::new(
                Some(player1),
                dummy_timestamp,
                Drop(3),
            ))
            .unwrap();

        assert_eq!(
            Brown,
            game.next_player.get(),
        );
        expect_disc(&game, 5, 3, Teal);

        //    v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ...TT..

        game
            .apply(&IntentEvent::new(
                Some(player2),
                dummy_timestamp,
                Drop(5),
            ))
            .unwrap();

        assert_eq!(
            Teal,
            game.next_player.get(),
        );
        expect_disc(&game, 5, 5, Brown);

        //      v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ...TTB.

        game
            .apply(&IntentEvent::new(
                Some(player1),
                dummy_timestamp,
                Drop(2),
            ))
            .unwrap();

        assert_eq!(
            Brown,
            game.next_player.get(),
        );
        expect_disc(&game, 5, 2, Teal);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ..TTTB.

        game
            .apply(&IntentEvent::new(
                Some(player2),
                dummy_timestamp,
                Drop(2),
            ))
            .unwrap();

        assert_eq!(
            Teal,
            game.next_player.get(),
        );
        expect_disc(&game, 4, 2, Brown);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ..B.B..
        // ..TTTB.

        game
            .apply(&IntentEvent::new(
                Some(player1),
                dummy_timestamp,
                Drop(1),
            ))
            .unwrap();

        assert_eq!(
            Brown,
            game.next_player.get(),
        );
        expect_disc(&game, 5, 1, Teal);
        assert_eq!(
            Some(Teal),
            game.winner.get(),
        );

        //  v
        // .......
        // .......
        // .......
        // .......
        // ..B.B..
        // .TTTTB.

        game
            .apply(&IntentEvent::new(Some(player1), dummy_timestamp, Reset))
            .unwrap();

        assert_eq!(
            None,
            game.winner.get(),
        );
    }
}
