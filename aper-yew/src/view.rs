use aper_stateroom::{ClientId, StateProgram};
use chrono::{DateTime, Duration, Utc};
use yew::{Callback, Html};

pub struct StateProgramViewContext<P: StateProgram> {
    pub callback: Callback<P::T>,
    pub redraw: Callback<()>,
    pub client_id: ClientId,
    pub offset: Duration,
}

impl<P: StateProgram> StateProgramViewContext<P> {
    pub fn timestamp(&self) -> DateTime<Utc> {
        Utc::now().checked_add_signed(self.offset).unwrap()
    }
}

pub trait StateProgramViewComponent: 'static {
    type Program: StateProgram;

    fn view(state: &Self::Program, context: StateProgramViewContext<Self::Program>) -> Html;
}
