use aper::{data_structures::map::Map, Attach, TreeMapRef};

#[test]
fn test_backtrack() {
    let treemap = TreeMapRef::new();

    {
        let mut map = Map::<u8, u8>::attach(treemap.clone());

        map.set(&1, &2);
        map.set(&3, &4);

        assert_eq!(map.get(&1), Some(2));
        assert_eq!(map.get(&3), Some(4));
    }

    // add an overlay to the map

    {
        let treemap = treemap.push_overlay();
        let mut map = Map::<u8, u8>::attach(treemap);

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

    // we can still access the original map

    {
        let map = Map::<u8, u8>::attach(treemap.clone());

        assert_eq!(map.get(&1), Some(2));
        assert_eq!(map.get(&3), Some(4));
    }
}
