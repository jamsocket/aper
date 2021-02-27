# Implementing new state machines

So far, we've created state machines by composing primitive, pre-built state machines and using the derive macro. For more flexibility, you can also implement your own.

I teased with an example of this with the `Counter`. As a slightly more complex example, let's implement Tic Tac Toe.

```rust
use aper::StateMachine;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
enum Player {
    X,
    O,
}

// The board is a 3x3 grid, flattened into an array:
// 0 | 1 | 2
// --+---+--
// 3 | 4 | 5
// --+---+--
// 6 | 7 | 8

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TicTacToe {
    board: [Option<Player>; 9],
    next_player: Option<Player>, // None if the game has ended.
}

impl TicTacToe {
    pub fn new() -> TicTacToe {
        TicTacToe {
            board: [None; 9],
            next_player: Some(Player::O),
        }
    }

    fn check_seq(&self, seq: &[usize; 3]) -> Option<Player> {
        let v1 = self.board[seq[0]]?;
        let v2 = self.board[seq[1]]?;
        let v3 = self.board[seq[2]]?;

        if (v1 == v2) && (v2 == v3) {
            Some(v1)
        } else {
            None
        }
    }

  fn winner(&self) -> Option<Player> {
        let seq_to_check: [[usize; 3]; 8] = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [2, 4, 6],
        ];

        for seq in &seq_to_check {
            let result = self.check_seq(seq);
            if result.is_some() {
                return result;
            }
        }

        None
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TicTacToePlay(usize);

impl StateMachine for TicTacToe {
    type Transition = TicTacToePlay;
    
    fn apply(&mut self, play: TicTacToePlay) {
        let this_player = if let Some(this_player) = self.next_player {
            this_player
        } else {
            // The game has already been won, don't accept another play.
            return;
        };
        let TicTacToePlay(play_index) = play;
        if self.board[play_index].is_some() {
            // Can't play over something that has already been played!
            return;
        }
        // Insert this play into the board.
        self.board[play_index] = self.next_player;
        
        // Check for a winner.
        if self.winner().is_some() {
            self.next_player = None;
            return;
        }

        // Update the next player.
        if Player::X == this_player {
            self.next_player = Some(Player::O);
        } else {
            self.next_player = Some(Player::X);
        }
    }
}
```