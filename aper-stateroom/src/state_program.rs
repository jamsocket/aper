use crate::TransitionEvent;
use aper::{Aper, Attach, TreeMapRef};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// This trait can be added to a [StateMachine] which takes a [TransitionEvent] as
/// its transition. Only state machines with this trait can be used directly with
/// the aper client/server infrastructure.
pub trait StateProgram:
    Aper<Intent = TransitionEvent<Self::T>> + Send + Sync + 'static
where
    <Self as StateProgram>::T: Unpin + Send + Sync,
{
    type T: Debug + Serialize + DeserializeOwned + Clone + PartialEq;

    /// A state machine may "suspend" an event which occurs at a specific time in the future.
    /// This is useful for ensuring that the state is updated at a future time regardless of
    /// a user-initiated state change before then. State machines that only change state as a
    /// result of user-initiated events can ignore this method, as the default implementation
    /// is to never suspend an event.
    ///
    /// This method is called by the server once after every call to `process_event`. If it
    /// returns `None`, no event is suspended, and any previously suspended event is canceled.
    /// If it returns `Some`, the provided event becomes the "suspended" event, replacing the
    /// prior suspended event if there was one.
    ///
    /// Only one event can be suspended at a time. If a state machine wants to be triggered for
    /// multiple events in the future, it is up to that state machine to return the
    /// (chronologically) next event each time this method is called.
    ///
    /// Currently, only the state machine running on the server ever has this method called.
    ///
    /// Since they are not associated with a particular player, suspended events trigger
    /// `process_event` with a `None` as the player in the [TransitionEvent].
    fn suspended_event(&self) -> Option<TransitionEvent<Self::T>> {
        None
    }

    fn new() -> Self;
}

/// A [StateProgram] implementation that can be built from any [StateMachine]. Transitions
/// are stripped of their metadata and passed down to the underlying state machine.
pub struct StateMachineContainerProgram<SM: Aper>(pub SM)
where
    <SM as Aper>::Intent: Send;

impl<SM> Attach for StateMachineContainerProgram<SM>
where
    SM: Aper,
    SM::Intent: Send,
{
    fn attach(treemap: TreeMapRef) -> Self {
        StateMachineContainerProgram(SM::attach(treemap))
    }
}

impl<SM: Aper> Aper for StateMachineContainerProgram<SM>
where
    <SM as Aper>::Intent: Send + Unpin + Sync + 'static,
{
    type Intent = TransitionEvent<SM::Intent>;
    type Error = SM::Error;

    fn apply(&mut self, intent: &Self::Intent) -> Result<(), Self::Error> {
        
        self.0.apply(&intent.intent)?;
        Ok(())
    }
}

// impl<SM: Aper + Default + Send + Sync + 'static> StateProgram
// where
//     <SM as Aper>::Intent: Send + Unpin + Sync,
// {
//     type T = SM::Intent;

//     fn new() -> Self {
//         Self::default()
//     }
// }
