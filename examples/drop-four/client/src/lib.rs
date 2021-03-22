#![recursion_limit = "1024"]

use aper_yew::{ClientBuilder, View, ViewContext};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::state::PlayState;
pub use crate::state::{
    Board, DropFourGame, DropFourGameTransition, Player, BOARD_COLS, BOARD_ROWS,
};
use board_component::BoardComponent;
use aper::PlayerID;

mod board_component;
mod state;

#[derive(Clone)]
struct GameView;

impl GameView {
    fn view_waiting(&self, waiting_player: Option<PlayerID>, context: &ViewContext<DropFourGameTransition>) -> Html {
        return html! {
            if Some(context.player) == waiting_player {
                html! {
                    <p>{"Waiting for another player."}</p>
                }
            } else {
                let message = if waiting_player.is_some() {
                    "One player is waiting to play."
                } else {
                    "Nobody is waiting to play."
                };

                html! {
                    <div>
                        <button onclick=context.callback.reform(|_| Some(DropFourGameTransition::Join))>{"Join"}</button>
                        <p>{message}</p>
                    </div>
                }
            }
        };
    }

    fn view_playing(
        &self,
        board: &Board,
        next_player: Player,
        winner: Option<Player>,
        interactive: bool,
        context: &ViewContext<DropFourGameTransition>,
    ) -> Html {
        let status_message = if let Some(winner) = winner {
            format!("{} is the winner!", winner.name())
        } else {
            format!("It's {}'s turn", next_player.name())
        };

        return html! {
            <div>
                <BoardComponent
                    board=board
                    player=next_player
                    interactive=interactive
                    callback=context.callback.reform(Some).clone() />
                <p>{status_message}</p>
                <button onclick=context.callback.reform(|_| Some(DropFourGameTransition::Reset))>{"New Game"}</button>
            </div>
        };
    }

    fn view_inner(&self, state: &DropFourGame, context: &ViewContext<DropFourGameTransition>) -> Html {
        match state.state() {
            PlayState::Playing {
                board,
                next_player,
                winner,
                ..
            } => {
                let interactive = state.is_player_next(context.player);
                self.view_playing(board, *next_player, *winner, interactive, context)
            },
            PlayState::Waiting { waiting_player, .. } => self.view_waiting(*waiting_player, context),
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
                { self.view_inner(&state, context) }
            </div>
        };
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    ClientBuilder::new(GameView).mount_to_body();
}
