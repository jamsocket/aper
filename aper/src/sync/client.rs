use super::messages::{
    ClientTransitionNumber, MessageToClient, MessageToServer, StateVersionNumber,
};
use crate::StateMachine;
use std::{collections::VecDeque, rc::Rc};

#[derive(Debug, Clone)]
struct OptimisticState<S: StateMachine> {
    transition_number: ClientTransitionNumber,
    transition: S::Transition,
    state: Rc<S>,
}

#[derive(Default, Debug, Clone)]
pub struct StateClient<S: StateMachine> {
    golden_state: Rc<S>,
    optimistic_states: VecDeque<OptimisticState<S>>,
    version: StateVersionNumber,
    next_transition: ClientTransitionNumber,
}

impl<S: StateMachine> StateClient<S> {
    pub fn new(state: S, version: StateVersionNumber) -> Self {
        StateClient {
            golden_state: Rc::new(state),
            optimistic_states: VecDeque::new(),
            version,
            next_transition: ClientTransitionNumber::default(),
        }
    }

    pub fn push_transition(
        &mut self,
        transition: S::Transition,
    ) -> Result<MessageToServer<S>, S::Conflict> {
        let current_state = self.state();
        let state = current_state.apply(&transition)?;

        let transition_number = self.next_transition();

        let optimistic_state = OptimisticState {
            transition: transition.clone(),
            state: Rc::new(state),
            transition_number,
        };

        self.optimistic_states.push_back(optimistic_state);

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
    ) -> Result<(), S::Conflict> {
        match message {
            MessageToClient::SetState { state, version } => {
                self.golden_state = Rc::new(state);
                self.optimistic_states = VecDeque::default();
                self.version = version;
                Ok(())
            }

            MessageToClient::ConfirmTransition {
                transition_number,
                version,
            } => {
                if let Some(OptimisticState {
                    transition_number: optimistic_transition_number,
                    state,
                    ..
                }) = self.optimistic_states.pop_front()
                {
                    // TODO: this is recoverable, but panicking until we have a logging solution because we want to know about it.
                    assert_eq!(
                        optimistic_transition_number,
                        transition_number,
                        "Expected response about transition {:?} but got response about transition {:?}",
                        optimistic_transition_number,
                        transition_number
                    );

                    self.golden_state = state;
                    self.version = version;

                    Ok(())
                } else {
                    panic!(
                        "Remote confirmed transition {:?} but we don't have a record of it.",
                        transition_number
                    );
                }
            }

            MessageToClient::Conflict {
                transition_number,
                conflict,
            } => {
                if let Some(OptimisticState {
                    transition_number: optimistic_transition_number,
                    ..
                }) = self.optimistic_states.pop_front()
                {
                    // TODO: this is recoverable, but panicking until we have a logging solution because we want to know about it.
                    assert_eq!(
                        optimistic_transition_number,
                        transition_number,
                        "Expected response about transition {:?} but got response about transition {:?}",
                        optimistic_transition_number,
                        transition_number
                    );

                    Err(conflict)
                } else {
                    panic!(
                        "Remote rejected transition {:?} but we don't have a record of it.",
                        transition_number
                    );
                }
            }

            MessageToClient::PeerTransition {
                transition,
                version,
            } => {
                assert_eq!(
                    self.version,
                    version.prior_version(),
                    "Client has version {:?} but transition implies version {:?}",
                    self.version,
                    version.prior_version()
                );
                self.golden_state = Rc::new(self.golden_state.apply(&transition).unwrap());
                self.version = version;

                let mut state = &self.golden_state;
                for optimistic_state in self.optimistic_states.iter_mut() {
                    optimistic_state.state = state
                        .apply(&optimistic_state.transition)
                        .map(Rc::new)
                        .unwrap_or_else(|_| state.clone());
                    state = &optimistic_state.state;
                }

                Ok(())
            }
        }
    }

    pub fn state(&self) -> Rc<S> {
        if let Some(v) = self.optimistic_states.back() {
            v.state.clone()
        } else {
            self.golden_state.clone()
        }
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

        m1.receive_message_from_server(MessageToClient::SetState {
            state: counter,
            version: StateVersionNumber(0),
        })
        .unwrap();

        assert_eq!(0, m1.state().value());

        m1.push_transition(Counter::increment(4)).unwrap();

        assert_eq!(4, m1.state().value());
    }
}
