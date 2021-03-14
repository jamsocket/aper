pub use aper_derive::{StateMachine, Transition};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// This trait indicates that a type can be used as the transition of a [StateMachine].
pub trait Transition:
    Sized + Unpin + 'static + Send + Clone + DeserializeOwned + Serialize + Debug + PartialEq
{
}

/// This trait provides the methods that Aper needs to be able to interact with
/// an object as a state machine.
///
/// None of the methods in this trait provide access to the internal data of the
/// state machine. It's up to you to implement accessor methods (or use public
/// fields) in order to expose the data necessary to render your views.
pub trait StateMachine:
    Sized + Unpin + 'static + Send + Clone + DeserializeOwned + Serialize + Debug
{
    /// The [StateMachine::Transition] type associates another type with this state machine
    /// as its transitions.
    type Transition: Transition;

    /// Update the state machine according to the given [Transition]. This method *must* be
    /// deterministic: calling it on a clone of the state with a clone of the [Transition]
    /// must result in the same state, even at a different time and on a different machine. This
    /// is the requirement that allows Aper to keep the state in sync across multiple machines.
    fn apply(&mut self, transition: Self::Transition);
}
