use aper_stateroom::{ClientId, Timestamp};
use yew::{Callback, Html};

/// Context passed to a [View].
pub struct ViewContext<T> {
    /// A function called to invoke a state machine transformation.
    pub callback: Callback<T>,

    /// A function called to force a redraw.
    pub redraw: Callback<()>,

    /// The ID of the current player.
    pub client: ClientId,

    /// An estimate of the server's time as of the call to `view`.
    pub time: Timestamp,
}

/// Applies to a struct that can turn a value of the associated `State` type into `yew::Html`.
/// The resulting view can emit events of type `Option<Callback>`.
pub trait View: Clone + PartialEq {
    type State;
    type Callback;

    /// Turn a value into HTML.
    fn view(&self, value: &Self::State, context: &ViewContext<Self::Callback>) -> Html;
}
