use aper::{StateMachine};
use aper::data_structures::Atom;
use serde::{Serialize, Deserialize};

#[derive(StateMachine, Debug, Serialize, Deserialize, Clone)]
struct MyRecordStruct {
    left: Atom<u32>,
    right: Atom<String>,
}

#[test]
fn test_derive() {
    let mut r = MyRecordStruct {
        left: Atom::new(30),
        right: Atom::new("blah".to_string()),
    };

    r.apply(r.map_left(|d| d.replace(4))).unwrap();
    r.apply(r.map_right(|d| d.replace("foo".to_string()))).unwrap();

    assert_eq!(&4, r.left.value());
    assert_eq!("foo", r.right.value());
}