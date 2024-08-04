use super::messages::{MessageToClient, MessageToServer, StateVersionNumber};
use crate::StateMachine;

#[derive(Default)]
pub struct StateServer<S: StateMachine> {
    pub version: StateVersionNumber,
    state: S,
}

pub struct StateServerMessageResponse<S: StateMachine> {
    pub reply_message: MessageToClient<S>,
    pub broadcast_message: Option<MessageToClient<S>>,
}

impl<S: StateMachine> StateServer<S> {
    pub fn new(state: S) -> Self {
        StateServer {
            version: StateVersionNumber::default(),
            state,
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn receive_message(
        &mut self,
        message: MessageToServer<S>,
    ) -> StateServerMessageResponse<S> {
        match message {
            MessageToServer::DoTransition {
                transition_number,
                transition,
            } => match self.state.apply(&transition) {
                Ok(state) => {
                    self.state = state;
                    self.version.0 += 1;

                    StateServerMessageResponse {
                        reply_message: MessageToClient::ConfirmTransition {
                            transition_number,
                            version: self.version,
                        },
                        broadcast_message: Some(MessageToClient::PeerTransition {
                            transition,
                            version: self.version,
                        }),
                    }
                }
                Err(_) => todo!(),
            },
            MessageToServer::RequestState => StateServerMessageResponse {
                reply_message: MessageToClient::SetState {
                    state: self.state.clone(),
                    version: self.version,
                },
                broadcast_message: None,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{data_structures::Counter, sync::messages::ClientTransitionNumber};

    #[test]
    pub fn basic_messages() {
        let counter = Counter::new(110);
        let mut server: StateServer<Counter> = StateServer::new(counter);

        let result = server.receive_message(MessageToServer::RequestState);

        if let StateServerMessageResponse {
            reply_message: MessageToClient::SetState { state, version },
            broadcast_message: None,
        } = result
        {
            assert_eq!(0, version.0);
            assert_eq!(110, state.value());
        } else {
            panic!("Response did not match pattern.");
        }

        let result = server.receive_message(MessageToServer::DoTransition {
            transition_number: ClientTransitionNumber(1),
            transition: Counter::increment(3),
        });

        if let StateServerMessageResponse {
            reply_message:
                MessageToClient::ConfirmTransition {
                    transition_number: ClientTransitionNumber(1),
                    version: StateVersionNumber(1),
                },
            broadcast_message:
                Some(MessageToClient::PeerTransition {
                    transition,
                    version: StateVersionNumber(1),
                }),
        } = result
        {
            assert_eq!(Counter::increment(3), transition);
        } else {
            panic!("Response did not match pattern.");
        }

        assert_eq!(113, server.state.value());
        assert_eq!(1, server.version.0);
    }

    // TODO: test conflict case.
}
