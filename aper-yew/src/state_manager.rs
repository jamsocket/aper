use aper::{StateProgram, Timestamp, TransitionEvent};
use chrono::Utc;
use std::fmt::Debug;

#[derive(Debug)]
pub struct StateManager<State: StateProgram> {
    state: Box<State>,
    last_server_time: Timestamp,
    last_local_time: Timestamp,
}

impl<State: StateProgram> StateManager<State> {
    /// Estimates the current time on the server, by taking the server time of the
    /// last message the server sent and adding the local time that has passed
    /// since receiving that message.
    pub fn get_estimated_server_time(&self) -> Timestamp {
        let elapsed = Utc::now().signed_duration_since(self.last_local_time);
        self.last_server_time + elapsed
    }

    pub fn new(state: State, server_time: Timestamp) -> StateManager<State> {
        StateManager {
            state: Box::new(state),
            last_server_time: server_time,
            last_local_time: Utc::now(),
        }
    }

    pub fn process_event(&mut self, event: TransitionEvent<<State as StateProgram>::Transition>) {
        self.last_local_time = Utc::now();
        self.last_server_time = event.timestamp;

        self.state.apply(event);
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }
}
