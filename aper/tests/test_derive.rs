use aper::data_structures::{Atom, AtomRc};
use aper::StateMachine;
use serde::{Deserialize, Serialize};

#[derive(StateMachine, Debug, Serialize, Deserialize, Clone)]
struct MyRecordStruct {
    left: Atom<u32>,
    right: AtomRc<String>,
}

#[test]
fn test_derive() {
    let mut r = MyRecordStruct {
        left: Atom::new(30),
        right: AtomRc::new("blah".to_string()),
    };

    r = r.apply(&r.map_left(|d| d.replace(4))).unwrap();
    r = r
        .apply(&r.map_right(|d| d.replace("foo".to_string())))
        .unwrap();

    assert_eq!(&4, r.left.value());
    assert_eq!("foo", r.right.value());
}
