use crate::StateMachine;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Copy, Clone)]
pub struct StateVersionNumber(pub u32);

impl StateVersionNumber {
    pub fn prior_version(&self) -> StateVersionNumber {
        StateVersionNumber(self.0 - 1)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Copy, Clone)]
pub struct ClientTransitionNumber(pub u32);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageToServer<S: StateMachine> {
    DoTransition {
        transition_number: ClientTransitionNumber,
        transition: S::Transition,
    },
    RequestState,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageToClient<S>
where
    S: StateMachine,
{
    /// Set local state.
    /// Resets transition counter and empties local optimistic transitions.
    SetState {
        #[serde(bound = "")]
        state: S,
        version: StateVersionNumber,
    },

    /// Apply a transition made by a peer.
    PeerTransition {
        transition: S::Transition,
        version: StateVersionNumber,
    },

    /// Acknowledge a transition made by this replica.
    ConfirmTransition {
        transition_number: ClientTransitionNumber,
        version: StateVersionNumber,
    },

    /// State that a transition made by this replica caused a conflict and will
    /// not be processed.
    Conflict {
        transition_number: ClientTransitionNumber,
        conflict: S::Conflict,
    },
}