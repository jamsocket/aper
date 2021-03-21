use aper::{StateMachine, StateProgram, Transition, TransitionEvent};
use serde::{Deserialize, Serialize};

pub const BOARD_ROWS: usize = 6;
pub const BOARD_COLS: usize = 7;
pub type Board = [[Option<Player>; BOARD_COLS]; BOARD_ROWS];
const NEEDED_IN_A_ROW: usize = 4;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Player {
    Brown,
    Teal,
}

impl Player {
    pub fn name(&self) -> &'static str {
        match self {
            Player::Teal => "Teal",
            Player::Brown => "Brown",
        }
    }

    pub fn other(&self) -> Player {
        match self {
            Player::Brown => Player::Teal,
            Player::Teal => Player::Brown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum PlayState {
    NextTurn(Player),
    Winner(Player),
}

impl Default for PlayState {
    fn default() -> PlayState {
        PlayState::NextTurn(Player::Brown)
    }
}

#[derive(Transition, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DropFourGameTransition {
    Drop(usize),
    Reset,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct DropFourGame {
    board: Board,
    state: PlayState,
}

impl StateMachine for DropFourGame {
    type Transition = TransitionEvent<DropFourGameTransition>;

    fn apply(&mut self, event: Self::Transition) {
        match (self.state, event.transition) {
            (PlayState::NextTurn(p), DropFourGameTransition::Drop(c)) => {
                // Find first available row.
                if let Some(insert_row) = self.lowest_open_row(c) {
                    self.board[insert_row][c] = Some(p);

                    self.state =
                        if let Some(winner) = self.check_winner_at(insert_row as i32, c as i32) {
                            PlayState::Winner(winner)
                        } else {
                            PlayState::NextTurn(p.other())
                        };
                }
            }
            (PlayState::Winner(p), DropFourGameTransition::Reset) => {
                self.board = Default::default();
                self.state = PlayState::NextTurn(p.other()); // Losing player goes first.
            }
            _ => {
                // State transition received is incompatible with the current state.
                // TODO: once Aper supports conflicts, this should raise a conflict.
            },
        }
    }
}

impl StateProgram<DropFourGameTransition> for DropFourGame {}

impl DropFourGame {
    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn state(&self) -> PlayState {
        self.state
    }

    pub fn drop(&self, col: usize) -> DropFourGameTransition {
        DropFourGameTransition::Drop(col)
    }

    pub fn reset(&self) -> DropFourGameTransition {
        DropFourGameTransition::Reset
    }

    fn lowest_open_row(&self, col: usize) -> Option<usize> {
        (0..BOARD_ROWS).rev().find(|r| self.board[*r][c].is_none())
    }

    fn count_same_from(&self, row: i32, col: i32, row_d: i32, col_d: i32) -> usize {
        let val = self.board[row as usize][col as usize];

        for i in 1..(NEEDED_IN_A_ROW as i32) {
            let rr = row + i * row_d;
            let cc = col + i * col_d;

            if rr < 0
                || rr >= BOARD_ROWS as i32
                || cc < 0
                || cc >= BOARD_COLS as i32
                || self.board[rr as usize][cc as usize] != val
            {
                return i as usize - 1;
            }
        }
        return NEEDED_IN_A_ROW;
    }

    fn count_same_bidirectional(&self, row: i32, col: i32, row_d: i32, col_d: i32) -> usize {
        1 + self.count_same_from(row, col, row_d, col_d)
            + self.count_same_from(row, col, -row_d, -col_d)
    }

    fn check_winner_at(&self, row: i32, col: i32) -> Option<Player> {
        let player = self.board[row as usize][col as usize];
        if self.count_same_bidirectional(row, col, 1, 0) >= NEEDED_IN_A_ROW {
            player
        } else if self.count_same_bidirectional(row, col, 0, 1) >= NEEDED_IN_A_ROW {
            player
        } else if self.count_same_bidirectional(row, col, 1, 1) >= NEEDED_IN_A_ROW {
            player
        } else if self.count_same_bidirectional(row, col, 1, -1) >= NEEDED_IN_A_ROW {
            player
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use aper::{PlayerID, Timestamp};

    use chrono::{TimeZone, Utc};

    use crate::state::Player::{Brown, Teal};

    use super::*;

    #[test]
    fn test_game() {
        let mut game = DropFourGame::default();
        let dummy_timestamp = Utc.timestamp_millis(0);
        let player1 = PlayerID(1);
        let player2 = PlayerID(2);

        assert_eq!(PlayState::NextTurn(Player::Brown), game.state());

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            game.drop(4),
        ));

        assert_eq!(PlayState::NextTurn(Player::Teal), game.state());
        assert_eq!(Some(Brown), game.board()[5][4]);

        //     v
        // .......
        // .......
        // .......
        // .......
        // .......
        // ....B..

        game.apply(TransitionEvent::new(
            Some(player2),
            dummy_timestamp,
            game.drop(4),
        ));

        assert_eq!(PlayState::NextTurn(Player::Brown), game.state());
        assert_eq!(Some(Teal), game.board()[4][4]);

        //     v
        // .......
        // .......
        // .......
        // .......
        // ....Y..
        // ....B..

        game.apply(TransitionEvent::new(
            Some(player2),
            dummy_timestamp,
            game.drop(3),
        ));

        assert_eq!(PlayState::NextTurn(Player::Teal), game.state());
        assert_eq!(Some(Brown), game.board()[5][3]);

        //    v
        // .......
        // .......
        // .......
        // .......
        // ....Y..
        // ...BB..

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            game.drop(5),
        ));

        assert_eq!(PlayState::NextTurn(Player::Brown), game.state());
        assert_eq!(Some(Teal), game.board()[5][5]);

        //      v
        // .......
        // .......
        // .......
        // .......
        // ....Y..
        // ...BBY.

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            game.drop(2),
        ));

        assert_eq!(PlayState::NextTurn(Player::Teal), game.state());
        assert_eq!(Some(Brown), game.board()[5][2]);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ....Y..
        // ..BBBY.

        game.apply(TransitionEvent::new(
            Some(player2),
            dummy_timestamp,
            game.drop(2),
        ));

        assert_eq!(PlayState::NextTurn(Player::Brown), game.state());
        assert_eq!(Some(Teal), game.board()[4][2]);

        //   v
        // .......
        // .......
        // .......
        // .......
        // ..Y.Y..
        // ..BBBY.

        game.apply(TransitionEvent::new(
            Some(player1),
            dummy_timestamp,
            game.drop(1),
        ));

        assert_eq!(PlayState::Winner(Player::Brown), game.state());
        assert_eq!(Some(Brown), game.board()[5][1]);

        //  v
        // .......
        // .......
        // .......
        // .......
        // ..Y.Y..
        // .BBBBY.
    }
}
