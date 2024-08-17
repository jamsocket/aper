use crate::{
    connection::{ClientConnection, MessageToServer},
    treemap::TreeMap,
    Mutation, TreeMapRef,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    fmt::Debug,
};

pub trait Attach {
    fn attach(map: TreeMapRef) -> Self;

    fn listen<F: Fn() -> bool + 'static + Send + Sync>(&self, listener: F) {
        // Default implementation does nothing.
    }
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
    map: TreeMap,
    intent_stack: VecDeque<SpeculativeIntent<A::Intent>>,

    /// The next unused client version number for this client.
    next_client_version: u64,

    /// The highest *local* client version that has been confirmed by the server.
    verified_client_version: u64,

    /// The highest *server* version that has been confirmed by the server.
    /// Note that server and client versions are not related.
    verified_server_version: u64,
}

impl<A: Aper> Default for AperClient<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Aper> AperClient<A> {
    pub fn new() -> Self {
        let map = TreeMap::default();
        // add an overlay for speculative (local) changes
        map.push_overlay();

        Self {
            map,
            intent_stack: VecDeque::new(),
            next_client_version: 1,
            verified_client_version: 0,
            verified_server_version: 0,
        }
    }

    pub fn connect<F: Fn(MessageToServer) + 'static, FS: Fn(A, u32) + 'static>(
        self,
        message_callback: F,
        state_callback: FS,
    ) -> ClientConnection<A> {
        ClientConnection::new(self, message_callback, state_callback)
    }

    pub fn state(&self) -> A {
        A::attach(TreeMapRef::new_root(&self.map))
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

    /// Apply a mutation to the local client state.
    pub fn apply(&mut self, intent: &A::Intent) -> Result<u64, A::Error> {
        self.map.push_overlay();

        {
            let mut sm = A::attach(TreeMapRef::new_root(&self.map));

            if let Err(e) = sm.apply(intent) {
                // reverse changes.
                self.map.pop_overlay();
                return Err(e);
            }
        }

        let version = self.next_client_version;
        self.intent_stack.push_back(SpeculativeIntent {
            intent: intent.clone(),
            version,
        });
        self.next_client_version += 1;

        self.map.combine_down();

        Ok(version)
    }

    /// Mutate the local client state according to server-verified mutations.
    pub fn mutate(
        &mut self,
        mutations: &Vec<Mutation>,
        client_version: Option<u64>,
        server_version: u64,
    ) {
        // pop speculative overlay
        self.map.pop_overlay();
        self.verified_server_version = server_version;

        self.map.mutate(mutations);

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

        // push new speculative overlay
        self.map.push_overlay();

        for speculative_intent in self.intent_stack.iter() {
            // push a working overlay
            self.map.push_overlay();
            let mut sm = A::attach(TreeMapRef::new_root(&self.map));

            if sm.apply(&speculative_intent.intent).is_err() {
                // reverse changes.
                self.map.pop_overlay();
                continue;
            }

            self.map.combine_down();
        }
    }
}

pub struct AperServer<A: Aper> {
    map: TreeMap,
    version: u64,
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Aper> Default for AperServer<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Aper> AperServer<A> {
    pub fn new() -> Self {
        let map = TreeMap::default();

        Self {
            map,
            version: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn state_snapshot(&self) -> Vec<Mutation> {
        // self.map.clone().into_mutations()
        todo!()
    }

    pub fn apply(&mut self, intent: &A::Intent) -> Result<Vec<Mutation>, A::Error> {
        self.map.push_overlay();

        let mut sm = A::attach(TreeMapRef::new_root(&self.map));

        if let Err(e) = sm.apply(intent) {
            // reverse changes.
            self.map.pop_overlay();
            return Err(e);
        }

        self.version += 1;

        let mutations = self.map.top_layer_mutations();
        self.map.combine_down();

        Ok(mutations)
    }

    pub fn state(&self) -> A {
        A::attach(TreeMapRef::new_root(&self.map))
    }
}
