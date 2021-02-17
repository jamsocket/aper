use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::PlayerID;

/// A transition with associated metadata: which player triggered it and when.
/// The player ID is optional, since `SuspendedEvent`s do not have a player associated
/// with them.
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct TransitionEvent<Transition> {
    pub player_id: Option<PlayerID>,
    pub transition: Transition,
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

impl<Transition> TransitionEvent<Transition> {
    pub fn new(player_id: PlayerID, transition: Transition) -> TransitionEvent<Transition> {
        TransitionEvent {
            player_id: Some(player_id),
            transition,
            timestamp: Utc::now(),
        }
    }

    pub fn new_tick_event(transition: Transition) -> TransitionEvent<Transition> {
        TransitionEvent {
            player_id: None,
            transition,
            timestamp: Utc::now(),
        }
    }
}
