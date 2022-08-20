use aper::{NeverConflict, StateMachine};
use aper_stateroom::{ClientId, StateProgram, TransitionEvent};
use serde::{Deserialize, Serialize};

pub const BOARD_ROWS: usize = 6;
pub const BOARD_COLS: usize = 7;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Board(pub [[Option<PlayerColor>; BOARD_COLS]; BOARD_ROWS]);
const NEEDED_IN_A_ROW: usize = 4;

impl Board {
    fn lowest_open_row(&self, col: usize) -> Option<usize> {
        (0..BOARD_ROWS).rev().find(|r| self.0[*r][col].is_none())
    }

    fn count_same_from(&self, row: i32, col: i32, row_d: i32, col_d: i32) -> usize {
        let val = self.0[row as usize][col as usize];

        for i in 1..(NEEDED_IN_A_ROW as i32) {
            let rr = row + i * row_d;
            let cc = col + i * col_d;

            if rr < 0
                || rr >= BOARD_ROWS as i32
                || cc < 0
                || cc >= BOARD_COLS as i32
                || self.0[rr as usize][cc as usize] != val
            {
                return i as usize - 1;
            }
        }
        NEEDED_IN_A_ROW
    }

    fn count_same_bidirectional(&self, row: i32, col: i32, row_d: i32, col_d: i32) -> usize {
        1 + self.count_same_from(row, col, row_d, col_d)
            + self.count_same_from(row, col, -row_d, -col_d)
    }

    fn check_winner_at(&self, row: i32, col: i32) -> Option<PlayerColor> {
        let player = self.0[row as usize][col as usize];
        if self.count_same_bidirectional(row, col, 1, 0) >= NEEDED_IN_A_ROW
            || self.count_same_bidirectional(row, col, 0, 1) >= NEEDED_IN_A_ROW
            || self.count_same_bidirectional(row, col, 1, 1) >= NEEDED_IN_A_ROW
            || self.count_same_bidirectional(row, col, 1, -1) >= NEEDED_IN_A_ROW
        {
            player
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

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub struct PlayerMap {
    pub teal_player: ClientId,
    pub brown_player: ClientId,
}

impl PlayerMap {
    fn id_of_color(&self, color: PlayerColor) -> ClientId {
        match color {
            PlayerColor::Brown => self.brown_player,
            PlayerColor::Teal => self.teal_player,
        }
    }

    pub fn color_of_player(&self, player_id: ClientId) -> Option<PlayerColor> {
        if self.brown_player == player_id {
            Some(PlayerColor::Brown)
        } else if self.teal_player == player_id {
            Some(PlayerColor::Teal)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayState {
    Waiting {
        waiting_player: Option<ClientId>,
    },
    Playing {
        next_player: PlayerColor,
        board: Board,
        player_map: PlayerMap,
        winner: Option<PlayerColor>,
    },
}

impl Default for PlayState {
    fn default() -> PlayState {
        PlayState::Waiting {
            waiting_player: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum GameTransition {
    Join,
    Drop(usize),
    Reset,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct DropFourGame(PlayState);

impl DropFourGame {
    pub fn state(&self) -> &PlayState {
        &self.0
    }
}

impl StateMachine for DropFourGame {
    type Transition = TransitionEvent<GameTransition>;
    type Conflict = NeverConflict;

    fn apply(&self, event: Self::Transition) -> Result<Self, NeverConflict> {
        let mut new_self = self.clone();
        match event.transition {
            GameTransition::Join => {
                if let PlayState::Waiting {
                    waiting_player: Some(waiting_player),
                } = new_self.0
                {
                    let player_map = PlayerMap {
                        teal_player: waiting_player,
                        brown_player: event.player.unwrap(),
                    };

                    new_self.0 = PlayState::Playing {
                        next_player: PlayerColor::Teal,
                        board: Default::default(),
                        player_map,
                        winner: None,
                    }
                } else if let PlayState::Waiting { .. } = self.0 {
                    new_self.0 = PlayState::Waiting {
                        waiting_player: event.player,
                    }
                }
            }
            GameTransition::Drop(c) => {
                if let PlayState::Playing {
                    board,
                    next_player,
                    player_map,
                    winner,
                } = &mut new_self.0
                {
                    if winner.is_some() {
                        return Ok(new_self);
                    } // Someone has already won.
                    if player_map.id_of_color(*next_player) != event.player.unwrap() {
                        return Ok(new_self);
                    } // Play out of turn.

                    if let Some(insert_row) = board.lowest_open_row(c) {
                        board.0[insert_row][c] = Some(*next_player);
                        *winner = board.check_winner_at(insert_row as i32, c as i32);
                        *next_player = next_player.other();
                    }
                }
            }
            GameTransition::Reset => {
                if let PlayState::Playing {
                    winner: Some(winner),
                    player_map,
                    ..
                } = new_self.0
                {
                    new_self.0 = PlayState::Playing {
                        next_player: winner.other(),
                        board: Default::default(),
                        player_map,
                        winner: None,
                    }
                }
            }
        }

        Ok(new_self)
    }
}

impl StateProgram for DropFourGame {
    type T = GameTransition;

    fn new(_: &str) -> Self {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use aper_stateroom::ClientId;

    use chrono::{TimeZone, Utc};

    use super::GameTransition::{Drop, Join, Reset};
    use super::PlayState::{Playing, Waiting};
    use super::PlayerColor::{Brown, Teal};

    use super::*;

    fn expect_disc(game: &DropFourGame, row: usize, col: usize, value: PlayerColor) {
        let board = match &game.0 {
            PlayState::Playing { board, .. } => &board.0,
            _ => panic!("Called .board() on DropFourGame in Waiting state."),
        };

        assert_eq!(Some(value), board[row][col]);
    }

    #[test]
    fn test_game() {
        let mut game = DropFourGame::default();
        let dummy_timestamp = Utc.timestamp_millis(0);
        let player1: ClientId = 1.into();
        let player2: ClientId = 2.into();

        assert_eq!(
            Waiting {
                waiting_player: None
            },
            *game.state()
        );
        game.apply(TransitionEvent::new(Some(player1), dummy_timestamp, Join))
            .unwrap();
        assert_eq!(
            Waiting {
                waiting_player: Some(player1)
            },
            *game.state()
        );

        game.apply(TransitionEvent::new(Some(player2), dummy_timestamp, Join))
            .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                next_player: Teal,
                ..
            }
        ));

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            Drop(4),
        ))
        .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                next_player: Brown,
                ..
            }
        ));
        expect_disc(&game, 5, 4, Teal);

        //     v
        // .......
        // .......
        // .......
        // .......
        // .......
        // ....T..

        game.apply(TransitionEvent::new(
            Some(player2),
            dummy_timestamp,
            Drop(4),
        ))
        .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                next_player: Teal,
                ..
            }
        ));
        expect_disc(&game, 4, 4, Brown);

        //     v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ....T..

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            Drop(3),
        ))
        .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                next_player: Brown,
                ..
            }
        ));
        expect_disc(&game, 5, 3, Teal);

        //    v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ...TT..

        game.apply(TransitionEvent::new(
            Some(player2),
            dummy_timestamp,
            Drop(5),
        ))
        .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                next_player: Teal,
                ..
            }
        ));
        expect_disc(&game, 5, 5, Brown);

        //      v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ...TTB.

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            Drop(2),
        ))
        .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                next_player: Brown,
                ..
            }
        ));
        expect_disc(&game, 5, 2, Teal);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ....B..
        // ..TTTB.

        game.apply(TransitionEvent::new(
            Some(player2),
            dummy_timestamp,
            Drop(2),
        ))
        .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                next_player: Teal,
                ..
            }
        ));
        expect_disc(&game, 4, 2, Brown);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ..B.B..
        // ..TTTB.

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            Drop(1),
        ))
        .unwrap();

        assert!(matches!(
            game.state(),
            Playing {
                winner: Some(Teal),
                ..
            }
        ));
        expect_disc(&game, 5, 1, Teal);

        //  v
        // .......
        // .......
        // .......
        // .......
        // ..B.B..
        // .TTTTB.

        game.apply(TransitionEvent::new(Some(player1), dummy_timestamp, Reset))
            .unwrap();
        assert!(matches!(
            game.state(),
            Playing {
                winner: None,
                next_player: Brown,
                ..
            }
        ));
    }
}
