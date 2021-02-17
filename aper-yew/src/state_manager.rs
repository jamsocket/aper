use aper::{StateMachine, TransitionEvent};
use chrono::{DateTime, Utc};
use std::fmt::Debug;

#[derive(Debug)]
pub struct StateManager<State: StateMachine> {
    state: Box<State>,
    last_server_time: DateTime<Utc>,
    last_local_time: DateTime<Utc>,
}

impl<State: StateMachine> StateManager<State> {
    /// Estimates the current time on the server, by taking the server time of the
    /// last message the server sent and adding the local time that has passed
    /// since receiving that message.
    pub fn get_estimated_server_time(&self) -> DateTime<Utc> {
        let elapsed = Utc::now().signed_duration_since(self.last_local_time);
        self.last_server_time + elapsed
    }

    pub fn new(state: State, server_time: DateTime<Utc>) -> StateManager<State> {
        StateManager {
            state: Box::new(state),
            last_server_time: server_time,
            last_local_time: Utc::now(),
        }
    }

    pub fn process_event(&mut self, event: TransitionEvent<<State as StateMachine>::Transition>) {
        self.last_local_time = Utc::now();
        self.last_server_time = event.timestamp;

        self.state.process_event(event);
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }
}
