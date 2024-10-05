use aper::{
    data_structures::atom::Atom, Aper, AperClient, AperServer, AperSync, IntentMetadata,
    StoreHandle,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct Counter(Atom<u64>);

impl AperSync for Counter {
    fn attach(map: StoreHandle) -> Self {
        Self(Atom::attach(map))
    }
}

impl Counter {
    fn get(&self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
enum CounterIntent {
    IncrementBy(u64),
    SetTo(u64),
}

impl Aper for Counter {
    type Intent = CounterIntent;
    type Error = ();

    fn apply(
        &mut self,
        intent: &Self::Intent,
        _metadata: &IntentMetadata,
    ) -> Result<(), Self::Error> {
        match &intent {
            CounterIntent::IncrementBy(amount) => {
                self.0.set(self.0.get() + amount);
            }
            CounterIntent::SetTo(value) => {
                self.0.set(*value);
            }
        }

        Ok(())
    }
}

#[test]
fn test_local_change() {
    let mut client = AperClient::<Counter>::new();
    let mut server = AperServer::<Counter>::new();

    let version = client
        .apply(&CounterIntent::IncrementBy(5), &IntentMetadata::now())
        .unwrap();

    assert_eq!(1, version);
    assert_eq!(0, client.verified_client_version());
    assert_eq!(1, client.speculative_client_version());

    let mutations = server
        .apply(&CounterIntent::IncrementBy(5), &IntentMetadata::now())
        .unwrap();

    client.mutate(&mutations, Some(version), 1);

    assert_eq!(1, client.verified_client_version());
    assert_eq!(1, client.speculative_client_version());

    let state = client.state();
    assert_eq!(5, state.get());
}

#[test]
fn test_remote_change() {
    let mut server = AperServer::<Counter>::new();

    let mutations = server
        .apply(&CounterIntent::IncrementBy(5), &IntentMetadata::now())
        .unwrap();

    let mut client = AperClient::<Counter>::new();
    client.mutate(&mutations, None, 1);

    assert_eq!(0, client.verified_client_version());
    assert_eq!(0, client.speculative_client_version());

    let state = client.state();
    assert_eq!(5, state.get());
}

#[test]
fn test_speculative_change_remains() {
    // client1 makes a speculative change, then receives another change from the server.
    // client1 should apply both the speculative change and the server change.

    let mut server = AperServer::<Counter>::new();
    let mut client = AperClient::<Counter>::new();

    client
        .apply(&CounterIntent::IncrementBy(5), &IntentMetadata::now())
        .unwrap();

    let mutations = server
        .apply(&CounterIntent::IncrementBy(10), &IntentMetadata::now())
        .unwrap();

    client.mutate(&mutations, None, 1);

    assert_eq!(0, client.verified_client_version());
    assert_eq!(1, client.speculative_client_version());

    let state = client.state();
    assert_eq!(15, state.get());
}

#[test]
fn test_remote_changes_persist() {
    let mut server = AperServer::<Counter>::new();
    let mut client = AperClient::<Counter>::new();

    let mutations = server
        .apply(&CounterIntent::IncrementBy(5), &IntentMetadata::now())
        .unwrap();
    client.mutate(&mutations, None, 1);

    let state = client.state();
    assert_eq!(5, state.get());

    let mutations = server
        .apply(&CounterIntent::IncrementBy(5), &IntentMetadata::now())
        .unwrap();
    client.mutate(&mutations, None, 1);

    let state = client.state();
    assert_eq!(10, state.get());
}
