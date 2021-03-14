use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data_structures::ZenoIndex;
use crate::{StateMachine, Transition};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(bound = "")]
pub enum ListOperation<T: StateMachine + PartialEq> {
    Insert(ZenoIndex, Uuid, T),
    Append(Uuid, T),
    Prepend(Uuid, T),
    Delete(Uuid),
    Move(Uuid, ZenoIndex),
    Apply(Uuid, <T as StateMachine>::Transition),
}

impl<T: StateMachine + PartialEq> Transition for ListOperation<T> {}

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
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(bound = "")]
pub struct List<T: StateMachine + PartialEq> {
    items: BTreeMap<ZenoIndex, Uuid>,
    items_inv: BTreeMap<Uuid, ZenoIndex>,
    pool: HashMap<Uuid, T>,
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

impl<T: StateMachine + PartialEq> StateMachine for List<T> {
    type Transition = ListOperation<T>;

    fn apply(&mut self, transition_event: Self::Transition) {
        match transition_event {
            ListOperation::Append(id, value) => {
                let location = if let Some((last_location, _)) = self.items.iter().next_back() {
                    ZenoIndex::new_after(last_location)
                } else {
                    ZenoIndex::default()
                };
                self.do_insert(location, id, value)
            }
            ListOperation::Prepend(id, value) => {
                let location = if let Some((first_location, _)) = self.items.iter().next() {
                    ZenoIndex::new_before(first_location)
                } else {
                    ZenoIndex::default()
                };
                self.do_insert(location, id, value)
            }
            ListOperation::Insert(location, id, value) => self.do_insert(location, id, value),
            ListOperation::Delete(id) => self.do_delete(id),
            ListOperation::Move(id, location) => self.do_move(id, location),
            ListOperation::Apply(id, transition) => {
                if let Some(v) = self.pool.get_mut(&id) {
                    v.apply(transition)
                } else {
                    // TODO: resolve conflict.
                }
            }
        }
    }
}

pub type OperationWithId<T> = (Uuid, ListOperation<T>);

impl<T: StateMachine + PartialEq> List<T> {
    fn do_insert(&mut self, location: ZenoIndex, id: Uuid, value: T) {
        self.items.insert(location.clone(), id);
        self.items_inv.insert(id, location);
        self.pool.insert(id, value);
    }

    fn do_move(&mut self, id: Uuid, location: ZenoIndex) {
        if let Some(old_location) = self.items_inv.remove(&id) {
            self.items.remove(&old_location);
            self.items.insert(location.clone(), id);
            self.items_inv.insert(id, location);
        } else {
            // TODO: if the item is not in the pool, we have a conflict.
            // For now, we ignore it.
        }
    }

    fn do_delete(&mut self, id: Uuid) {
        if let Some(location) = self.items_inv.remove(&id) {
            self.items.remove(&location);
        }
        self.pool.remove(&id);
    }

    /// Construct an [OperationWithId] representing appending the given object to this
    /// list.
    pub fn append(&self, value: T) -> OperationWithId<T> {
        let id = Uuid::new_v4();
        (id, ListOperation::Append(id, value))
    }

    /// Construct a [OperationWithId] representing prepending the given object to this
    /// list.
    pub fn prepend(&self, value: T) -> OperationWithId<T> {
        let id = Uuid::new_v4();
        (id, ListOperation::Prepend(id, value))
    }

    /// Construct a [OperationWithId] representing inserting the given object at the
    /// given location in this list.
    pub fn insert(&self, location: ZenoIndex, value: T) -> OperationWithId<T> {
        let id = Uuid::new_v4();
        (id, ListOperation::Insert(location, id, value))
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
    fn test_list() {
        let mut list: List<Atom<i64>> = List::default();

        // Test Append.

        list.apply(list.append(Atom::new(5)).1);

        list.apply(list.append(Atom::new(3)).1);

        list.apply(list.append(Atom::new(143)).1);

        // Test Prepend.

        list.apply(list.prepend(Atom::new(99)).1);

        {
            let result: Vec<i64> = list.iter().map(|d| *d.value.value()).collect();
            assert_eq!(vec![99, 5, 3, 143], result);
        }

        // Test Insert.
        {
            let locations: Vec<ZenoIndex> = list.iter().map(|d| d.location).collect();

            list.apply(
                list.insert(
                    ZenoIndex::new_between(&locations[2], &locations[3]),
                    Atom::new(44),
                )
                .1,
            );

            list.apply(
                list.insert(
                    ZenoIndex::new_between(&locations[0], &locations[1]),
                    Atom::new(23),
                )
                .1,
            );

            list.apply(
                list.insert(
                    ZenoIndex::new_between(&locations[1], &locations[2]),
                    Atom::new(84),
                )
                .1,
            );

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value.value()).collect();
                assert_eq!(vec![99, 23, 5, 84, 3, 44, 143], result);
            }
        }

        // Test Delete.
        {
            let uuids: Vec<Uuid> = list.iter().map(|d| d.id).collect();

            list.apply(list.delete(uuids[2]));

            list.apply(list.delete(uuids[3]));

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
                ZenoIndex::new_between(&locations[2], &locations[3]),
            ));

            list.apply(list.move_item(uuids[4], ZenoIndex::new_before(&locations[0])));

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value.value()).collect();
                assert_eq!(vec![143, 23, 3, 99, 44], result);
            }
        }
    }
}
