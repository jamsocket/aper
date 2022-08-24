use aper_stateroom::{ClientId, StateProgram, Timestamp};
use chrono::{DateTime, Utc};
use std::rc::Rc;
use yew::{Callback, Html, Properties};

#[derive(Properties)]
pub struct StateProgramViewComponentProps<S: StateProgram> {
    pub state: Rc<S>,

    /// A function called to invoke a state machine transformation.
    pub callback: Callback<S::T>,

    /// A function called to force a redraw.
    pub redraw: Callback<()>,

    /// The ID of the current player.
    pub client: ClientId,

    /// An estimate of the server's time as of the render.
    pub time: Timestamp,
}

impl<S: StateProgram> PartialEq for StateProgramViewComponentProps<S> {
    fn eq(&self, other: &Self) -> bool {
        self.callback == other.callback
            && self.redraw == other.redraw
            && self.client == other.client
            && self.time == other.time
            && Rc::ptr_eq(&self.state, &other.state)
    }
}

pub struct StateProgramViewContext<P: StateProgram> {
    pub callback: Callback<P::T>,
    pub client_id: ClientId,
    pub timestamp: DateTime<Utc>,
}

pub trait StateProgramViewComponent: 'static {
    type Program: StateProgram;

    fn view(state: Rc<Self::Program>, context: StateProgramViewContext<Self::Program>) -> Html;
}
