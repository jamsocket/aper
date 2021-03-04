use crate::{StateMachine, SuspendedEvent, TransitionEvent, Transition};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;

pub trait StateProgram:
    Sized + Unpin + 'static + Send + Clone + DeserializeOwned + Serialize + Debug
{
    /// The [StateMachine::Transition] type associates another type with this state machine
    /// as its transition.
    type Transition: Transition;

    /// Update the state machine according to the given [TransitionEvent]. This method *must* be
    /// deterministic: calling it on a clone of the state with a clone of the [TransitionEvent]
    /// must result in the same state, even at a different time and on a different machine. This
    /// is the requirement that allows Aper to keep the state in sync across multiple machines.
    fn apply(&mut self, transition: TransitionEvent<Self::Transition>);

    /// A state machine may "suspend" an event which occurs at a specific time in the future.
    /// This is useful for ensuring that the state is updated at a future time regardless of
    /// a user-initiated state change before then. State machines that only change state as a
    /// result of user-initiated events can ignore this method, as the default implementation
    /// is to never suspend an event.
    ///
    /// This method is called by the server once after every call to `process_event`. If it
    /// returns `None`, no event is suspended, and any previously suspended event is cancelled.
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
    fn suspended_event(&self) -> Option<SuspendedEvent<Self::Transition>> {
        None
    }
}

impl<T: StateMachine> StateProgram for T {
    type Transition = <T as StateMachine>::Transition;

    fn apply(&mut self, transition: TransitionEvent<Self::Transition>) {
        StateMachine::apply(self, transition.transition);
    }

    fn suspended_event(&self) -> Option<SuspendedEvent<Self::Transition>> {
        None
    }
}

/// A trait indicating that a struct can be used to create a [StateMachine] for a given type.
/// If your [StateMachine] does not need to be initialized with any external data or state,
/// implement [std::default::Default] on it to avoid the need for a factory.
pub trait StateProgramFactory<State: StateProgram>: Sized + Unpin + 'static + Send {
    fn create(&mut self) -> State;
}

/// [StateMachineFactory] implementation that uses the `default` method of the relevant
/// [StateMachine] type.
#[derive(Default)]
struct DefaultStateProgramFactory<State: StateProgram + Default> {
    _phantom: PhantomData<State>,
}

impl<State: 'static + StateProgram + Default + Unpin + Send> StateProgramFactory<State>
    for DefaultStateProgramFactory<State>
{
    fn create(&mut self) -> State {
        Default::default()
    }
}
