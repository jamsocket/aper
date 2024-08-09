use aper::connection::{MessageToClient, MessageToServer, ServerConnection, ServerHandle};
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use state_program::{StateMachineContainerProgram, StateProgram};
pub use stateroom::ClientId;
use stateroom::{MessagePayload, StateroomContext, StateroomService};
use std::collections::HashMap;

mod state_program;

pub struct AperStateroomService<P: StateProgram> {
    connection: ServerConnection<P>,
    suspended_event: Option<IntentEvent<P::T>>,
    client_connections: HashMap<ClientId, ServerHandle<P>>,

    /// Pseudo-connection for sending timer events.
    timer_event_handle: ServerHandle<P>,
}

impl<P: StateProgram> Default for AperStateroomService<P> {
    fn default() -> Self {
        let mut connection = ServerConnection::new();
        let timer_event_handle = connection.connect(|_| {});

        AperStateroomService {
            connection,
            suspended_event: None,
            client_connections: HashMap::new(),
            timer_event_handle,
        }
    }
}

impl<P: StateProgram> AperStateroomService<P> {
    fn update_suspended_event(&mut self, ctx: &impl StateroomContext) {
        let susp = self.connection.state().suspended_event();
        if susp == self.suspended_event {
            return;
        }

        if let Some(ev) = &susp {
            let dur = ev.timestamp.signed_duration_since(Utc::now());
            ctx.set_timer(dur.num_milliseconds().max(0) as u32);
        }

        self.suspended_event = susp;
    }

    fn process_message(
        &mut self,
        message: MessageToServer,
        client_id: Option<ClientId>,
        ctx: &impl StateroomContext,
    ) {
        if let Some(handle) = client_id.and_then(|id| self.client_connections.get_mut(&id)) {
            handle.receive(&message);
        } else {
            self.timer_event_handle.receive(&message);
        }

        self.update_suspended_event(ctx);
    }
}

impl<P: StateProgram> StateroomService for AperStateroomService<P>
where
    P::T: Unpin + Send + Sync + 'static,
{
    fn init(&mut self, ctx: &impl StateroomContext) {
        self.update_suspended_event(ctx);
    }

    fn connect(&mut self, client_id: ClientId, ctx: &impl StateroomContext) {
        let ctx = Clone::clone(ctx);
        let callback = move |message: &MessageToClient| {
            ctx.send_message(client_id, serde_json::to_string(&message).unwrap().as_str());
        };

        let handle = self.connection.connect(callback);

        self.client_connections.insert(client_id, handle);
    }

    fn disconnect(&mut self, user: ClientId, _ctx: &impl StateroomContext) {
        self.client_connections.remove(&user);
    }

    fn message(
        &mut self,
        client_id: ClientId,
        message: MessagePayload,
        ctx: &impl StateroomContext,
    ) {
        match message {
            MessagePayload::Text(txt) => {
                let message: MessageToServer = serde_json::from_str(&txt).unwrap();
                self.process_message(message, Some(client_id), ctx);
            }
            MessagePayload::Bytes(bytes) => {
                let message: MessageToServer = bincode::deserialize(&bytes).unwrap();
                self.process_message(message, Some(client_id), ctx);
            }
        }
    }

    fn timer(&mut self, ctx: &impl StateroomContext) {
        if let Some(mut event) = self.suspended_event.take() {
            event.timestamp = Utc::now();
            let event = bincode::serialize(&event).unwrap();
            self.process_message(
                MessageToServer::Intent {
                    intent: event,
                    client_version: 0,
                },
                None,
                ctx,
            );
        }
    }
}

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IntentEvent<T>
where
    T: Unpin + Send + Sync + 'static + Clone,
{
    #[serde(with = "ts_milliseconds")]
    pub timestamp: Timestamp,
    pub client: Option<u32>,
    pub intent: T,
}

impl<T> IntentEvent<T>
where
    T: Unpin + Send + Sync + 'static + Clone,
{
    pub fn new(client: Option<u32>, timestamp: Timestamp, intent: T) -> IntentEvent<T> {
        IntentEvent {
            timestamp,
            client,
            intent,
        }
    }
}
