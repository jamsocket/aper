use super::messages::{
    ClientTransitionNumber, MessageToClient, MessageToServer, StateVersionNumber,
};
use crate::StateMachine;
use std::{collections::VecDeque, rc::Rc};

#[derive(Debug, Clone)]
pub struct StateClient<S: StateMachine> {
    golden_state: Rc<S>,
    transitions: VecDeque<(ClientTransitionNumber, S::Transition)>,
    optimistic_state: Rc<S>,
    version: StateVersionNumber,
    next_transition: ClientTransitionNumber,
}

impl<S: StateMachine + Default> Default for StateClient<S> {
    fn default() -> Self {
        Self {
            golden_state: Default::default(),
            transitions: Default::default(),
            optimistic_state: Default::default(),
            version: Default::default(),
            next_transition: Default::default(),
        }
    }
}

impl<S: StateMachine> StateClient<S> {
    pub fn new(state: S, version: StateVersionNumber) -> Self {
        let state = Rc::new(state);
        StateClient {
            golden_state: state.clone(),
            optimistic_state: state,
            transitions: VecDeque::new(),
            version,
            next_transition: ClientTransitionNumber::default(),
        }
    }

    pub fn push_transition(
        &mut self,
        transition: S::Transition,
    ) -> Result<MessageToServer<S>, S::Conflict> {
        let current_state = self.state();
        self.optimistic_state = Rc::new(current_state.apply(&transition)?);

        let transition_number = self.next_transition();
        self.transitions
            .push_back((transition_number, transition.clone()));

        Ok(MessageToServer::DoTransition {
            transition_number,
            transition,
        })
    }

    pub fn next_transition(&mut self) -> ClientTransitionNumber {
        let result = self.next_transition;
        self.next_transition.0 += 1;

        result
    }

    pub fn receive_message_from_server(
        &mut self,
        message: MessageToClient<S>,
    ) -> Option<MessageToServer<S>> {
        match message {
            MessageToClient::SetState { state, version } => {
                let state = Rc::new(state);
                self.golden_state = state.clone();
                self.optimistic_state = state;
                self.transitions = VecDeque::new(); // Don't replay transitions, for now?
                self.version = version;
                None
            }

            MessageToClient::ConfirmTransition {
                transition_number,
                version,
            } => {
                if let Some((optimistic_transition_number, transition)) =
                    self.transitions.pop_front()
                {
                    if optimistic_transition_number != transition_number {
                        // Remote confirmed a transition out of expected order.
                        return Some(MessageToServer::RequestState);
                    }

                    if let Ok(state) = self.golden_state.apply(&transition) {
                        self.golden_state = Rc::new(state);
                    } else {
                        // A transition confirmed by the server shouldn't create a conflict,
                        // so something has drifted.
                        return Some(MessageToServer::RequestState);
                    }
                    self.version = version;

                    None
                } else {
                    // Remote confirmed a transition but we don't have any local transitions.
                    Some(MessageToServer::RequestState)
                }
            }

            MessageToClient::Conflict {
                transition_number,
                ..
            } => {
                if let Some((optimistic_transition_number, _)) =
                    self.transitions.pop_front()
                {
                    if optimistic_transition_number != transition_number {
                        return Some(MessageToServer::RequestState);
                    }

                    // We've popped the transition that caused a conflict, nothing more to do.
                    None
                } else {
                    Some(MessageToServer::RequestState)
                }
            }

            MessageToClient::PeerTransition {
                transition,
                version,
            } => {
                if self.version != version.prior_version() {
                    return Some(MessageToServer::RequestState);
                }

                let state = if let Ok(state) = self.golden_state.apply(&transition) {
                    state
                } else {
                    // Applying state locally caused conflict.
                    return Some(MessageToServer::RequestState);
                };

                self.golden_state = Rc::new(state);
                self.version = version;

                let mut state = self.golden_state.clone();
                for (_, transition) in &self.transitions {
                    if let Ok(st) = state.apply(&transition) {
                        state = Rc::new(st);
                    };
                }

                self.optimistic_state = state;

                None
            }
        }
    }

    pub fn state(&self) -> Rc<S> {
        self.optimistic_state.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data_structures::Counter;

    #[test]
    fn test_simple_state() {
        let counter = Counter::default();
        let mut m1 = StateClient::<Counter>::default();

        assert!(m1.receive_message_from_server(MessageToClient::SetState {
            state: counter,
            version: StateVersionNumber(0),
        }).is_none());

        assert_eq!(0, m1.state().value());

        m1.push_transition(Counter::increment(4)).unwrap();

        assert_eq!(4, m1.state().value());
    }
}
