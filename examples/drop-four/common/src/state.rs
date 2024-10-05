use aper::{
    data_structures::{atom::Atom, fixed_array::FixedArray},
    Aper, AperSync, IntentMetadata,
};
use serde::{Deserialize, Serialize};

pub const BOARD_ROWS: u32 = 6;
pub const BOARD_COLS: u32 = 7;
pub const BOARD_SIZE: u32 = BOARD_ROWS * BOARD_COLS;

#[derive(AperSync, Clone)]
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
        (0..BOARD_ROWS).rev().find(|&r| self.get(r, col).is_none())
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
        let player = self.get(row as u32, col as u32)?;

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum PlayerColor {
    Brown,
    #[default]
    Teal,
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

#[derive(AperSync, Clone)]
pub struct PlayerMap {
    pub teal_player: Atom<Option<u32>>,
    pub brown_player: Atom<Option<u32>>,
}

impl PlayerMap {
    fn id_of_color(&self, color: PlayerColor) -> Option<u32> {
        match color {
            PlayerColor::Brown => self.brown_player.get(),
            PlayerColor::Teal => self.teal_player.get(),
        }
    }

    pub fn color_of_player(&self, player_id: u32) -> Option<PlayerColor> {
        if self.brown_player.get() == Some(player_id) {
            Some(PlayerColor::Brown)
        } else if self.teal_player.get() == Some(player_id) {
            Some(PlayerColor::Teal)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum PlayState {
    #[default]
    Waiting,
    Playing,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum GameTransition {
    Join,
    Drop(usize),
    Reset,
}

#[derive(AperSync, Clone)]
pub struct DropFourGame {
    pub play_state: Atom<PlayState>,
    pub next_player: Atom<PlayerColor>,
    pub board: Board,
    pub player_map: PlayerMap,
    pub winner: Atom<Option<PlayerColor>>,
}

impl Aper for DropFourGame {
    type Intent = GameTransition;
    type Error = ();

    fn apply(&mut self, intent: &Self::Intent, metadata: &IntentMetadata) -> Result<(), ()> {
        match intent {
            GameTransition::Join => {
                if PlayState::Waiting == self.play_state.get() {
                    if self.player_map.teal_player.get().is_none() {
                        self.player_map.teal_player.set(metadata.client);
                    } else if self.player_map.brown_player.get().is_none() {
                        self.player_map.brown_player.set(metadata.client);
                        self.play_state.set(PlayState::Playing);
                    }
                }
            }
            GameTransition::Drop(c) => {
                if PlayState::Playing == self.play_state.get() {
                    if self.winner.get().is_some() {
                        return Ok(());
                    } // Someone has already won.
                    if self.player_map.id_of_color(self.next_player.get()) != metadata.client {
                        return Ok(());
                    } // Play out of turn.

                    if let Some(insert_row) = self.board.lowest_open_row(*c as u32) {
                        self.board
                            .set(insert_row, *c as u32, Some(self.next_player.get()));

                        let winner = self.board.check_winner_at(insert_row as i32, *c as i32);

                        self.winner.set(winner);
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

#[cfg(test)]
mod tests {
    use super::GameTransition::{Drop, Join, Reset};
    use super::PlayState::{Playing, Waiting};
    use super::PlayerColor::{Brown, Teal};
    use aper::Store;
    use chrono::{TimeZone, Utc};

    use super::*;

    impl DropFourGame {
        fn new() -> Self {
            let storeref = Store::default();
            Self::attach(storeref.handle())
        }
    }

    fn expect_disc(game: &DropFourGame, row: usize, col: usize, value: PlayerColor) {
        assert_eq!(Some(value), game.board.get(row as u32, col as u32));
    }

    #[test]
    fn test_game() {
        let mut game = DropFourGame::new();
        let player1 = 1;
        let player2 = 2;

        let player1_meta = IntentMetadata {
            client: Some(player1),
            timestamp: Utc.timestamp_millis_opt(0).unwrap(),
        };
        let player2_meta = IntentMetadata {
            client: Some(player2),
            timestamp: Utc.timestamp_millis_opt(0).unwrap(),
        };

        assert_eq!(Waiting, game.play_state.get());

        game.apply(&Join, &player1_meta).unwrap();

        assert_eq!(Waiting, game.play_state.get());

        assert_eq!(Some(player1), game.player_map.teal_player.get());

        game.apply(&Join, &player2_meta).unwrap();

        assert_eq!(game.play_state.get(), Playing,);
        assert_eq!(Some(player2), game.player_map.brown_player.get());
        assert_eq!(Teal, game.next_player.get());

        game.apply(&Drop(4), &player1_meta).unwrap();

        expect_disc(&game, 5, 4, Teal);
        assert_eq!(Brown, game.next_player.get());

        //     v
        // .......
        // .......
        // .......
        // .......
        // .......
        // ....T..

        game.apply(&Drop(4), &player2_meta).unwrap();

        assert_eq!(Teal, game.next_player.get());
        expect_disc(&game, 4, 4, Brown);

        //     v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ....T..

        game.apply(&Drop(3), &player1_meta).unwrap();

        assert_eq!(Brown, game.next_player.get());
        expect_disc(&game, 5, 3, Teal);

        //    v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ...TT..

        game.apply(&Drop(5), &player2_meta).unwrap();

        assert_eq!(Teal, game.next_player.get());
        expect_disc(&game, 5, 5, Brown);

        //      v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ...TTB.

        game.apply(&Drop(2), &player1_meta).unwrap();

        assert_eq!(Brown, game.next_player.get());
        expect_disc(&game, 5, 2, Teal);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ..TTTB.

        game.apply(&Drop(2), &player2_meta).unwrap();

        assert_eq!(Teal, game.next_player.get());
        expect_disc(&game, 4, 2, Brown);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ..B.B..
        // ..TTTB.

        game.apply(&Drop(1), &player1_meta).unwrap();

        assert_eq!(Brown, game.next_player.get());
        expect_disc(&game, 5, 1, Teal);
        assert_eq!(Some(Teal), game.winner.get());

        //  v
        // .......
        // .......
        // .......
        // .......
        // ..B.B..
        // .TTTTB.

        game.apply(&Reset, &player1_meta).unwrap();

        assert_eq!(None, game.winner.get());
    }
}
