use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
pub use jamsocket::ClientId;
use jamsocket::{
    JamsocketContext, JamsocketServiceFactory, MessageRecipient, SimpleJamsocketService,
    WrappedJamsocketService,
};
use serde::{Deserialize, Serialize};
pub use state_program::{StateMachineContainerProgram, StateProgram};
use std::convert::Infallible;
use std::marker::PhantomData;

mod state_program;

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

    fn process_transition(
        &mut self,
        transition: TransitionEvent<P::T>,
        ctx: &impl JamsocketContext,
    ) {
        self.state.apply(transition.clone()).unwrap();
        ctx.send_message(
            MessageRecipient::Broadcast,
            serde_json::to_string(&StateUpdateMessage::TransitionState::<P>(transition))
                .unwrap()
                .as_str(),
        );
        self.update_suspended_event(ctx);
    }

    fn check_and_process_transition(
        &mut self,
        client_id: ClientId,
        transition: TransitionEvent<P::T>,
        ctx: &impl JamsocketContext,
    ) {
        if transition.player != Some(client_id) {
            log::warn!(
                "Received a transition from a client with an invalid player ID. {:?} != {:?}",
                transition.player,
                client_id
            );
            return;
        }
        self.process_transition(transition, ctx);
    }
}

impl<P: StateProgram> SimpleJamsocketService for AperJamsocketService<P>
    where P::T: Unpin + Send + Sync + 'static
{
    fn new(room_id: &str, ctx: &impl JamsocketContext) -> Self {
        let mut serv = AperJamsocketService {
            state: P::new(room_id),
            suspended_event: None,
        };

        serv.update_suspended_event(ctx);

        serv
    }

    fn connect(&mut self, client_id: ClientId, ctx: &impl JamsocketContext) {
        ctx.send_message(
            MessageRecipient::Client(client_id),
            serde_json::to_string(&StateUpdateMessage::ReplaceState::<P>(
                self.state.clone(),
                Utc::now(),
                client_id,
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

pub struct AperJamsocketServiceBuilder<K: StateProgram, C: JamsocketContext> {
    ph_k: PhantomData<K>,
    ph_c: PhantomData<C>,
}

impl<K: StateProgram, C: JamsocketContext> Default for AperJamsocketServiceBuilder<K, C> {
    fn default() -> Self {
        AperJamsocketServiceBuilder {
            ph_k: Default::default(),
            ph_c: Default::default(),
        }
    }
}

impl<K: StateProgram, C: JamsocketContext> JamsocketServiceFactory<C>
    for AperJamsocketServiceBuilder<K, C>
    where K::T: Unpin + Send + Sync + 'static
{
    type Service = WrappedJamsocketService<AperJamsocketService<K>, C>;
    type Error = Infallible;

    fn build(&self, room_id: &str, context: C) -> Result<Self::Service, Infallible> {
        let service = AperJamsocketService::new(room_id, &context);
        Ok(WrappedJamsocketService::new(service, context))
    }
}

/// A message from the server to a client that tells it to update its state.
#[derive(Serialize, Deserialize, Debug)]
pub enum StateUpdateMessage<State: StateProgram> 
where State::T: Unpin + Send + Sync + 'static + Clone
{
    /// Instructs the client to completely discard its existing state and replace it
    /// with the provided one. This is currently only used to set the initial state
    /// when a client first connects.
    ReplaceState(
        #[serde(bound = "")] State,
        #[serde(with = "ts_milliseconds")] Timestamp,
        ClientId,
    ),

    /// Instructs the client to apply the given [TransitionEvent] to its copy of
    /// the state to synchronize it with the server. All state updates
    /// after the initial state is sent are sent through [StateUpdateMessage::TransitionState].
    TransitionState(#[serde(bound = "")] TransitionEvent<State::T>),
}

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransitionEvent<T>
where T: Unpin + Send + Sync + 'static + Clone
{
    #[serde(with = "ts_milliseconds")]
    pub timestamp: Timestamp,
    pub player: Option<ClientId>,
    pub transition: T,
}

impl<T> TransitionEvent<T>
where T: Unpin + Send + Sync + 'static + Clone
{
    pub fn new(
        player: Option<ClientId>,
        timestamp: Timestamp,
        transition: T,
    ) -> TransitionEvent<T> {
        TransitionEvent {
            timestamp,
            player,
            transition,
        }
    }
}
