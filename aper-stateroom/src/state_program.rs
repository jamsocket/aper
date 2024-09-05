use crate::IntentEvent;
use aper::{Aper, AperSync, Store, StoreHandle};
use serde::{de::DeserializeOwned, Serialize};

/// This trait can be added to a [StateMachine] which takes a [TransitionEvent] as
/// its transition. Only state machines with this trait can be used directly with
/// the aper client/server infrastructure.
pub trait StateProgram:
    Aper<Intent = IntentEvent<Self::WrappedIntent>> + Send + Sync + 'static
where
    <Self as StateProgram>::WrappedIntent: Unpin + Send + Sync,
{
    type WrappedIntent: Serialize + DeserializeOwned + Clone + PartialEq;

    /// A state machine may "suspend" an event which occurs at a specific time in the future.
    /// This is useful for ensuring that the state is updated at a future time regardless of
    /// a user-initiated state change before then. State machines that only change state as a
    /// result of user-initiated events can ignore this method, as the default implementation
    /// is to never suspend an event.
    fn suspended_event(&self) -> Option<IntentEvent<Self::WrappedIntent>> {
        None
    }

    fn new() -> Self;
}

/// A [StateProgram] implementation that can be built from any [StateMachine]. Transitions
/// are stripped of their metadata and passed down to the underlying state machine.
pub struct StateMachineContainerProgram<SM>(pub SM)
where
    SM: Aper + Send + Sync + 'static,
    <SM as Aper>::Intent: Send;

impl<SM> AperSync for StateMachineContainerProgram<SM>
where
    SM: Aper + Send + Sync + 'static,
    SM::Intent: Send,
{
    fn attach(store: StoreHandle) -> Self {
        StateMachineContainerProgram(SM::attach(store))
    }
}

impl<SM> Aper for StateMachineContainerProgram<SM>
where
    SM: Aper + Send + Sync + 'static,
    <SM as Aper>::Intent: Send + Unpin + Sync + 'static,
{
    type Intent = IntentEvent<SM::Intent>;
    type Error = SM::Error;

    fn apply(&mut self, intent: &Self::Intent) -> Result<(), Self::Error> {
        self.0.apply(&intent.intent)?;
        Ok(())
    }
}

impl<SM> StateProgram for StateMachineContainerProgram<SM>
where
    SM: Aper + Send + Sync + 'static,
    <SM as Aper>::Intent: Send + Unpin + Sync + 'static,
{
    type WrappedIntent = SM::Intent;

    fn new() -> Self {
        let store = Store::default();
        StateMachineContainerProgram(SM::attach(store.handle()))
    }
}
