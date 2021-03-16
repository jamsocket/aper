use aper_yew::{ClientBuilder, View, ViewContext};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub use crate::state::{Player, DropFourGame, DropFourGameTransition, Board, BOARD_COLS, BOARD_ROWS};
use crate::state::PlayState;

mod state;

#[derive(Clone)]
struct GameView;

impl GameView {
    fn view_state_text(&self, state: PlayState) -> String {
        match state {
            PlayState::NextTurn(p) => {
                format!("It's {}'s turn", p.name())
            }
            PlayState::Winner( p) => {
                format!("{} is the winner!", p.name())
            }
        }
    }

    fn view_drop_button(&self, col: usize, callback: Callback<Option<DropFourGameTransition>>) -> Html {
        return html! {
            <td>
                <button onclick=callback.reform(move |_| Some(DropFourGameTransition::Drop(col)))>{"v"}</button>
            </td>
        }
    }

    fn view_header(&self, callback: Callback<Option<DropFourGameTransition>>) -> Html {
        return html! {
            <tr>
                {for (0..BOARD_COLS).map(|i| self.view_drop_button(i, callback.clone()))}
            </tr>
        }
    }

    fn view_cell(&self, cell: Option<Player>) -> Html {
        let inner_value = match cell {
            None => "",
            Some(Player::Yellow) => "Y",
            Some(Player::Blue) => "B",
        };

        html! {
            <td>
                { inner_value }
            </td>
        }
    }

    fn view_row(&self, row: [Option<Player>; BOARD_COLS]) -> Html {
        return html! {
            <tr>
                {for (0..BOARD_COLS).map(|i| self.view_cell(row[i]))}
            </tr>
        }
    }

    fn view_board(&self, board: &Board, callback: Callback<Option<DropFourGameTransition>>) -> Html {
        return html!{
            <table>
                {self.view_header(callback)}
                {for (0..BOARD_ROWS).map(|i| self.view_row(board[i]))}
            </table>
        }
    }
}

impl View for GameView {
    type Callback = DropFourGameTransition;
    type State = DropFourGame;

    fn view(&self, state: &Self::State, context: &ViewContext<Self::Callback>) -> Html {

        return html! {
            <div>
                <h1>{"Hello, Aper!"}</h1>
                {self.view_board(state.board(), context.callback.clone())}
                <p>{self.view_state_text(state.state())}</p>
                <button onclick=context.callback.reform(|_| Some(DropFourGameTransition::Reset))>{"New Game"}</button>
            </div>
        };
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    ClientBuilder::new(GameView).mount_to_body();
}
