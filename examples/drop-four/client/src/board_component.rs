use yew::{Callback, Component};
use yew::prelude::*;

use crate::{BOARD_COLS, BOARD_ROWS, DropFourGame, DropFourGameTransition, Player};
use crate::state::PlayState;

const CELL_SIZE: u32 = 80;
const CELL_INNER_SIZE: u32 = 70;
const CELL_HOLE_SIZE: u32 = 60;

const TEAL: &str = "#4CA9AB";
const BROWN: &str = "#C4A07F";

const BOARD_FG: &str = "#D8E3D7";
const BOARD_BG: &str = "#bbc4bb";

const PADDING_SIDE: u32 = 40;
const PADDING_TOP: u32 = 50;
const PADDING_BOTTOM: u32 = 10;


pub struct BoardComponent {
    hover_col: Option<u32>,
    props: BoardProps,
    link: ComponentLink<Self>,
}

pub struct SetHoverCol(Option<u32>);

#[derive(Properties, Clone, PartialEq)]
pub struct BoardProps {
    pub state: DropFourGame,
    pub callback: Callback<DropFourGameTransition>,
}

impl BoardComponent {
    fn view_disc(&self, player: Player, offset: i32) -> Html {
        let color = match player {
            Player::Brown => BROWN,
            Player::Teal => TEAL,
        };

        return html! {
            <g>
                <circle
                    r={CELL_INNER_SIZE/2}
                    fill=&color cy=offset />
                <circle
                    r={CELL_INNER_SIZE/2}
                    fill="black"
                    opacity="0.2"
                    mask="url(#hole_shadow)" />
            </g>
        };
    }

    fn view_holes(&self) -> impl Iterator<Item=Html> {
        (0..BOARD_COLS as u32).flat_map(
            |c| (0..BOARD_ROWS as u32).map(move |r|
                html! {<circle
                    r={CELL_HOLE_SIZE/2}
                    fill="black"
                    cx={CELL_SIZE * c + CELL_SIZE/2}
                    cy={CELL_SIZE * r + CELL_SIZE/2}
                />}
            )
        )
    }

    fn view_hover_zones(&self) -> Html {
        let set_hover_col = self.link.callback(SetHoverCol);
        // TODO: ugly.
        let set_hover_col_clone = set_hover_col.clone();
        let drop_tile = self.props.callback.reform(|c| DropFourGameTransition::Drop(c));
        let zones = (0..BOARD_COLS as u32).map(
            move |c| html! {
                <rect
                    x={CELL_SIZE * c}
                    width=CELL_SIZE
                    height={CELL_SIZE * BOARD_ROWS as u32}
                    opacity=0
                    onmouseover=set_hover_col.reform(move |_| Some(c))
                    onclick=drop_tile.reform(move |_| c as usize)
                />
            }
        );

        html! {
            <g>
                { for zones }
            </g>
        }
    }

    fn view_tentative_disc(&self) -> Html {
        if let Some(disc_col) = self.hover_col {
            if let PlayState::NextTurn(c) = self.props.state.state() {
                let tx = CELL_SIZE * disc_col + CELL_SIZE / 2;
                let ty = CELL_SIZE / 2;
                let style = format!("transform: translate({}px, {}px)", tx, ty);

                return html! {
                    <g style=style class="tentative" >
                        { self.view_disc(c, -(CELL_INNER_SIZE as i32) / 2) }
                    </g>
                }
            }
        }

        html! {}
    }

    fn view_played_discs(&self) -> Html {
        let board = self.props.state.board();

        let col_groups = (0..BOARD_COLS).map(|col| {
            let discs = (0..BOARD_ROWS).rev().flat_map(|row| {
                board[row][col].map(|p| {
                    let ty = CELL_SIZE * row as u32 + CELL_SIZE / 2;
                    let style = format!("transform: translate(0, {}px)", ty);

                    html! {
                        <g style=style class="disc">
                            { self.view_disc(p, 0) }
                        </g>
                    }
                })
            });

            let tx = CELL_SIZE * col as u32 + CELL_SIZE / 2;
            let transform = format!("translate({} 0)", tx);

            html! {
                <g transform=transform>
                    { for discs }
                </g>
            }
        });

        html! {
            <g>
                { for col_groups }
            </g>
        }
    }
}

impl Component for BoardComponent {
    type Properties = BoardProps;
    type Message = SetHoverCol;

    fn view(&self) -> Html {
        let height = BOARD_ROWS as u32 * CELL_SIZE;
        let width = BOARD_COLS as u32 * CELL_SIZE;

        let svg_width = width + 2 * PADDING_SIDE;
        let svg_height = height + PADDING_TOP + PADDING_BOTTOM;

        return html! {
            <svg width=svg_width height=svg_height>
                <mask id="board">
                    <rect width=width height=height fill="white" />
                    { for self.view_holes() }
                </mask>
                <mask id="hole_shadow">
                    <circle r={CELL_HOLE_SIZE/2} fill="white" />
                    <circle r={CELL_HOLE_SIZE/2} fill="black" cy=4 />
                </mask>

                <g transform=format!("translate({} {})", PADDING_SIDE, PADDING_TOP) >
                    <g transform=format!("scale(0.98) translate(6 6)") >
                        <rect width=width height=height fill=BOARD_BG mask="url(#board)" />
                    </g>

                    { self.view_played_discs() }

                    { self.view_tentative_disc() }

                    <rect width=width height=height fill=BOARD_FG mask="url(#board)" />

                    { self.view_hover_zones() }
                </g>
            </svg>
        };
    }

    fn update(&mut self, msg: SetHoverCol) -> ShouldRender {
        let SetHoverCol(c) = msg;

        if c != self.hover_col {
            self.hover_col = c;
            true
        } else {
            false
        }
    }

    fn change(&mut self, props: BoardProps) -> ShouldRender {
        self.props = props;
        true
    }

    fn create(props: BoardProps, link: ComponentLink<Self>) -> Self {
        BoardComponent {
            props,
            link,
            hover_col: None,
        }
    }
}