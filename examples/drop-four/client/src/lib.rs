use aper_yew::{ClientBuilder, View, ViewContext};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub use crate::state::{Player, DropFourGame, DropFourGameTransition, Board, BOARD_COLS, BOARD_ROWS};
use crate::state::PlayState;

mod state;

#[derive(Clone)]
struct GameView;


const CELL_SIZE: u32 = 80;
const CELL_INNER_SIZE: u32 = 70;
const CELL_HOLE_SIZE: u32 = 60;

const YELLOW: &str = "#4CA9AB";
const RED: &str = "#C4A07F";

const BOARD_FG: &str = "#D8E3D7";
const BOARD_BG: &str = "#bbc4bb";

const PADDING_SIDE: u32 = 40;
const PADDING_TOP: u32 = 50;
const PADDING_BOTTOM: u32 = 10;

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

    fn view_tile(&self, color: &str, offset: u32) -> Html {
        return html! {
            <g>
                <circle r={CELL_INNER_SIZE/2} fill=&color cy=offset />
                <circle r={CELL_INNER_SIZE/2} fill="black" opacity="0.2" mask="url(#circ)" />
            </g>
        }
    }

    fn view_board(&self, board: &Board, callback: Callback<Option<DropFourGameTransition>>) -> Html {
        let height = BOARD_ROWS as u32 * CELL_SIZE;
        let width = BOARD_COLS as u32 * CELL_SIZE;

        let svg_width = width + 2 * PADDING_SIDE;
        let svg_height = height + PADDING_TOP + PADDING_BOTTOM;

        let holes = (0..BOARD_COLS as u32).flat_map(
            |c| (0..BOARD_ROWS as u32).map(move |r|
                html! {<circle
                    r={CELL_HOLE_SIZE/2}
                    fill="black"
                    cx={CELL_SIZE * c + CELL_SIZE/2}
                    cy={CELL_SIZE * r + CELL_SIZE/2}
                />}
            )
        );

        return html!{
            <svg width=svg_width height=svg_height style="border: 1px solid black;">
                <mask id="board">
                    <rect width=width height=height fill="white" />
                    { for holes }
                </mask>

                <g transform=format!("translate({} {})", PADDING_SIDE, PADDING_TOP) >
                    <g transform=format!("scale(0.98) translate(6 6)") >
                        <rect width=width height=height fill=BOARD_BG mask="url(#board)" />
                    </g>

                    <rect width=width height=height fill=BOARD_FG mask="url(#board)" />
                </g>
            </svg>
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
