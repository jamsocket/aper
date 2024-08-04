use crate::{StateProgram, TransitionEvent};
use aper::connection::{ClientConnection, MessageToClient};
use chrono::{DateTime, Duration, Utc};
use stateroom::ClientId;

pub struct StateProgramClient<S: StateProgram> {
    client: ClientConnection<S>,
    pub client_id: ClientId,
    pub server_time_delta: Duration,
}

impl<S: StateProgram> StateProgramClient<S> {
    fn current_server_time(&self) -> DateTime<Utc> {
        Utc::now()
            .checked_sub_signed(self.server_time_delta)
            .unwrap()
    }

    pub fn state(&self) -> S {
        self.client.state()
    }

    fn wrap_intent(&self, intent: S::T) -> TransitionEvent<S::T> {
        let timestamp = self.current_server_time();

        TransitionEvent {
            client: Some(self.client_id),
            timestamp,
            intent,
        }
    }

    pub fn receive_message_from_server(&mut self, message: MessageToClient) {
        self.client.receive(&message);
    }

    pub fn push_intent(&mut self, intent: S::T) -> Result<(), S::Error> {
        let intent = self.wrap_intent(intent);
        self.client.apply(&intent);
        Ok(())
    }
}
