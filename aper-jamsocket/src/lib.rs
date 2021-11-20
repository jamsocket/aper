use std::{collections::HashMap, convert::Infallible, marker::PhantomData};

use aper::{PlayerID, StateProgram, StateUpdateMessage, TransitionEvent};
use chrono::Utc;
use jamsocket::{ClientId, JamsocketContext, JamsocketServiceFactory, MessageRecipient, SimpleJamsocketService, WrappedJamsocketService};

pub struct AperJamsocketService<P: StateProgram> {
    state: P,
    suspended_event: Option<TransitionEvent<P::T>>,
    client_to_player: HashMap<ClientId, PlayerID>,
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

    fn check_and_process_transition(&mut self, client_id: ClientId, transition: TransitionEvent<P::T>, ctx: &impl JamsocketContext) {
        let user = if let Some(user) = self.client_to_player.get(&client_id) {
            user
        } else {
            return
        };

        if transition.player != Some(*user) {
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
            suspended_event: None,
            client_to_player: HashMap::default(),
        };

        serv.update_suspended_event(ctx);

        serv
    }

    fn connect(&mut self, client_id: ClientId, ctx: &impl JamsocketContext) {
        let player_id = PlayerID(self.client_to_player.len());
        self.client_to_player.insert(client_id, player_id);

        ctx.send_message(
            MessageRecipient::Client(client_id),
            serde_json::to_string(&StateUpdateMessage::ReplaceState::<P>(
                self.state.clone(),
                Utc::now(),
                player_id,
            ))
            .unwrap()
            .as_str(),
        );
    }

    fn disconnect(&mut self, _user: ClientId, _ctx: &impl JamsocketContext) {}

    fn message(&mut self, user: ClientId, message: &str, ctx: &impl JamsocketContext) {
        let transition: TransitionEvent<P::T> = serde_json::from_str(message).unwrap();
        self.check_and_process_transition(user, transition, ctx);
    }

    fn binary(&mut self, user: ClientId, message: &[u8], ctx: &impl JamsocketContext) {
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
    type Error = Infallible;

    fn build(&self, room_id: &str, context: C) -> Result<Self::Service, Self::Error> {
        let service = AperJamsocketService::new(room_id, &context);
        Ok(WrappedJamsocketService::new(service, context))
    }
}