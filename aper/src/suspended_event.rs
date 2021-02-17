use chrono::{DateTime, Utc};

/// Represents a transition that a [crate::StateMachine] would like to receive in the
/// future from the server, as well as the time that it would like to receive it. Each state machine
/// can have either zero or one transition suspended at any point in time, and may change or
/// remove that suspended event every time its state is updated (and only then).
///
/// Storing and maintaining the suspended events is the responsibility of the code that owns the
/// [crate::StateMachine] object.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct SuspendedEvent<Transition> {
    /// When the event should be triggered. Note that this is not necessarily equal to the timestamp
    /// field on the [crate::TransitionEvent] that is created when this event is triggered, since
    /// there may be a small delay between the time an event is requested at and the time it is
    /// actually fired. The latter is used as the timestamp when creating the
    /// [crate::TransitionEvent].
    pub time: DateTime<Utc>,

    /// The transition that should be triggered. This is turned into a [crate::TransitionEvent]
    /// instance using `None` as the player, since these events are not considered to be initiated
    /// by any player.
    pub transition: Transition,
}

impl<Transition> SuspendedEvent<Transition> {
    pub fn new(time: DateTime<Utc>, transition: Transition) -> SuspendedEvent<Transition> {
        SuspendedEvent { time, transition }
    }

    /// Create a new [SuspendedEvent] that has the same time as this one, but whose transition is
    /// the result of applying the provided function to it. This is useful when nesting state
    /// machines.
    pub fn map_transition<NewTransition>(
        self,
        fun: impl FnOnce(Transition) -> NewTransition,
    ) -> SuspendedEvent<NewTransition> {
        SuspendedEvent::new(self.time, fun(self.transition))
    }
}
