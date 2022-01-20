use std::fmt::Debug;

use aper::StateMachine;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::TransitionEvent;

/// This trait can be added to a [StateMachine] which takes a [TransitionEvent] as
/// its transition. Only state machines with this trait can be used directly with
/// the aper client/server infrastructure.
pub trait StateProgram: StateMachine<Transition = TransitionEvent<Self::T>>
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

    fn new(init_value: &str) -> Self;
}

/// A [StateProgram] implementation that can be built from any [StateMachine]. Transitions
/// are stripped of their metadata and passed down to the underlying state machine.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(bound = "")]
pub struct StateMachineContainerProgram<SM: StateMachine>(pub SM)
where
    <SM as StateMachine>::Transition: Send;

impl<SM: StateMachine> StateMachine for StateMachineContainerProgram<SM>
where
    <SM as StateMachine>::Transition: Send + Unpin + Sync,
{
    type Transition = TransitionEvent<SM::Transition>;
    type Conflict = SM::Conflict;

    fn apply(&mut self, transition: Self::Transition) -> Result<(), Self::Conflict> {
        self.0.apply(transition.transition)
    }
}

impl<SM: StateMachine + Default> StateProgram for StateMachineContainerProgram<SM>
where
    <SM as StateMachine>::Transition: Send + Unpin + Sync,
{
    type T = SM::Transition;

    fn new(_init_value: &str) -> Self {
        Self::default()
    }
}
