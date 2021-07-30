use std::marker::PhantomData;

use aper::{PlayerID, StateProgram, StateUpdateMessage, TransitionEvent};
use chrono::Utc;
use jamsocket::{JamsocketContext, JamsocketService, JamsocketServiceFactory, MessageRecipient};

pub struct AperJamsocketService<P: StateProgram, C: JamsocketContext> {
    state: P,
    suspended_event: Option<TransitionEvent<P::T>>,
    context: C,
}

impl<P: StateProgram, C: JamsocketContext> AperJamsocketService<P, C> {
    fn update_suspended_event(&mut self) {
        let susp = self.state.suspended_event();
        if susp == self.suspended_event {
            return;
        }

        if let Some(ev) = &susp {
            if let Ok(dur) = ev.timestamp.signed_duration_since(Utc::now()).to_std() {
                self.context.set_timer(dur.as_millis() as u32);
            }
        }

        self.suspended_event = susp;
    }

    fn process_transition(&mut self, user: u32, transition: TransitionEvent<P::T>) {
        if transition.player != Some(PlayerID(user as usize)) {
            log::warn!(
                "Received a transition from a client with an invalid player ID. {:?} != {}",
                transition.player,
                user
            );
            return;
        }
        self.state.apply(transition.clone());
        self.context.send_message(
            MessageRecipient::Broadcast,
            serde_json::to_string(&StateUpdateMessage::TransitionState::<P>(transition))
                .unwrap()
                .as_str(),
        );
        self.update_suspended_event();
    }
}

impl<P: StateProgram, C: JamsocketContext> JamsocketService for AperJamsocketService<P, C> {
    fn connect(&mut self, user: u32) {
        self.context.send_message(
            MessageRecipient::User(user),
            serde_json::to_string(&StateUpdateMessage::ReplaceState::<P>(
                self.state.clone(),
                Utc::now(),
                PlayerID(user as usize),
            ))
            .unwrap()
            .as_str(),
        );
    }

    fn disconnect(&mut self, _user: u32) {}

    fn message(&mut self, user: u32, message: &str) {
        let transition: TransitionEvent<P::T> = serde_json::from_str(message).unwrap();
        self.process_transition(user, transition);
    }

    fn binary(&mut self, user: u32, message: &[u8]) {
        let transition: TransitionEvent<P::T> = bincode::deserialize(message).unwrap();
        self.process_transition(user, transition);
    }

    fn timer(&mut self) {
        if let Some(event) = self.suspended_event.take() {
            self.state.apply(event);
            self.update_suspended_event();
        }
    }
}

pub struct AperJamsocketServiceBuilder<
    K: StateProgram<U=String>,
> {
    ph_k: PhantomData<K>,
}

/// This manual derive is necessary because the derive macro for Default
/// isn't smart enough to realize that `K` is only used as a phantom type,
/// so it adds `K: Default` to the bounds of the derived implementation.
impl<K: StateProgram<U=String>> Default for AperJamsocketServiceBuilder<K> {
    fn default() -> Self {
        AperJamsocketServiceBuilder {
            ph_k: Default::default()
        }
    }
}

impl<K: StateProgram<U=String>, C: JamsocketContext>
    JamsocketServiceFactory<C> for AperJamsocketServiceBuilder<K>
{
    type Service = AperJamsocketService<K, C>;

    fn build(&self, room_id: &str, context: C) -> Self::Service {
        let state = K::new(room_id.to_string());

        AperJamsocketService {
            state,
            suspended_event: None,
            context,
        }
    }
}
