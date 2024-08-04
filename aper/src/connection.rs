use crate::{Aper, AperClient, AperServer};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    cell::RefCell,
    sync::{atomic::AtomicU64, Arc, Mutex},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageToServer {
    Intent {
        intent: Vec<u8>,
        client_version: u64,
    },
    RequestState {
        latest_version: u64,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum MessageToClient {
    Apply {
        mutations: Vec<crate::Mutation>,
        client_version: Option<u64>,
        server_version: u64,
    },
}

pub struct ClientConnection<A: Aper> {
    client: AperClient<A>,
    message_callback: Box<dyn Fn(MessageToServer)>,
    state_callback: Box<dyn Fn(A)>,
}

impl<A: Aper> ClientConnection<A> {
    pub fn new<F: Fn(MessageToServer) + 'static, FS: Fn(A) + 'static>(
        client: AperClient<A>,
        message_callback: F,
        state_callback: FS,
    ) -> Self {
        Self {
            client,
            message_callback: Box::new(message_callback),
            state_callback: Box::new(state_callback),
        }
    }

    pub fn state(&self) -> A {
        self.client.state()
    }

    pub fn apply(&mut self, intent: &A::Intent) -> Result<(), A::Error> {
        let version = self.client.apply(&intent)?;
        let intent = bincode::serialize(intent).unwrap();
        (self.message_callback)(MessageToServer::Intent {
            intent,
            client_version: version,
        });

        (self.state_callback)(self.client.state());

        Ok(())
    }

    pub fn receive(&mut self, message: &MessageToClient) {
        match message {
            MessageToClient::Apply {
                mutations,
                client_version: version,
                server_version,
            } => {
                self.client.mutate(mutations, *version, *server_version);

                (self.state_callback)(self.client.state());
            }
        }
    }
}

pub struct ServerConnection<A: Aper> {
    callbacks: Arc<DashMap<u64, Box<dyn Fn(&MessageToClient) + Send + Sync>>>,
    server: Arc<Mutex<AperServer<A>>>,
    next_client_id: AtomicU64,
}

impl<A: Aper> ServerConnection<A> {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(DashMap::new()),
            server: Arc::new(Mutex::new(AperServer::new())),
            next_client_id: AtomicU64::new(0),
        }
    }
}

impl<A: Aper> ServerConnection<A> {
    pub fn connect<F: Fn(&MessageToClient) + Send + Sync + 'static>(
        &mut self,
        callback: F,
    ) -> ServerHandle<A> {
        let client_id = self
            .next_client_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.callbacks.insert(client_id, Box::new(callback));
        ServerHandle {
            server: self.server.clone(),
            client_id,
            callbacks: self.callbacks.clone(),
        }
    }
}

pub struct ServerHandle<A: Aper> {
    client_id: u64,
    server: Arc<Mutex<AperServer<A>>>,
    callbacks: Arc<DashMap<u64, Box<dyn Fn(&MessageToClient) + Send + Sync>>>,
}

impl<A: Aper> ServerHandle<A> {
    pub fn receive(&mut self, message: &MessageToServer) {
        match message {
            MessageToServer::Intent {
                intent,
                client_version,
            } => {
                let intent = bincode::deserialize(intent).unwrap();
                let mut server_borrow = self.server.lock().unwrap();
                let Ok(mutations) = server_borrow.apply(&intent) else {
                    // still need to ack the client.

                    self.callbacks.get(&self.client_id).map(|callback| {
                        callback(&MessageToClient::Apply {
                            mutations: vec![],
                            client_version: Some(*client_version),
                            server_version: server_borrow.version(),
                        });
                    });

                    return;
                };

                let version = server_borrow.version();

                let message_to_others = MessageToClient::Apply {
                    mutations: mutations.clone(),
                    client_version: None,
                    server_version: version,
                };

                let message_to_sender = MessageToClient::Apply {
                    mutations: mutations.clone(),
                    client_version: Some(*client_version),
                    server_version: version,
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
            MessageToServer::RequestState { latest_version } => {
                let server = self.server.lock().unwrap();
                let c = server.borrow();
                let mutations = c.state_snapshot();

                self.callbacks.get(&self.client_id).map(|callback| {
                    callback(&MessageToClient::Apply {
                        mutations,
                        client_version: None,
                        server_version: c.version(),
                    });
                });
            }
        }
    }
}

impl<A: Aper> Drop for ServerHandle<A> {
    fn drop(&mut self) {
        self.callbacks.remove(&self.client_id);
    }
}
