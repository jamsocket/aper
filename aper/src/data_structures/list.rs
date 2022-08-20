use crate::StateMachine;
use fractional_index::ZenoIndex;
use serde::de::Visitor;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
//use std::collections::{BTreeMap, HashMap};
use im_rc::{HashMap, OrdMap};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Bound::{Excluded, Unbounded};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ListPosition {
    Beginning,
    End,
    AbsolutePosition(ZenoIndex),
    Before(Uuid, ZenoIndex),
    After(Uuid, ZenoIndex),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(bound = "")]
pub enum ListOperation<T: StateMachine + PartialEq> {
    Insert(ListPosition, Uuid, T),
    Delete(Uuid),
    Move(Uuid, ZenoIndex),
    Apply(Uuid, <T as StateMachine>::Transition),
}

/// Represents a view of an entry in a list during iteration. Each
/// item contains a borrow of its `value`; its `location` as a [ZenoIndex],
/// and a unique identifier which is opaque but must be passed for
/// [List::delete] and [List::move_item] calls.
pub struct ListItem<'a, T: StateMachine + PartialEq> {
    pub value: &'a T,
    pub location: ZenoIndex,
    pub id: Uuid,
}

/// Represents a list of items, similar to a `Vec`, but designed to be robust
/// to concurrent modifications from multiple users.
#[derive(Clone, PartialEq, Debug)]
pub struct List<T: StateMachine + PartialEq> {
    items: OrdMap<ZenoIndex, Uuid>,
    items_inv: OrdMap<Uuid, ZenoIndex>,
    pool: HashMap<Uuid, T>,
}

impl<T: StateMachine + PartialEq> Serialize for List<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.items.len()))?;
        for i in self.items.iter().map(|(zi, id)| (zi, id, &self.pool[id])) {
            seq.serialize_element(&i)?;
        }
        seq.end()
    }
}

impl<'de, T: StateMachine + PartialEq> Deserialize<'de> for List<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ListVisitor<T>(PhantomData<T>);
        impl<'de, T: StateMachine + PartialEq> Visitor<'de> for ListVisitor<T> {
            type Value = List<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of tuples")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut list = List::new();

                while let Some((zi, id, v)) = seq.next_element::<(ZenoIndex, Uuid, T)>()? {
                    list.items.insert(zi.clone(), id);
                    list.items_inv.insert(id, zi);
                    list.pool.insert(id, v);
                }
                Ok(list)
            }
        }

        deserializer.deserialize_seq(ListVisitor(PhantomData))
    }
}

impl<T: StateMachine + PartialEq> Default for List<T> {
    fn default() -> Self {
        List {
            items: Default::default(),
            items_inv: Default::default(),
            pool: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ListConflict<T: StateMachine> {
    /// No item exists with the given UUID. It may have been deleted
    /// after the transition was created.
    ItemDoesNotExist(Uuid),
    ChildConflict(T::Conflict),
}

impl<T: StateMachine + PartialEq> StateMachine for List<T> {
    type Transition = ListOperation<T>;
    type Conflict = ListConflict<T>;

    fn apply(&self, transition_event: Self::Transition) -> Result<Self, ListConflict<T>> {
        match transition_event {
            ListOperation::Insert(location, id, value) => self.do_insert(location, id, value),
            ListOperation::Delete(id) => self.do_delete(id),
            ListOperation::Move(id, location) => self.do_move(id, location),
            ListOperation::Apply(id, transition) => {
                if let Some(v) = self.pool.get(&id) {
                    match v.apply(transition) {
                        Ok(v) => {
                            let mut new_self = self.clone();
                            new_self.pool = new_self.pool.update(id, v);
                            Ok(new_self)
                        }
                        Err(e) => Err(ListConflict::ChildConflict(e)),
                    }
                } else {
                    Err(ListConflict::ItemDoesNotExist(id))
                }
            }
        }
    }
}

pub type OperationWithId<T> = (Uuid, ListOperation<T>);

impl<T: StateMachine + PartialEq> List<T> {
    pub fn new() -> List<T> {
        Self::default()
    }

    pub fn get_location(&self, position: ListPosition) -> ZenoIndex {
        let location = match position {
            ListPosition::Beginning => {
                // return a zenoindex < the index of the first list element
                return if let Some((i, _)) = self.items.iter().next() {
                    ZenoIndex::new_before(i)
                } else {
                    ZenoIndex::default()
                };
            }
            ListPosition::End => {
                return if let Some((i, _)) = self.items.iter().next_back() {
                    ZenoIndex::new_after(i)
                } else {
                    ZenoIndex::default()
                }
            }
            ListPosition::AbsolutePosition(p) => p,
            ListPosition::Before(uuid, fallback_location) => {
                if let Some(location) = self.items_inv.get(&uuid) {
                    ZenoIndex::new_before(location)
                } else {
                    ZenoIndex::new_before(&fallback_location)
                }
            }
            ListPosition::After(uuid, fallback_location) => {
                if let Some(location) = self.items_inv.get(&uuid) {
                    ZenoIndex::new_after(location)
                } else {
                    ZenoIndex::new_after(&fallback_location)
                }
            }
        };

        if self.items.contains_key(&location) {
            if let Some((next_location, _)) =
                self.items.range((Excluded(&location), Unbounded)).next()
            {
                ZenoIndex::new_between(&location, next_location)
                    .expect("Should always be able to find a zeno index between adjacent keys.")
            } else {
                ZenoIndex::new_after(&location)
            }
        } else {
            location
        }
    }

    fn do_insert(
        &self,
        position: ListPosition,
        id: Uuid,
        value: T,
    ) -> Result<Self, ListConflict<T>> {
        let location = self.get_location(position);

        let mut new_self = self.clone();
        new_self.items.insert(location.clone(), id);
        new_self.items_inv.insert(id, location);
        new_self.pool.insert(id, value);
        Ok(new_self)
    }

    fn do_move(&self, id: Uuid, location: ZenoIndex) -> Result<Self, ListConflict<T>> {
        let mut new_self = self.clone();
        if let Some(old_location) = new_self.items_inv.remove(&id) {
            new_self.items.remove(&old_location);
            new_self.items.insert(location.clone(), id);
            new_self.items_inv.insert(id, location);
            Ok(new_self)
        } else {
            Err(ListConflict::ItemDoesNotExist(id))
        }
    }

    fn do_delete(&self, id: Uuid) -> Result<Self, ListConflict<T>> {
        let mut new_self = self.clone();
        if let Some(location) = new_self.items_inv.remove(&id) {
            new_self.items.remove(&location);
        }
        new_self.pool.remove(&id);

        Ok(new_self)
    }

    pub fn insert_between(&self, id1: &Uuid, id2: &Uuid, value: T) -> OperationWithId<T> {
        let id = Uuid::new_v4();
        let loc1 = self.items_inv.get(id1).unwrap();
        let loc2 = self.items_inv.get(id2).unwrap();
        let new_loc = ZenoIndex::new_between(loc1, loc2)
            .expect("Should be able to insert between two items in list.");
        (
            id,
            ListOperation::Insert(ListPosition::AbsolutePosition(new_loc), id, value),
        )
    }

    /// Construct an [OperationWithId] representing appending the given object to this
    /// list.
    pub fn append(&self, value: T) -> OperationWithId<T> {
        let id = Uuid::new_v4();
        (id, ListOperation::Insert(ListPosition::End, id, value))
    }

    /// Construct a [OperationWithId] representing prepending the given object to this
    /// list.
    pub fn prepend(&self, value: T) -> OperationWithId<T> {
        let id = Uuid::new_v4();
        (
            id,
            ListOperation::Insert(ListPosition::Beginning, id, value),
        )
    }

    /// Construct a [OperationWithId] representing inserting the given object at the
    /// given location in this list.
    pub fn insert(&self, location: ZenoIndex, value: T) -> OperationWithId<T> {
        let id = Uuid::new_v4();
        (
            id,
            ListOperation::Insert(ListPosition::AbsolutePosition(location), id, value),
        )
    }

    /// Construct a [ListOperation] representing deleting the object given (by id)
    /// in this list.
    pub fn delete(&self, id: Uuid) -> ListOperation<T> {
        ListOperation::Delete(id)
    }

    /// Construct a [ListOperation] representing moving an object already in this
    /// list to the given location in the list.
    pub fn move_item(&self, id: Uuid, new_location: ZenoIndex) -> ListOperation<T> {
        ListOperation::Move(id, new_location)
    }

    pub fn map_item(
        &self,
        id: Uuid,
        fun: impl FnOnce(&T) -> <T as StateMachine>::Transition,
    ) -> <Self as StateMachine>::Transition {
        if let Some(it) = self.pool.get(&id) {
            ListOperation::Apply(id, fun(it))
        } else {
            // Handle conflict.
            panic!("Conflict should be better handled.")
        }
    }

    /// Returns an iterator over [ListItem] views into this list.
    pub fn iter(&self) -> impl Iterator<Item = ListItem<T>> {
        self.items.iter().map(move |(location, id)| ListItem {
            location: location.clone(),
            id: *id,
            value: &self.pool[id],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::Atom;

    #[test]
    fn test_conflict() {
        let my_list: List<Atom<u32>> = List::new();

        let id = Uuid::new_v4();
        let transition = my_list.move_item(id, ZenoIndex::default());

        assert_eq!(
            Err(ListConflict::ItemDoesNotExist(id)),
            my_list.apply(transition)
        );
    }

    #[test]
    fn test_get_location() {
        let my_list: List<Atom<u32>> = List::new();
        let mut ids: Vec<Uuid> = vec![];

        for i in 0..10 {
            let (id, transition) = my_list.append(Atom::new(i));
            ids.push(id);

            my_list.apply(transition).unwrap();
        }

        // Beginning

        assert!(
            my_list.get_location(ListPosition::Beginning) < *my_list.items.keys().next().unwrap()
        );

        // Ending

        assert!(
            my_list.get_location(ListPosition::End) > *my_list.items.keys().next_back().unwrap()
        );

        // AbsolutePosition

        assert!(
            my_list.get_location(ListPosition::AbsolutePosition(
                my_list.items_inv[&ids[4]].clone()
            )) > my_list.items_inv[&ids[4]]
        );

        // Before

        assert!(
            my_list.get_location(ListPosition::Before(
                ids[7],
                my_list.items_inv[&ids[7]].clone()
            )) < my_list.items_inv[&ids[7]]
        );

        // After

        assert!(
            my_list.get_location(ListPosition::After(
                ids[7],
                my_list.items_inv[&ids[7]].clone()
            )) > my_list.items_inv[&ids[7]]
        );
    }

    #[test]
    fn test_insert_between_merge() {
        let my_list: List<Atom<u32>> = List::new();

        let (id1, transition1) = my_list.append(Atom::new(1));
        let (id2, transition2) = my_list.append(Atom::new(2));

        my_list.apply(transition2).unwrap(); // my_list = [2]
        my_list.apply(transition1).unwrap(); // my_list = [2, 1]

        let (_id3, transition3) = my_list.insert_between(&id2, &id1, Atom::new(3));

        let (_id4, transition4) = my_list.insert_between(&id2, &id1, Atom::new(4));

        my_list.apply(transition4).unwrap();
        assert_eq!(
            vec![2, 4, 1],
            my_list
                .iter()
                .map(|d| *d.value.value())
                .collect::<Vec<u32>>()
        );
        my_list.apply(transition3).unwrap();
        assert_eq!(
            vec![2, 4, 3, 1],
            my_list
                .iter()
                .map(|d| *d.value.value())
                .collect::<Vec<u32>>()
        );
    }

    #[test]
    fn test_list() {
        let list: List<Atom<i64>> = List::default();

        // Test Append.

        list.apply(list.append(Atom::new(5)).1).unwrap();

        list.apply(list.append(Atom::new(3)).1).unwrap();

        list.apply(list.append(Atom::new(143)).1).unwrap();

        // Test Prepend.

        list.apply(list.prepend(Atom::new(99)).1).unwrap();

        {
            let result: Vec<i64> = list.iter().map(|d| *d.value.value()).collect();
            assert_eq!(vec![99, 5, 3, 143], result);
        }

        // Test Insert.
        {
            let locations: Vec<ZenoIndex> = list.iter().map(|d| d.location).collect();

            list.apply(
                list.insert(
                    ZenoIndex::new_between(&locations[2], &locations[3]).unwrap(),
                    Atom::new(44),
                )
                .1,
            )
            .unwrap();

            list.apply(
                list.insert(
                    ZenoIndex::new_between(&locations[0], &locations[1]).unwrap(),
                    Atom::new(23),
                )
                .1,
            )
            .unwrap();

            list.apply(
                list.insert(
                    ZenoIndex::new_between(&locations[1], &locations[2]).unwrap(),
                    Atom::new(84),
                )
                .1,
            )
            .unwrap();

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value.value()).collect();
                assert_eq!(vec![99, 23, 5, 84, 3, 44, 143], result);
            }
        }

        // Test Delete.
        {
            let uuids: Vec<Uuid> = list.iter().map(|d| d.id).collect();

            list.apply(list.delete(uuids[2])).unwrap();

            list.apply(list.delete(uuids[3])).unwrap();

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value.value()).collect();
                assert_eq!(vec![99, 23, 3, 44, 143], result);
            }
        }

        // Test Move.
        {
            let uuids: Vec<Uuid> = list.iter().map(|d| d.id).collect();
            let locations: Vec<ZenoIndex> = list.iter().map(|d| d.location).collect();

            list.apply(list.move_item(
                uuids[0],
                ZenoIndex::new_between(&locations[2], &locations[3]).unwrap(),
            ))
            .unwrap();

            list.apply(list.move_item(uuids[4], ZenoIndex::new_before(&locations[0])))
                .unwrap();

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value.value()).collect();
                assert_eq!(vec![143, 23, 3, 99, 44], result);
            }
        }
    }

    #[test]
    fn test_list_serialization() {
        // Serialization of nonempty List to JSON used to fail
        // because serde-json requires map keys to be strings.

        let list: List<Atom<i64>> = List::default();
        list.apply(list.append(Atom::new(5)).1).unwrap();

        let result = serde_json::to_string(&list).unwrap();

        let parsed_list: List<Atom<i64>> = serde_json::from_str(&result).unwrap();

        assert_eq!(list.items, parsed_list.items);
        assert_eq!(list.items_inv, parsed_list.items_inv);
        assert_eq!(list.pool, parsed_list.pool);
    }
}
