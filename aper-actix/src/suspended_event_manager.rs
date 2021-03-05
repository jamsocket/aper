use crate::channel_actor::ChannelActor;
use crate::messages::ChannelMessage;
use actix::{AsyncContext, Context, SpawnHandle};
use aper::{StateProgram, Transition, TransitionEvent};
use chrono::Utc;
use core::option::Option;
use core::option::Option::Some;
use std::marker::PhantomData;

/// A struct that owns zero or one suspended event, and implements the replacement
/// logic, including cancelling an event's future when it is replaced.
pub struct SuspendedEventManager<T: Transition, State: StateProgram<T>> {
    suspended_event: Option<(TransitionEvent<T>, SpawnHandle)>,
    phantom: PhantomData<State>,
}

impl<T: Transition, State: StateProgram<T>> SuspendedEventManager<T, State> {
    pub fn new() -> Self {
        SuspendedEventManager {
            suspended_event: None,
            phantom: Default::default(),
        }
    }

    /// Replace the suspended event with the given one. If the two events are the same,
    /// this method is a no-op.
    pub fn replace(
        &mut self,
        suspended_event: Option<TransitionEvent<T>>,
        ctx: &mut Context<ChannelActor<T, State>>,
    ) {
        if self.suspended_event.as_ref().map(|d| &d.0) == suspended_event.as_ref() {
            // Nothing to do since this is the same event.
            return;
        }

        if let Some((_, handle)) = self.suspended_event {
            ctx.cancel_future(handle);
        }

        if let Some(suspended_event) = suspended_event {
            if let Ok(duration) = suspended_event
                .timestamp
                .signed_duration_since(Utc::now())
                .to_std()
            {
                let handle =
                    ctx.notify_later(ChannelMessage::Tick(suspended_event.clone()), duration);
                self.suspended_event = Some((suspended_event, handle))
            } else {
                println!("Negative duration encountered.")
            }
        }
    }
}
