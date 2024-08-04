use crate::{
    connection::{ClientConnection, MessageToServer},
    treemap::TreeMapRef,
    Mutation,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt::Debug};

pub trait Attach {
    fn attach(map: TreeMapRef) -> Self;
}

pub trait Aper: Attach {
    type Intent: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq;
    type Error: Debug;

    fn apply(&mut self, intent: &Self::Intent) -> Result<(), Self::Error>;
}

struct SpeculativeIntent<I> {
    intent: I,
    version: u64,
}

pub struct AperClient<A: Aper> {
    verified: TreeMapRef,
    speculative: TreeMapRef,
    intent_stack: VecDeque<SpeculativeIntent<A::Intent>>,

    /// The next unused client version number for this client.
    next_client_version: u64,

    /// The highest *local* client version that has been confirmed by the server.
    verified_client_version: u64,

    /// The highest *server* version that has been confirmed by the server.
    /// Note that server and client versions are not related.
    verified_server_version: u64,
}

impl<A: Aper> AperClient<A> {
    pub fn new() -> Self {
        let verified = TreeMapRef::new();
        let speculative = verified.push_overlay();

        Self {
            verified,
            speculative,
            intent_stack: VecDeque::new(),
            next_client_version: 1,
            verified_client_version: 0,
            verified_server_version: 0,
        }
    }

    pub fn connect<F: Fn(MessageToServer) + 'static, FS: Fn(A) + 'static>(
        self,
        message_callback: F,
        state_callback: FS,
    ) -> ClientConnection<A> {
        ClientConnection::new(self, message_callback, state_callback)
    }

    pub fn state(&self) -> A {
        A::attach(self.speculative.clone())
    }

    pub fn verified_client_version(&self) -> u64 {
        self.verified_client_version
    }

    pub fn speculative_client_version(&self) -> u64 {
        self.intent_stack
            .back()
            .map_or(self.verified_client_version, |index| index.version)
    }

    pub fn verified_server_version(&self) -> u64 {
        self.verified_server_version
    }

    pub fn apply(&mut self, intent: &A::Intent) -> Result<u64, A::Error> {
        let overlay = self.speculative.push_overlay();

        {
            let mut sm = A::attach(overlay.clone());

            if let Err(err) = sm.apply(&intent) {
                return Err(err);
            }
        }

        let version = self.next_client_version;
        self.intent_stack.push_back(SpeculativeIntent {
            intent: intent.clone(),
            version,
        });
        self.next_client_version += 1;

        self.speculative.combine(&overlay);

        Ok(version)
    }

    pub fn mutate(
        &mut self,
        mutations: &Vec<Mutation>,
        client_version: Option<u64>,
        server_version: u64,
    ) {
        self.verified.mutate(mutations);
        self.verified_server_version = server_version;

        if let Some(version) = client_version {
            self.verified_client_version = version;

            if let Some(index) = self.intent_stack.front() {
                if index.version == version {
                    self.intent_stack.pop_front();
                    // happy case; no need to recompute other speculative intents
                    return;
                }
            }

            while let Some(index) = self.intent_stack.front() {
                if index.version > version {
                    break;
                }

                self.intent_stack.pop_front();
            }
        }

        self.speculative = self.verified.push_overlay();

        for speculative_intent in self.intent_stack.iter() {
            let overlay = self.speculative.push_overlay();
            let mut sm = A::attach(overlay.clone());

            if let Err(_) = sm.apply(&speculative_intent.intent) {
                continue;
            }

            self.speculative.combine(&overlay);
        }
    }
}

pub struct AperServer<A: Aper> {
    state: TreeMapRef,
    version: u64,
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Aper> AperServer<A> {
    pub fn new() -> Self {
        let state = TreeMapRef::new();

        Self {
            state,
            version: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn state_snapshot(&self) -> Vec<Mutation> {
        self.state.clone().into_mutations()
    }

    pub fn apply(&mut self, intent: &A::Intent) -> Result<Vec<Mutation>, A::Error> {
        let overlay = self.state.push_overlay();
        println!("00 {:?}", overlay.collect());

        let mut sm = A::attach(overlay.clone());

        sm.apply(&intent)?;

        println!("aa {:?}", overlay.collect());

        self.version += 1;

        let mutations = overlay.into_mutations();
        println!("bb {:?}", mutations);
        self.state.mutate(&mutations);

        Ok(mutations)
    }
}
