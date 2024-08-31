use aper::{
    data_structures::{atom::Atom, fixed_array::FixedArray},
    Aper, AperClient, AperSync, Mutation,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;

#[derive(AperSync)]
struct SimpleStruct {
    atom_i32: Atom<i32>,
    atom_string: Atom<String>,
    fixed_array: FixedArray<5, u8>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SimpleIntent {
    SetAtomI32(i32),
    SetAtomString(String),
    SetFixedArray(u32, u8),
}

impl Aper for SimpleStruct {
    type Intent = SimpleIntent;
    type Error = ();

    fn apply(&mut self, intent: &Self::Intent) -> Result<(), Self::Error> {
        match intent {
            SimpleIntent::SetAtomI32(value) => self.atom_i32.set(*value),
            SimpleIntent::SetAtomString(value) => self.atom_string.set(value.clone()),
            SimpleIntent::SetFixedArray(index, value) => self.fixed_array.set(*index, *value),
        }

        Ok(())
    }
}

#[test]
fn test_apply_listener() {
    // let map = TreeMapRef::new();
    // let mut st = DummyStruct::attach(map);
    let mut client: AperClient<SimpleStruct> = aper::AperClient::new();

    let (atom_i32_send, atom_i32_recv) = channel();
    let (atom_string_send, atom_string_recv) = channel();
    let (fixed_array_send, fixed_array_recv) = channel();

    let st = client.state();

    st.atom_i32.listen(move || atom_i32_send.send(()).is_ok());
    st.atom_string
        .listen(move || atom_string_send.send(()).is_ok());
    st.fixed_array
        .listen(move || fixed_array_send.send(()).is_ok());

    client.apply(&SimpleIntent::SetAtomI32(42)).unwrap();

    assert!(atom_i32_recv.try_recv().is_ok());
    assert!(atom_string_recv.try_recv().is_err());
    assert!(fixed_array_recv.try_recv().is_err());

    client
        .apply(&SimpleIntent::SetAtomString("hello".to_string()))
        .unwrap();

    assert!(atom_i32_recv.try_recv().is_err());
    assert!(atom_string_recv.try_recv().is_ok());
    assert!(fixed_array_recv.try_recv().is_err());

    client.apply(&SimpleIntent::SetFixedArray(0, 42)).unwrap();

    assert!(atom_i32_recv.try_recv().is_err());
    assert!(atom_string_recv.try_recv().is_err());
    assert!(fixed_array_recv.try_recv().is_ok());
}

#[test]
fn test_mutate_listener_simple() {
    // simple case: server mutates a value directly

    let mut client: AperClient<SimpleStruct> = aper::AperClient::new();

    let (atom_i32_send, atom_i32_recv) = channel();
    let (atom_string_send, atom_string_recv) = channel();
    let (fixed_array_send, fixed_array_recv) = channel();

    let st = client.state();

    st.atom_i32.listen(move || atom_i32_send.send(()).is_ok());
    st.atom_string
        .listen(move || atom_string_send.send(()).is_ok());
    st.fixed_array
        .listen(move || fixed_array_send.send(()).is_ok());

    client.mutate(
        &vec![Mutation {
            prefix: vec![b"atom_i32".to_vec()],
            entries: vec![(b"".to_vec(), Some(42i32.to_le_bytes().to_vec()))],
        }],
        None,
        1,
    );

    assert_eq!(42, st.atom_i32.get());

    assert!(atom_i32_recv.try_recv().is_ok());
    assert!(atom_string_recv.try_recv().is_err());
    assert!(fixed_array_recv.try_recv().is_err());
}

#[derive(AperSync)]
struct LinkedFields {
    lhs: Atom<i32>,
    rhs: Atom<i32>,
    sum: Atom<i32>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum LinkedFieldIntent {
    SetLhs(i32),
    SetRhs(i32),
}

impl Aper for LinkedFields {
    type Intent = LinkedFieldIntent;
    type Error = ();

    fn apply(&mut self, intent: &Self::Intent) -> Result<(), Self::Error> {
        match intent {
            LinkedFieldIntent::SetLhs(value) => self.lhs.set(*value),
            LinkedFieldIntent::SetRhs(value) => self.rhs.set(*value),
        }

        self.sum.set(self.lhs.get() + self.rhs.get());

        Ok(())
    }
}

#[test]
fn test_mutate_listener_incidental() {
    // more complex case: server mutation causes another value to be recomputed

    let mut client: AperClient<LinkedFields> = aper::AperClient::new();

    let (lhs_send, lhs_recv) = channel();
    let (rhs_send, rhs_recv) = channel();
    let (sum_send, sum_recv) = channel();

    let st = client.state();

    st.lhs.listen(move || lhs_send.send(()).is_ok());
    st.rhs.listen(move || rhs_send.send(()).is_ok());
    st.sum.listen(move || sum_send.send(()).is_ok());

    client.apply(&LinkedFieldIntent::SetLhs(1)).unwrap();

    assert_eq!(1, st.lhs.get());
    assert_eq!(1, st.sum.get());

    assert!(lhs_recv.try_recv().is_ok());
    assert!(rhs_recv.try_recv().is_err());
    assert!(sum_recv.try_recv().is_ok());

    // now mutate the rhs, which should cause the sum to be recomputed

    client.mutate(
        &vec![Mutation {
            prefix: vec![b"rhs".to_vec()],
            entries: vec![(b"".to_vec(), Some(26i32.to_le_bytes().to_vec()))],
        }],
        None,
        1,
    );

    // TODO: we shouldn't need to reconstruct the state, but we do because the state refers to a specific layer.
    let st = client.state();

    assert_eq!(26, st.rhs.get());
    assert_eq!(27, st.sum.get());

    // note: the underlying value of lhs did not change,
    // so we could omit it in the future as an optimization.
    assert!(lhs_recv.try_recv().is_ok());
    assert!(rhs_recv.try_recv().is_ok());
    assert!(sum_recv.try_recv().is_ok());
}
