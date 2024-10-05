use crate::{Aper, AperClient, AperServer, IntentMetadata, Store};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    sync::{atomic::AtomicU32, Arc, Mutex},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageToServer {
    Intent {
        intent: Vec<u8>,
        client_version: u64,
        metadata: IntentMetadata,
    },
    RequestState {
        latest_version: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageToClientType {
    Apply {
        mutations: Vec<crate::Mutation>,
        client_version: Option<u64>,
        server_version: u64,
    },
    Hello {
        /// The client's assigned ID.
        client_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageToClient {
    pub message: MessageToClientType,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

pub struct ClientConnection<A: Aper> {
    client: AperClient<A>,
    message_callback: Box<dyn Fn(MessageToServer)>,
    client_id: Option<u32>,
}

impl<A: Aper> ClientConnection<A> {
    pub fn new<F: Fn(MessageToServer) + 'static>(
        client: AperClient<A>,
        message_callback: F,
    ) -> Self {
        // Request initial state.

        let init_message = MessageToServer::RequestState { latest_version: 0 };

        (message_callback)(init_message);

        Self {
            client,
            message_callback: Box::new(message_callback),
            client_id: None,
        }
    }

    pub fn client_id(&self) -> Option<u32> {
        self.client_id
    }

    pub fn state(&self) -> A {
        self.client.state()
    }

    pub fn store(&self) -> Store {
        self.client.store()
    }

    /// Send an intent to the server, and apply it speculatively to the local state.
    pub fn apply(&mut self, intent: A::Intent) -> Result<(), A::Error> {
        let metadata = IntentMetadata::new(self.client_id, Utc::now());
        let version = self.client.apply(&intent, &metadata)?;
        let intent = bincode::serialize(&intent).unwrap();
        (self.message_callback)(MessageToServer::Intent {
            intent,
            client_version: version,
            metadata,
        });

        Ok(())
    }

    pub fn receive(&mut self, message: &MessageToClient) {
        match &message.message {
            MessageToClientType::Apply {
                mutations,
                client_version: version,
                server_version,
            } => {
                self.client.mutate(mutations, *version, *server_version);
            }
            MessageToClientType::Hello { client_id } => {
                self.client_id = Some(*client_id);
            }
        }
    }
}

pub struct ServerConnection<A: Aper> {
    callbacks: Arc<DashMap<u32, Box<dyn Fn(&MessageToClient) + Send + Sync>>>,
    server: Arc<Mutex<AperServer<A>>>,
    next_client_id: AtomicU32,
}

impl<A: Aper> Default for ServerConnection<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Aper> ServerConnection<A> {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(DashMap::new()),
            server: Arc::new(Mutex::new(AperServer::new())),
            next_client_id: AtomicU32::new(0),
        }
    }

    pub fn connect<F: Fn(&MessageToClient) + Send + Sync + 'static>(
        &mut self,
        callback: F,
    ) -> ServerHandle<A> {
        let client_id = self
            .next_client_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        (callback)(&MessageToClient {
            message: MessageToClientType::Hello { client_id },
            timestamp: Utc::now(),
        });

        self.callbacks.insert(client_id, Box::new(callback));

        ServerHandle {
            server: self.server.clone(),
            client_id,
            callbacks: self.callbacks.clone(),
        }
    }

    pub fn state(&self) -> A {
        self.server.lock().unwrap().state()
    }
}

pub struct ServerHandle<A: Aper> {
    client_id: u32,
    server: Arc<Mutex<AperServer<A>>>,
    callbacks: Arc<DashMap<u32, Box<dyn Fn(&MessageToClient) + Send + Sync>>>,
}

impl<A: Aper> ServerHandle<A> {
    pub fn receive(&mut self, message: &MessageToServer) {
        match message {
            MessageToServer::Intent {
                intent,
                client_version,
                metadata,
            } => {
                let intent = bincode::deserialize(intent).unwrap();
                let mut server_borrow = self.server.lock().unwrap();
                let Ok(mutations) = server_borrow.apply(&intent, &metadata) else {
                    // still need to ack the client.

                    if let Some(callback) = self.callbacks.get(&self.client_id) {
                        let time = Utc::now();
                        let message = MessageToClient {
                            message: MessageToClientType::Apply {
                                mutations: vec![],
                                client_version: Some(*client_version),
                                server_version: server_borrow.version(),
                            },
                            timestamp: time,
                        };

                        callback(&message);
                    }

                    return;
                };

                let version = server_borrow.version();
                let time = Utc::now();

                let message_to_others = MessageToClient {
                    message: MessageToClientType::Apply {
                        mutations: mutations.clone(),
                        client_version: None,
                        server_version: version,
                    },
                    timestamp: time,
                };

                let message_to_sender = MessageToClient {
                    message: MessageToClientType::Apply {
                        mutations: mutations.clone(),
                        client_version: Some(*client_version),
                        server_version: version,
                    },
                    timestamp: time,
                };

                for entry in self.callbacks.iter() {
                    let (other_client_id, callback) = entry.pair();
                    if *other_client_id == self.client_id {
                        callback(&message_to_sender);
                    } else {
                        callback(&message_to_others);
                    }
                }
            }
            MessageToServer::RequestState { .. } => {
                let server = self.server.lock().unwrap();
                let c = server.borrow();
                let mutations = c.state_snapshot();

                if let Some(callback) = self.callbacks.get(&self.client_id) {
                    let time = Utc::now();
                    let message = MessageToClient {
                        message: MessageToClientType::Apply {
                            mutations,
                            client_version: None,
                            server_version: c.version(),
                        },
                        timestamp: time,
                    };

                    callback(&message);
                }
            }
        }
    }
}

impl<A: Aper> Drop for ServerHandle<A> {
    fn drop(&mut self) {
        self.callbacks.remove(&self.client_id);
    }
}
