use aper::{data_structures::atom::Atom, Aper, AperClient, AperServer, Attach, TreeMapRef};
use serde::{Deserialize, Serialize};

struct Counter(Atom<u64>);

impl Attach for Counter {
    fn attach(map: TreeMapRef) -> Self {
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

    fn apply(&mut self, intent: &Self::Intent) -> Result<(), Self::Error> {
        match intent {
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

    let intent = CounterIntent::IncrementBy(5);
    let version = client.apply(&intent).unwrap();

    assert_eq!(1, version);
    assert_eq!(0, client.verified_client_version());
    assert_eq!(1, client.speculative_client_version());

    let mutations = server.apply(&intent).unwrap();

    client.mutate(&mutations, Some(version), 1);

    assert_eq!(1, client.verified_client_version());
    assert_eq!(1, client.speculative_client_version());

    let state = client.state();
    assert_eq!(5, state.get());
}

#[test]
fn test_remote_change() {
    let mut server = AperServer::<Counter>::new();

    let intent = CounterIntent::IncrementBy(5);
    let mutations = server.apply(&intent).unwrap();

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

    client.apply(&CounterIntent::IncrementBy(5)).unwrap();

    let mutations = server.apply(&CounterIntent::SetTo(10)).unwrap();

    client.mutate(&mutations, None, 1);

    assert_eq!(0, client.verified_client_version());
    assert_eq!(1, client.speculative_client_version());

    let state = client.state();
    assert_eq!(15, state.get());
}
