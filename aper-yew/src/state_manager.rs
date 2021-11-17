use aper_jamsocket::{StateProgram, Timestamp, TransitionEvent};
use chrono::Utc;
use std::fmt::Debug;

/// A container for the local copy of the state. Maintains an estimate of the
/// time on the server.
#[derive(Debug)]
pub struct StateManager<State: StateProgram> {
    /// The client's latest up-to-date snapshot
    golden_state: Box<State>,
    /// The client's optimistic projection of the latest up-to-date snapshot
    optimistic_state: Box<State>,
    sent_transition: Option<TransitionEvent<State::T>>,
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
            golden_state: Box::new(state.clone()),
            optimistic_state: Box::new(state),
            sent_transition: None,
            last_server_time: server_time,
            last_local_time: Utc::now(),
        }
    }

    /// Process an event that originated at this client.
    /// Returns `true` if the transition resulted in an optimistic state change.
    pub fn process_local_event(&mut self, event: TransitionEvent<State::T>) -> bool {
        // if sent_transition is Some(_), do nothing.
        // otherwise
        // - apply event to optimistic_state
        // - store event in sent_transition
        if self.sent_transition.is_none() {
            if self.optimistic_state.apply(event.clone()).is_err() {
                return false;
            }
            self.sent_transition = Some(event);
            true
        } else {
            false
        }
    }

    /// Process an event that came from the server
    pub fn process_remote_event(&mut self, event: TransitionEvent<State::T>) {
        // if sent_transition is None, same behavior as before
        // otherwise:
        // - if sent_transition is NOT the same as event:
        //   - apply event to golden_state
        //   - clone golden_state as optimistic_state
        //   - reset sent_transition

        self.last_local_time = Utc::now();
        self.last_server_time = event.timestamp;
        self.golden_state
            .apply(event.clone())
            .expect("Message from server caused conflict.");

        match &self.sent_transition {
            Some(transition) => {
                if *transition != event {
                    self.optimistic_state = self.golden_state.clone();
                }

                self.sent_transition = None;
            }
            None => {
                self.optimistic_state = self.golden_state.clone();
            }
        }
    }

    // We don't want to expose the golden state
    // As far as the caller is concerned, the optimistic state *is* the golden state
    pub fn get_state(&self) -> &State {
        &self.optimistic_state
    }
}
