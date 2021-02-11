use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::PlayerID;

/// A transition with associated metadata: which player triggered it and when.
/// The player ID is optional, since `SuspendedEvent`s do not have a player associated
/// with them.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionEvent<Transition> {
    pub player_id: Option<PlayerID>,
    pub transition: Transition,
    pub timestamp: SystemTime,
}

impl<Transition> TransitionEvent<Transition> {
    pub fn new(player_id: PlayerID, transition: Transition) -> TransitionEvent<Transition> {
        TransitionEvent {
            player_id: Some(player_id),
            transition,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn new_tick_event(transition: Transition) -> TransitionEvent<Transition> {
        TransitionEvent {
            player_id: None,
            transition,
            timestamp: std::time::SystemTime::now(),
        }
    }
}
