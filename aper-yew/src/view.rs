use std::rc::Rc;

use aper_stateroom::{ClientId, StateProgram, Timestamp};
use yew::{Callback, Component, Properties};

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

pub trait StateProgramViewComponent:
    Component<Properties = StateProgramViewComponentProps<Self::Program>>
{
    type Program: StateProgram;
}
