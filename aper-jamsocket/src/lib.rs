use std::marker::PhantomData;

use aper::{PlayerID, StateProgram, StateUpdateMessage, TransitionEvent};
use chrono::Utc;
use jamsocket::{JamsocketContext, JamsocketServiceFactory, MessageRecipient, SimpleJamsocketService, WrappedJamsocketService};

pub struct AperJamsocketService<P: StateProgram> {
    state: P,
    suspended_event: Option<TransitionEvent<P::T>>,
}

impl<P: StateProgram> AperJamsocketService<P> {
    fn update_suspended_event(&mut self, ctx: &impl JamsocketContext) {
        let susp = self.state.suspended_event();
        if susp == self.suspended_event {
            return;
        }

        if let Some(ev) = &susp {
            if let Ok(dur) = ev.timestamp.signed_duration_since(Utc::now()).to_std() {
                ctx.set_timer(dur.as_millis() as u32);
            }
        }

        self.suspended_event = susp;
    }

    fn process_transition(&mut self, transition: TransitionEvent<P::T>, ctx: &impl JamsocketContext) {
        self.state.apply(transition.clone());
        ctx.send_message(
            MessageRecipient::Broadcast,
            serde_json::to_string(&StateUpdateMessage::TransitionState::<P>(transition))
                .unwrap()
                .as_str(),
        );
        self.update_suspended_event(ctx);
    }

    fn check_and_process_transition(&mut self, user: u32, transition: TransitionEvent<P::T>, ctx: &impl JamsocketContext) {
        if transition.player != Some(PlayerID(user as usize)) {
            log::warn!(
                "Received a transition from a client with an invalid player ID. {:?} != {}",
                transition.player,
                user
            );
            return;
        }
        self.process_transition(transition, ctx);
    }
}

impl<P: StateProgram> SimpleJamsocketService for AperJamsocketService<P> {
    fn new(room_id: &str, ctx: &impl JamsocketContext) -> Self {
        let mut serv = AperJamsocketService {
            state: P::new(room_id),
            suspended_event: None
        };

        serv.update_suspended_event(ctx);

        serv
    }

    fn connect(&mut self, user: u32, ctx: &impl JamsocketContext) {
        ctx.send_message(
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

    fn disconnect(&mut self, _user: u32, _ctx: &impl JamsocketContext) {}

    fn message(&mut self, user: u32, message: &str, ctx: &impl JamsocketContext) {
        let transition: TransitionEvent<P::T> = serde_json::from_str(message).unwrap();
        self.check_and_process_transition(user, transition, ctx);
    }

    fn binary(&mut self, user: u32, message: &[u8], ctx: &impl JamsocketContext) {
        let transition: TransitionEvent<P::T> = bincode::deserialize(message).unwrap();
        self.check_and_process_transition(user, transition, ctx);
    }

    fn timer(&mut self, ctx: &impl JamsocketContext) {
        if let Some(event) = self.suspended_event.take() {
            self.process_transition(event, ctx);
            self.update_suspended_event(ctx);
        }
    }
}

pub struct AperJamsocketServiceBuilder<
    K: StateProgram,
    C: JamsocketContext,
> {
    ph_k: PhantomData<K>,
    ph_c: PhantomData<C>,
}

impl<
    K: StateProgram,
    C: JamsocketContext,
> Default for AperJamsocketServiceBuilder<K, C> {
    fn default() -> Self {
        AperJamsocketServiceBuilder {
            ph_k: Default::default(),
            ph_c: Default::default(),
        }
    }
}

impl<K: StateProgram, C: JamsocketContext> JamsocketServiceFactory<C> for AperJamsocketServiceBuilder<K, C> {
    type Service = WrappedJamsocketService<AperJamsocketService<K>, C>;

    fn build(&self, room_id: &str, context: C) -> Self::Service {
        let service = AperJamsocketService::new(room_id, &context);
        WrappedJamsocketService::new(service, context)
    }
}