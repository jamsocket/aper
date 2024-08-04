use crate::{Aper, AperClient, AperServer};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    sync::{atomic::AtomicU64, Arc},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageToServer {
    Intent {
        intent: Vec<u8>,
        client_version: u64,
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
    callback: Box<dyn Fn(MessageToServer)>,
}

impl<A: Aper> ClientConnection<A> {
    pub fn new<F: Fn(MessageToServer) + 'static>(client: AperClient<A>, callback: F) -> Self {
        Self {
            client,
            callback: Box::new(callback),
        }
    }

    pub fn state(&self) -> A {
        self.client.state()
    }

    pub fn apply(&mut self, intent: &A::Intent) -> Result<(), A::Error> {
        let version = self.client.apply(&intent)?;
        let intent = bincode::serialize(intent).unwrap();
        (self.callback)(MessageToServer::Intent {
            intent,
            client_version: version,
        });
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
            }
        }
    }
}

pub struct ServerConnection<A: Aper> {
    callbacks: Arc<DashMap<u64, Box<dyn Fn(&MessageToClient)>>>,
    server: Arc<RefCell<AperServer<A>>>,
    next_client_id: AtomicU64,
}

impl<A: Aper> ServerConnection<A> {
    pub fn connect<F: Fn(&MessageToClient) + 'static>(&mut self, callback: F) -> ServerHandle<A> {
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
    server: Arc<RefCell<AperServer<A>>>,
    callbacks: Arc<DashMap<u64, Box<dyn Fn(&MessageToClient)>>>,
}

impl<A: Aper> ServerHandle<A> {
    pub fn receive(&mut self, message: &MessageToServer, client_id: u64) {
        match message {
            MessageToServer::Intent {
                intent,
                client_version,
            } => {
                let intent = bincode::deserialize(intent).unwrap();
                let mut server_borrow = self.server.borrow_mut();
                let Ok(mutations) = server_borrow.apply(&intent) else {
                    // still need to ack the client.

                    self.callbacks.get(&client_id).map(|callback| {
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
                    if *other_client_id == client_id {
                        callback(&message_to_sender);
                    } else {
                        callback(&message_to_others);
                    }
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
