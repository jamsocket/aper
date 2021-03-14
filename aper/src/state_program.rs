use crate::{StateMachine, Transition, TransitionEvent};
use serde::{Serialize, Deserialize};

pub trait StateProgram<T: Transition>: StateMachine<Transition = TransitionEvent<T>> {
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
    fn suspended_event(&self) -> Option<TransitionEvent<T>> {
        None
    }
}

/// A trait indicating that a struct can be used to create a [StateMachine] for a given type.
/// If your [StateMachine] does not need to be initialized with any external data or state,
/// implement [std::default::Default] on it to avoid the need for a factory.
pub trait StateProgramFactory<T: Transition, State: StateProgram<T>>:
    Sized + Unpin + 'static + Send
{
    fn create(&mut self) -> State;
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(bound = "")]
pub struct StateMachineContainerProgram<SM: StateMachine>(pub SM);

impl<SM: StateMachine> StateMachine for StateMachineContainerProgram<SM> {
    type Transition = TransitionEvent<SM::Transition>;

    fn apply(&mut self, transition: Self::Transition) {
        self.0.apply(transition.transition);
    }
}

impl<SM: StateMachine> StateProgram<SM::Transition> for StateMachineContainerProgram<SM> {

}

/*
/// [StateMachineFactory] implementation that uses the `default` method of the relevant
/// [StateMachine] type.
#[derive(Default)]
struct DefaultStateProgramFactory<T: Transition, State: StateProgram<T> + Default> {
    _phantom_state: PhantomData<State>,
    _phantom_transition: PhantomData<T>,
}

 */

/*
impl<State: 'static + StateProgram + Default + Unpin + Send> StateProgramFactory<State>
    for DefaultStateProgramFactory<State>
{
    fn create(&mut self) -> State {
        Default::default()
    }
}
*/
