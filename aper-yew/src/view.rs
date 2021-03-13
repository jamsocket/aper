use yew::{Html, Callback};
use aper::{PlayerID, Timestamp};

pub struct ViewContext<T> {
    pub callback: Callback<Option<T>>,
    pub redraw: Callback<()>,
    pub player: PlayerID,
    pub time: Timestamp,
}

pub trait View: Clone {
    type State;
    type Callback;

    fn view(&self, value: &Self::State, context: &ViewContext<Self::Callback>) -> Html;
}
