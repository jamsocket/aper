use yew::prelude::*;
use yew::{Callback, Component};

use crate::{Board, GameTransition, PlayerColor, BOARD_COLS, BOARD_ROWS};

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
}

pub struct SetHoverCol(Option<u32>);

#[derive(Properties, Clone)]
pub struct BoardProps {
    pub board: Board,
    pub player: PlayerColor,
    pub interactive: bool,
    pub callback: Callback<GameTransition>,
}

impl PartialEq for BoardProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl BoardComponent {
    fn view_disc(&self, player: PlayerColor, offset: i32) -> Html {
        let color = match player {
            PlayerColor::Brown => BROWN,
            PlayerColor::Teal => TEAL,
        };

        html! {
            <g>
                <circle
                    r={(CELL_INNER_SIZE/2).to_string()}
                    fill={color} cy={offset.to_string()} />
                <circle
                    r={(CELL_INNER_SIZE/2).to_string()}
                    fill="black"
                    opacity="0.2"
                    mask="url(#hole_shadow)" />
            </g>
        }
    }

    fn view_holes(&self) -> impl Iterator<Item = Html> {
        (0..BOARD_COLS).flat_map(|c| {
            (0..BOARD_ROWS).map(move |r| {
                html! {<circle
                    r={(CELL_HOLE_SIZE/2).to_string()}
                    fill="black"
                    cx={(CELL_SIZE * c + CELL_SIZE/2).to_string()}
                    cy={(CELL_SIZE * r + CELL_SIZE/2).to_string()}
                />}
            })
        })
    }

    fn view_hover_zones(&self, context: &yew::Context<Self>) -> Html {
        let set_hover_col = context.link().callback(SetHoverCol);
        let drop_tile = context.props().callback.reform(GameTransition::Drop);
        let zones = (0..BOARD_COLS).map(move |c| {
            html! {
                <rect
                    x={(CELL_SIZE * c).to_string()}
                    width={CELL_SIZE.to_string()}
                    height={(CELL_SIZE * BOARD_ROWS).to_string()}
                    opacity="0"
                    onmouseover={set_hover_col.reform(move |_| Some(c))}
                    onclick={drop_tile.reform(move |_| c as usize)}
                />
            }
        });

        html! {
            <g>
                { for zones }
            </g>
        }
    }

    fn view_tentative_disc(&self, context: &yew::Context<Self>) -> Html {
        if let Some(disc_col) = self.hover_col {
            if context.props().interactive {
                let tx = CELL_SIZE * disc_col + CELL_SIZE / 2;
                let ty = CELL_SIZE / 2;
                let style = format!("transform: translate({}px, {}px)", tx, ty);

                return html! {
                    <g style={style} class="tentative" >
                        { self.view_disc(context.props().player, -(CELL_INNER_SIZE as i32) / 2) }
                    </g>
                };
            }
        }

        html! {}
    }

    fn view_played_discs(&self, context: &yew::Context<Self>) -> Html {
        let board = &context.props().board;

        let col_groups = (0..BOARD_COLS).map(|col| {
            let discs = (0..BOARD_ROWS).rev().flat_map(|row| {
                board.get(row, col).map(|p| {
                    let ty = CELL_SIZE * row + CELL_SIZE / 2;
                    let style = format!("transform: translate(0, {}px)", ty);

                    html! {
                        <g style={style} class="disc">
                            { self.view_disc(p, 0) }
                        </g>
                    }
                })
            });

            let tx = CELL_SIZE * col + CELL_SIZE / 2;
            let transform = format!("translate({} 0)", tx);

            html! {
                <g transform={transform}>
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

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let height = BOARD_ROWS * CELL_SIZE;
        let width = BOARD_COLS * CELL_SIZE;

        let svg_width = width + 2 * PADDING_SIDE;
        let svg_height = height + PADDING_TOP + PADDING_BOTTOM;

        html! {
            <svg width={svg_width.to_string()} height={svg_height.to_string()}>
                <mask id="board">
                    <rect width={width.to_string()} height={height.to_string()} fill="white" />
                    { for self.view_holes() }
                </mask>
                <mask id="hole_shadow">
                    <circle r={(CELL_HOLE_SIZE/2).to_string()} fill="white" />
                    <circle r={(CELL_HOLE_SIZE/2).to_string()} fill="black" cy="4" />
                </mask>

                <g transform={format!("translate({} {})", PADDING_SIDE, PADDING_TOP)} >
                    <g transform="scale(0.98) translate(6 6)">
                        <rect width={width.to_string()} height={height.to_string()} fill={BOARD_BG} mask="url(#board)" />
                    </g>

                    { self.view_played_discs(context) }

                    { self.view_tentative_disc(context) }

                    <rect width={width.to_string()} height={height.to_string()} fill={BOARD_FG} mask="url(#board)" />

                    { self.view_hover_zones(context) }
                </g>
            </svg>
        }
    }

    fn update(&mut self, _context: &yew::Context<Self>, msg: SetHoverCol) -> bool {
        let SetHoverCol(c) = msg;

        if c != self.hover_col {
            self.hover_col = c;
            true
        } else {
            false
        }
    }

    fn create(_context: &yew::Context<Self>) -> Self {
        BoardComponent { hover_col: None }
    }
}
