use aper::{data_structures::atom_map::AtomMap, AperSync, Store, StoreHandle};

#[test]
fn test_backtrack() {
    let treemap = Store::default();
    let mut map = AtomMap::<u8, u8>::attach(StoreHandle::new_root(&treemap));

    {
        map.set(&1, &2);
        map.set(&3, &4);

        assert_eq!(map.get(&1), Some(2));
        assert_eq!(map.get(&3), Some(4));
    }

    // add an overlay to the map

    {
        treemap.push_overlay();

        // existing values are still there

        assert_eq!(map.get(&1), Some(2));
        assert_eq!(map.get(&3), Some(4));

        // new values can be added

        map.set(&5, &6);
        map.set(&7, &8);

        assert_eq!(map.get(&5), Some(6));
        assert_eq!(map.get(&7), Some(8));

        // existing values can be updated

        map.set(&1, &10);
        map.set(&3, &12);

        assert_eq!(map.get(&1), Some(10));
        assert_eq!(map.get(&3), Some(12));

        // existing values can be removed

        map.delete(&1);
        map.delete(&3);

        assert_eq!(map.get(&1), None);
        assert_eq!(map.get(&3), None);
    }

    // when we pop the overlay, the original values are restored

    {
        treemap.pop_overlay();

        assert_eq!(map.get(&1), Some(2));
        assert_eq!(map.get(&3), Some(4));
    }
}
