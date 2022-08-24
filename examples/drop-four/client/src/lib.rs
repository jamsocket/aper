#![recursion_limit = "1024"]

use aper_stateroom::ClientId;
use aper_yew::{
    StateProgramComponent, StateProgramComponentProps, StateProgramViewComponent,
    StateProgramViewComponentProps,
};
use board_component::BoardComponent;
use drop_four_common::{
    Board, DropFourGame, GameTransition, PlayState, PlayerColor, BOARD_COLS, BOARD_ROWS,
};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod board_component;

#[derive(Clone, PartialEq)]
struct GameView;

impl GameView {
    fn view_waiting(
        &self,
        waiting_player: Option<ClientId>,
        client_id: ClientId,
        callback: &Callback<GameTransition>,
    ) -> Html {
        if Some(client_id) == waiting_player {
            return html! {
                <p>{"Waiting for another player."}</p>
            };
        } else {
            let message = if waiting_player.is_some() {
                "One player is waiting to play."
            } else {
                "Nobody is waiting to play."
            };

            return html! {
                <div>
                    <button onclick={callback.reform(|_| GameTransition::Join)}>{"Join"}</button>
                    <p>{message}</p>
                </div>
            };
        }
    }

    fn view_playing(
        &self,
        board: &Board,
        next_player: PlayerColor,
        winner: Option<PlayerColor>,
        own_color: Option<PlayerColor>,
        callback: &Callback<GameTransition>,
    ) -> Html {
        let status_message = if let Some(own_color) = own_color {
            if let Some(winner) = winner {
                if winner == own_color {
                    "Congrats, you are the winner!".to_string()
                } else {
                    format!("{} is the winner. Better luck next time!", winner.name())
                }
            } else if next_player == own_color {
                "It's your turn!".to_string()
            } else {
                format!("It's {}'s turn", next_player.name())
            }
        } else {
            format!("You're observing. {} is next.", next_player.name())
        };

        return html! {
            <div>
                <p>{status_message}</p>
                <BoardComponent
                    board={board.clone()}
                    player={next_player}
                    interactive={Some(next_player)==own_color}
                    callback={callback.clone()} />
                {
                    if winner.is_some() {
                        html! {
                            <button onclick={callback.reform(|_| GameTransition::Reset)}>
                                {"New Game"}
                            </button>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        };
    }

    fn view_inner(
        &self,
        state: &DropFourGame,
        client_id: ClientId,
        callback: &Callback<GameTransition>,
    ) -> Html {
        match state.state() {
            PlayState::Playing {
                board,
                next_player,
                winner,
                player_map,
                ..
            } => {
                let own_color = player_map.color_of_player(client_id);
                self.view_playing(board, *next_player, *winner, own_color, callback)
            }
            PlayState::Waiting { waiting_player, .. } => {
                self.view_waiting(*waiting_player, client_id, callback)
            }
        }
    }
}

impl Component for GameView {
    type Message = ();
    type Properties = StateProgramViewComponentProps<DropFourGame>;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, context: &Context<Self>) -> Html {
        let callback = &context.props().callback;
        let state = &context.props().state;

        return html! {
            <div class="main">
                <h1>{"Drop Four"}</h1>
                { self.view_inner(state, context.props().client, callback) }
            </div>
        };
    }
}

impl StateProgramViewComponent for GameView {
    type Program = DropFourGame;
}

#[wasm_bindgen(start)]
pub fn entry() {
    let props = StateProgramComponentProps::new("ws");
    yew::start_app_with_props::<StateProgramComponent<GameView>>(props);
}
