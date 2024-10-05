use aper::AperSync;
use aper_yew::{FakeSend, YewAperClient};
use board_component::BoardComponent;
use drop_four_common::{
    Board, DropFourGame, GameTransition, PlayState, PlayerColor, BOARD_COLS, BOARD_ROWS,
};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod board_component;

fn view_waiting(
    connection: &YewAperClient<DropFourGame>,
    waiting_player: Option<u32>,
    client_id: u32,
) -> Html {
    if Some(client_id) == waiting_player {
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
                <button onclick={connection.callback(|| GameTransition::Join)}>{"Join"}</button>
                <p>{message}</p>
            </div>
        }
    }
}

fn view_playing(
    connection: &YewAperClient<DropFourGame>,
    board: &Board,
    next_player: PlayerColor,
    winner: Option<PlayerColor>,
    own_color: Option<PlayerColor>,
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

    html! {
        <div>
            <p>{status_message}</p>
            <BoardComponent
                connection={connection.clone()}
                board={board.clone()}
                player={next_player}
                interactive={Some(next_player)==own_color} />
            {
                if winner.is_some() {
                    html! {
                        <button onclick={connection.callback(|| GameTransition::Reset)}>
                            {"New Game"}
                        </button>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[function_component]
fn GameInner(props: &DropFourGameProps) -> Html {
    let state = props.connection.state();
    let client_id = props.connection.client_id().unwrap_or_default();

    let force_redraw = FakeSend::new(use_force_update());
    state.player_map.teal_player.listen(move || {
        force_redraw.value.force_update();
        true
    });

    let force_redraw = FakeSend::new(use_force_update());
    state.play_state.listen(move || {
        force_redraw.value.force_update();
        true
    });

    match state.play_state.get() {
        PlayState::Playing => {
            let own_color = state.player_map.color_of_player(client_id);
            view_playing(
                &props.connection,
                &state.board,
                state.next_player.get(),
                state.winner.get(),
                own_color,
            )
        }
        PlayState::Waiting => view_waiting(
            &props.connection,
            state.player_map.teal_player.get(),
            client_id,
        ),
    }
}

#[function_component]
fn GameView(props: &DropFourGameProps) -> Html {
    html! {
        <div class="main">
        <h1>{"Drop Four"}</h1>
            <GameInner connection={props.connection.clone()} />
        </div>
    }
}

#[derive(Clone, Properties, PartialEq)]
struct DropFourGameProps {
    connection: YewAperClient<DropFourGame>,
}

#[wasm_bindgen(start)]
pub fn entry() {
    let url = "ws://localhost:8080/ws";

    let connection = YewAperClient::<DropFourGame>::new(url);
    let props = DropFourGameProps { connection };

    yew::Renderer::<GameView>::with_props(props).render();
}
