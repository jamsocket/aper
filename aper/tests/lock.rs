//! Listener functions should be able to access document data if they hold a handle to an `AperSync` object.
//! A naive implementation where we lock the entire store doesn't work, because locking to iterate over
//! listeners breaks the ability to obtain a lock to access the store.

use aper::{data_structures::Atom, AperSync, Store};

#[test]
fn listener_can_access_data() {
    let (tx, rx) = std::sync::mpsc::channel::<u8>();
    let store = Store::default();

    let mut atom1: Atom<u8> = Atom::attach(store.handle());
    let atom2: Atom<u8> = Atom::attach(store.handle());

    atom1.listen(move || {
        tx.send(atom2.get()).unwrap();
        true
    });

    atom1.set(42);
    store.alert(&vec![]);

    assert_eq!(rx.try_recv().unwrap(), 42);
}
