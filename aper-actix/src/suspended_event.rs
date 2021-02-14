use crate::channel_actor::ChannelActor;
use crate::messages::ChannelMessage;
use actix::{AsyncContext, Context, SpawnHandle};
use aper::{StateMachine, SuspendedEvent};
use chrono::Utc;
use core::option::Option;
use core::option::Option::Some;

/// A struct that owns zero or one suspended event, and implements the replacement
/// logic, including cancelling an event's future when it is replaced.
pub struct SuspendedEventManager<State: StateMachine> {
    suspended_event: Option<(SuspendedEvent<State::Transition>, SpawnHandle)>,
}

impl<State: StateMachine> SuspendedEventManager<State> {
    pub fn new() -> Self {
        SuspendedEventManager {
            suspended_event: None,
        }
    }

    /// Replace the suspended event with the given one. If the two events are the same,
    /// this method is a no-op.
    pub fn replace(
        &mut self,
        suspended_event: Option<SuspendedEvent<State::Transition>>,
        ctx: &mut Context<ChannelActor<State>>,
    ) {
        if self.suspended_event.as_ref().map(|d| &d.0) == suspended_event.as_ref() {
            // Nothing to do since this is the same event.
            return;
        }

        if let Some((_, handle)) = self.suspended_event {
            ctx.cancel_future(handle);
        }

        if let Some(suspended_event) = suspended_event {
            let duration = suspended_event
                .time
                .signed_duration_since(Utc::now())
                .to_std()
                .unwrap_or_default();
            let handle = ctx.notify_later(
                ChannelMessage::Tick(suspended_event.transition.clone()),
                duration,
            );
            self.suspended_event = Some((suspended_event, handle))
        }
    }
}
