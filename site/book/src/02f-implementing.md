# Implementing a new state machine

So far, aside from the [opening example](building.md), we've created
state machines by composing primitive, pre-built
state machines and using the derive macro. For total flexibility, you 
can also implement your own. This may be useful when you want to
implement a shared data structure that can't be expressed efficiently 
with Aper's built-in data structures, or when your program state
involves complex update logic. A great example of the latter is games,
in which the games rules determine how state is updated.

To demonstrate this, let's implement a state machine representing
the game Tic Tac Toe.

Tic Tac Toe has two players, **X** and **O**, which we can represent
as an enum:

```rust,noplaypen
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
enum Player { X, O }
```

There are three possible states to a tic tac toe game:
1. The game is underway and waiting on a particular player to play 
   next.
2. The game has ended with a winner.
3. The game has ended in a tie, because the board is filled but no
   player has won.
   
We will represent all three of these with the `GameStatus` struct:

```rust,noplaypen
# use serde::{Serialize, Deserialize};
# #[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
# enum Player { X, O }
#
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
enum GameStatus {
    /// Indicates that the game is ongoing and the provided
    /// player is next.
    NextPlayer(Player),

    /// Indicates that the game has been won by the given player.
    Won(Player),

    /// Indicates that the game has ended in a tie.
    Tie,
}
```

The board is a 3x3 grid. For simplicity, let's flatten this into
an array with 9 entries. Each grid space can either be an `X`,
an `O`, or empty. We can thus represent each grid cell as an
`Option<Player>`.

The board cells correspond to indices in the flattened grid as
follows:

```plain
0 | 1 | 2
--+---+--
3 | 4 | 5
--+---+--
6 | 7 | 8
```

Combining the `GameStatus` with the grid, we have a game state that
looks like this:

```rust,noplaypen
# use serde::{Serialize, Deserialize};
# #[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
# enum Player { X, O }
#
# #[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
# enum GameStatus {
#     /// Indicates that the game is ongoing and the provided
#     /// player is next.
#     NextPlayer(Player),
#
#     /// Indicates that the game has been won by the given player.
#     Won(Player),
#
#     /// Indicates that the game has ended in a tie.
#     Tie,
# }
#
#[derive(Serialize, Deserialize, Clone, Debug)]
struct TicTacToe {
    /// The current state of the board as a flattened 3x3 grid.
    board: [Option<Player>; 9],

    /// The next player to play, or `None` if the game has ended.
    status: GameStatus,
}
```

Next, we need to implement some of the game logic. We need to be
able to construct a new game, and also to be able to check if a
player has won or if the game has ended in a tie.

```rust,noplaypen
# use serde::{Serialize, Deserialize};
# #[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
# enum Player { X, O }
#
# #[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
# enum GameStatus {
#     /// Indicates that the game is ongoing and the provided
#     /// player is next.
#     NextPlayer(Player),
#
#     /// Indicates that the game has been won by the given player.
#     Won(Player),
#
#     /// Indicates that the game has ended in a tie.
#     Tie,
# }
#
# #[derive(Serialize, Deserialize, Clone, Debug)]
# struct TicTacToe {
#     /// The current state of the board as a flattened 3x3 grid.
#     board: [Option<Player>; 9],
#
#     /// The next player to play, or `None` if the game has ended.
#     status: GameStatus,
# }
#
impl TicTacToe {
    pub fn new() -> TicTacToe {
        TicTacToe {
            // Start with an empty board.
            board: [None; 9],
            // We'll default to player `O` going first.
            status: GameStatus::NextPlayer(Player::O),
        }
    }

    /// Given an array of three grid indices, check whether
    /// the same player has a mark in all three. If so, return
    /// the value of that player. Otherwise, return `None`.
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

    /// Return the value of the player who has won if the
    /// game has ended, or else return `None`.
    fn winner(&self) -> Option<Player> {
        // To win tic tac toe, a player must occupy every
        // cell in either a column, row, or diagonal. There
        // are eight sets of three cells we need to check.
        let seq_to_check: [[usize; 3]; 8] = [
            // Three rows.
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            // Three columns.
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            // Two diagonals.
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

    /// Return `true` if the game has ended in a tie because
    /// there are no empty grid cells available to play.
    fn tie(&self) -> bool {
        self.board.iter().all(|d| d.is_some())
    }
}
```

The last step is to make `TicTacToe` a valid `StateMachine`.
We'll start by creating the transition type, `TicTacToeMove`.
Usually transition types are `enum`s, but they don't have to be.
In the case of Tic Tac Toe, there's only one type of move a
player can make: to make their mark in an available grid space.
We represent this one-and-only play with a `TicTacToeMove` struct,
referencing the cell in play by the same flattened numbering
scheme we used to implement the grid as a flat list.

```rust,noplaypen
# use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TicTacToeMove(usize);
```

Finally, we can implement `StateMachine` for `TicTacToe`, using
`TicTacToeMove` as the transition.

```rust,noplaypen
# use serde::{Serialize, Deserialize};
# #[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
# enum Player { X, O }
#
# #[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
# enum GameStatus {
#     /// Indicates that the game is ongoing and the provided
#     /// player is next.
#     NextPlayer(Player),
#
#     /// Indicates that the game has been won by the given player.
#     Won(Player),
#
#     /// Indicates that the game has ended in a tie.
#     Tie,
# }
#
# #[derive(Serialize, Deserialize, Clone, Debug)]
# struct TicTacToe {
#     /// The current state of the board as a flattened 3x3 grid.
#     board: [Option<Player>; 9],
#
#     /// The next player to play, or `None` if the game has ended.
#     status: GameStatus,
# }
# impl TicTacToe {
#     pub fn new() -> TicTacToe {
#         TicTacToe {
#             // Start with an empty board.
#             board: [None; 9],
#             // We'll default to player `O` going first.
#             status: GameStatus::NextPlayer(Player::O),
#         }
#     }
# 
#     /// Given an array of three grid indices, check whether
#     /// the same player has a mark in all three. If so, return
#     /// the value of that player. Otherwise, return `None`.
#     fn check_seq(&self, seq: &[usize; 3]) -> Option<Player> {
#         let v1 = self.board[seq[0]]?;
#         let v2 = self.board[seq[1]]?;
#         let v3 = self.board[seq[2]]?;
# 
#         if (v1 == v2) && (v2 == v3) {
#             Some(v1)
#         } else {
#             None
#         }
#     }
# 
#     /// Return the value of the player who has won if the
#     /// game has ended, or else return `None`.
#     fn winner(&self) -> Option<Player> {
#         // To win tic tac toe, a player must occupy every
#         // cell in either a column, row, or diagonal. There
#         // are eight sets of three cells we need to check.
#         let seq_to_check: [[usize; 3]; 8] = [
#             // Three rows.
#             [0, 1, 2],
#             [3, 4, 5],
#             [6, 7, 8],
#             // Three columns.
#             [0, 3, 6],
#             [1, 4, 7],
#             [2, 5, 8],
#             // Two diagonals.
#             [0, 4, 8],
#             [2, 4, 6],
#         ];
# 
#         for seq in &seq_to_check {
#             let result = self.check_seq(seq);
#             if result.is_some() {
#                 return result;
#             }
#         }
# 
#         None
#     }
# 
#     /// Return `true` if the game has ended in a tie because
#     /// there are no empty grid cells available to play.
#     fn tie(&self) -> bool {
#         self.board.iter().all(|d| d.is_some())
#     }
# }
# #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
# struct TicTacToeMove(usize);
#
use aper::{StateMachine};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum TicTacToeConflict {
    GameAlreadyEnded,
    SquareAlreadyFilled,
}

impl StateMachine for TicTacToe {
    type Transition = TicTacToeMove;
    type Conflict = TicTacToeConflict;
    
    fn apply(&self, play: &TicTacToeMove) -> Result<TicTacToe, TicTacToeConflict> {
        let this_player = if let GameStatus::NextPlayer(this_player)
                = self.status {
            this_player
        } else {
            // The game has already ended, don't accept another play.
            return Err(TicTacToeConflict::GameAlreadyEnded);
        };
        let TicTacToeMove(play_index) = play;
        if self.board[*play_index].is_some() {
            // Can't play over something that has already been played!
            return Err(TicTacToeConflict::SquareAlreadyFilled);
        }
        // Insert this play into the board.
        let mut board = self.board.clone();
        board[*play_index] = Some(this_player);
        
        // Check if the game has ended.
        if let Some(winner) = self.winner() {
            return Ok(TicTacToe {board, status: GameStatus::Won(winner)});
        } else if self.tie() {
            return Ok(TicTacToe {board, status: GameStatus::Tie});
        }

        // Update the next player.
        if Player::X == this_player {
            Ok(TicTacToe {board, status: GameStatus::NextPlayer(Player::O)})
        } else {
            Ok(TicTacToe {board, status: GameStatus::NextPlayer(Player::X)})
        }
    }
}
```