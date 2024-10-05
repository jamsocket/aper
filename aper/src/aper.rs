use crate::{
    connection::{ClientConnection, MessageToServer},
    store::{Store, StoreHandle},
    IntentMetadata, Mutation,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt::Debug};

pub trait AperSync: Clone {
    fn attach(map: StoreHandle) -> Self;

    fn listen<F: Fn() -> bool + 'static + Send + Sync>(&self, _listener: F) {
        // Default implementation does nothing.
    }
}

pub trait Aper: AperSync + 'static {
    type Intent: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq;
    type Error: Debug;

    fn apply(
        &mut self,
        intent: &Self::Intent,
        metadata: &IntentMetadata,
    ) -> Result<(), Self::Error>;

    fn suspended_event(&self) -> Option<(DateTime<Utc>, Self::Intent)> {
        None
    }
}

struct SpeculativeIntent<I> {
    metadata: IntentMetadata,
    intent: I,
    version: u64,
}

pub struct AperClient<A: Aper> {
    store: Store,
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
        let map = Store::default();
        // add an overlay for speculative (local) changes
        map.push_overlay();

        Self {
            store: map,
            intent_stack: VecDeque::new(),
            next_client_version: 1,
            verified_client_version: 0,
            verified_server_version: 0,
        }
    }

    pub fn store(&self) -> Store {
        self.store.clone()
    }

    pub fn connect<F: Fn(MessageToServer) + 'static>(
        self,
        message_callback: F,
    ) -> ClientConnection<A> {
        ClientConnection::new(self, message_callback)
    }

    pub fn state(&self) -> A {
        A::attach(self.store.handle())
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
    pub fn apply(
        &mut self,
        intent: &A::Intent,
        metadata: &IntentMetadata,
    ) -> Result<u64, A::Error> {
        self.store.push_overlay();

        {
            let mut sm = A::attach(self.store.handle());

            if let Err(e) = sm.apply(intent, metadata) {
                // reverse changes.
                self.store.pop_overlay();
                return Err(e);
            }
        }

        let version = self.next_client_version;
        self.intent_stack.push_back(SpeculativeIntent {
            intent: intent.clone(),
            metadata: metadata.clone(),
            version,
        });
        self.next_client_version += 1;

        self.store.combine_down();
        self.store.notify_dirty();

        Ok(version)
    }

    /// Mutate the local client state according to server-verified mutations.
    pub fn mutate(
        &mut self,
        mutations: &[Mutation],
        client_version: Option<u64>,
        server_version: u64,
    ) {
        // pop speculative overlay
        // TODO: we need to capture notifications from the speculative overlay being popped, since it could
        // undo changes that are not re-done.
        self.store.pop_overlay();
        self.verified_server_version = server_version;

        self.store.mutate(mutations);

        // push new speculative overlay
        self.store.push_overlay();

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

        for speculative_intent in self.intent_stack.iter() {
            // push a working overlay
            self.store.push_overlay();

            let mut sm = A::attach(self.store.handle());

            if sm
                .apply(&speculative_intent.intent, &speculative_intent.metadata)
                .is_err()
            {
                // reverse changes.
                self.store.pop_overlay();
                continue;
            }

            self.store.combine_down();
        }

        self.store.notify_dirty();
    }
}

pub struct AperServer<A: Aper> {
    map: Store,
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
        let map = Store::default();

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
        // this works because the server only has one layer
        self.map.top_layer_mutations()
    }

    pub fn apply(
        &mut self,
        intent: &A::Intent,
        metadata: &IntentMetadata,
    ) -> Result<Vec<Mutation>, A::Error> {
        self.map.push_overlay();

        let mut sm = A::attach(self.map.handle());

        if let Err(e) = sm.apply(intent, metadata) {
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
        A::attach(self.map.handle())
    }
}
