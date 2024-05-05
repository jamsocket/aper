use aper::sync::messages::{
    ClientTransitionNumber, MessageToClient, MessageToServer, StateVersionNumber,
};
use aper::sync::server::{StateServer, StateServerMessageResponse};
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use state_program::{StateMachineContainerProgram, StateProgram};
pub use state_program_client::StateProgramClient;
pub use stateroom::ClientId;
use stateroom::{MessagePayload, MessageRecipient, StateroomContext, StateroomService};

mod state_program;
mod state_program_client;

#[derive(Serialize, Deserialize, Debug)]
pub enum StateProgramMessage<S>
where
    S: StateProgram,
{
    InitialState {
        #[serde(with = "ts_milliseconds")]
        timestamp: DateTime<Utc>,
        client_id: ClientId,
        #[serde(bound = "")]
        state: S,
        version: StateVersionNumber,
    },
    Message {
        #[serde(bound = "")]
        message: MessageToClient<S>,
        #[serde(with = "ts_milliseconds")]
        timestamp: DateTime<Utc>,
    },
}

pub struct AperStateroomService<P: StateProgram + Default> {
    state: StateServer<P>,
    suspended_event: Option<TransitionEvent<P::T>>,
}

impl<P: StateProgram + Default> Default for AperStateroomService<P> {
    fn default() -> Self {
        AperStateroomService {
            state: StateServer::default(),
            suspended_event: None,
        }
    }
}

impl<P: StateProgram + Default> AperStateroomService<P> {
    fn update_suspended_event(&mut self, ctx: &impl StateroomContext) {
        let susp = self.state.state().suspended_event();
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

    fn process_message(
        &mut self,
        message: MessageToServer<P>,
        client_id: Option<ClientId>,
        ctx: &impl StateroomContext,
    ) {
        if let MessageToServer::DoTransition { transition, .. } = &message {
            if transition.client != client_id {
                log::warn!(
                    "Received a transition from a client with an invalid player ID. {:?} != {:?}",
                    transition.client,
                    client_id
                );
                return;
            }
        }

        let timestamp = Utc::now();
        let StateServerMessageResponse {
            reply_message,
            broadcast_message,
        } = self.state.receive_message(message);

        let reply_message = StateProgramMessage::Message {
            message: reply_message,
            timestamp,
        };

        if let Some(client_id) = client_id {
            ctx.send_message(
                MessageRecipient::Client(client_id),
                serde_json::to_string(&reply_message).unwrap().as_str(),
            );
        }

        if let Some(broadcast_message) = broadcast_message {
            let broadcast_message = StateProgramMessage::Message {
                message: broadcast_message,
                timestamp,
            };

            let recipient = if let Some(client_id) = client_id {
                MessageRecipient::EveryoneExcept(client_id)
            } else {
                MessageRecipient::Broadcast
            };

            ctx.send_message(
                recipient,
                serde_json::to_string(&broadcast_message).unwrap().as_str(),
            );
        }

        self.update_suspended_event(ctx);
    }
}

impl<P: StateProgram + Default> StateroomService for AperStateroomService<P>
where
    P::T: Unpin + Send + Sync + 'static,
{
    fn connect(&mut self, client_id: ClientId, ctx: &impl StateroomContext) {
        let response = StateProgramMessage::InitialState {
            timestamp: Utc::now(),
            client_id,
            state: self.state.state().clone(),
            version: self.state.version,
        };

        ctx.send_message(
            MessageRecipient::Client(client_id),
            serde_json::to_string(&response).unwrap().as_str(),
        );
    }

    fn disconnect(&mut self, _user: ClientId, _ctx: &impl StateroomContext) {}

    fn message(
        &mut self,
        client_id: ClientId,
        message: MessagePayload,
        ctx: &impl StateroomContext,
    ) {
        match message {
            MessagePayload::Text(txt) => {
                let message: MessageToServer<P> = serde_json::from_str(&txt).unwrap();
                self.process_message(message, Some(client_id), ctx);
            }
            MessagePayload::Bytes(bytes) => {
                let message: MessageToServer<P> = bincode::deserialize(&bytes).unwrap();
                self.process_message(message, Some(client_id), ctx);
            }
        }
    }

    fn timer(&mut self, ctx: &impl StateroomContext) {
        if let Some(event) = self.suspended_event.take() {
            self.process_message(
                MessageToServer::DoTransition {
                    transition_number: ClientTransitionNumber::default(),
                    transition: event,
                },
                None,
                ctx,
            );
        }
    }
}

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransitionEvent<T>
where
    T: Unpin + Send + Sync + 'static + Clone,
{
    #[serde(with = "ts_milliseconds")]
    pub timestamp: Timestamp,
    pub client: Option<ClientId>,
    pub transition: T,
}

impl<T> TransitionEvent<T>
where
    T: Unpin + Send + Sync + 'static + Clone,
{
    pub fn new(
        player: Option<ClientId>,
        timestamp: Timestamp,
        transition: T,
    ) -> TransitionEvent<T> {
        TransitionEvent {
            timestamp,
            client: player,
            transition,
        }
    }
}
