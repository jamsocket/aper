use aper::{
    data_structures::{FixedArray, Map},
    AperSync,
};

// A Rust macro that takes a list of values and returns a Vec that is constructed from bincode-serializing each value.
macro_rules! prefix {
    ($($x:expr),*) => {
        vec![$(bincode::serialize(&$x).unwrap()),*]
    };
}

#[test]
fn test_store_cleanup() {
    let store = aper::Store::default();
    let mut map = Map::<String, FixedArray<2, u32>>::attach(store.handle());

    map.get_or_create(&"key1".to_string());

    {
        let prefixes = store.prefixes();
        assert_eq!(prefixes, vec![prefix!("key1".to_string()),]);
    }

    map.delete(&"key1".to_string());

    // TODO: Enable this.

    // {
    //     let prefixes = store.prefixes();
    //     assert!(prefixes.is_empty());
    // }
}
