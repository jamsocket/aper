use aper::{
    data_structures::{atom::Atom, fixed_array::FixedArray},
    Aper, AperClient, Attach,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;

#[derive(Attach)]
struct DummyStruct {
    atom_i32: Atom<i32>,
    atom_string: Atom<String>,
    fixed_array: FixedArray<5, u8>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Intent {
    SetAtomI32(i32),
    SetAtomString(String),
    SetFixedArray(u32, u8),
}

impl Aper for DummyStruct {
    type Intent = Intent;
    type Error = ();

    fn apply(&mut self, intent: &Self::Intent) -> Result<(), Self::Error> {
        match intent {
            Intent::SetAtomI32(value) => self.atom_i32.set(*value),
            Intent::SetAtomString(value) => self.atom_string.set(value.clone()),
            Intent::SetFixedArray(index, value) => self.fixed_array.set(*index, *value),
        }

        Ok(())
    }
}

#[test]
fn test_listener() {
    // let map = TreeMapRef::new();
    // let mut st = DummyStruct::attach(map);
    let mut client: AperClient<DummyStruct> = aper::AperClient::new();

    let (atom_i32_send, atom_i32_recv) = channel();
    let (atom_string_send, atom_string_recv) = channel();
    let (fixed_array_send, fixed_array_recv) = channel();

    let st = client.state();

    st.atom_i32.listen(move || atom_i32_send.send(()).is_ok());
    st.atom_string
        .listen(move || atom_string_send.send(()).is_ok());
    st.fixed_array
        .listen(move || fixed_array_send.send(()).is_ok());

    client.apply(&Intent::SetAtomI32(42)).unwrap();

    assert!(atom_i32_recv.try_recv().is_ok());
    assert!(atom_string_recv.try_recv().is_err());
    assert!(fixed_array_recv.try_recv().is_err());
}
