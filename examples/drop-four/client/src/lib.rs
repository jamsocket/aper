#![recursion_limit="1024"]

use aper_yew::{ClientBuilder, View, ViewContext};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub use crate::state::{Player, DropFourGame, DropFourGameTransition, Board, BOARD_COLS, BOARD_ROWS};
use crate::state::PlayState;
use board_component::BoardComponent;

mod state;
mod board_component;

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
}

impl View for GameView {
    type Callback = DropFourGameTransition;
    type State = DropFourGame;

    fn view(&self, state: &Self::State, context: &ViewContext<Self::Callback>) -> Html {
        return html! {
            <div class="main">
                <h1>{"Drop Four"}</h1>
                <BoardComponent
                    state=state.clone()
                    callback=context.callback.reform(Some).clone() />
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
