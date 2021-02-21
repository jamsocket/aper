use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data_structures::ZenoIndex;
use crate::{StateMachine, TransitionEvent};
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ListOperation<T> {
    Insert(ZenoIndex, Uuid, T),
    Append(Uuid, T),
    Prepend(Uuid, T),
    Delete(Uuid),
    Move(Uuid, ZenoIndex),
}

/// Represents a view of an entry in a list during iteration. Each
/// item contains a borrow of its `value`; its `location` as a [ZenoIndex],
/// and a unique identifier which is opaque but must be passed for
/// [List::delete] and [List::move_item] calls.
pub struct ListItem<'a, T> {
    pub value: &'a T,
    pub location: ZenoIndex,
    pub id: Uuid,
}

/// Represents a list of items, similar to a `Vec`, but designed to be robust
/// to concurrent modifications from multiple users.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct List<T: 'static + Unpin + Send + Clone + PartialEq + Debug> {
    items: BTreeMap<ZenoIndex, Uuid>,
    items_inv: BTreeMap<Uuid, ZenoIndex>,
    pool: HashMap<Uuid, T>,
}

impl<T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug>
    StateMachine for List<T>
{
    type Transition = ListOperation<T>;

    fn process_event(&mut self, transition_event: TransitionEvent<Self::Transition>) {
        match transition_event.transition {
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
        }
    }
}

impl<T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug> List<T> {
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

    /// Construct a [ListOperation] representing appending the given object to this
    /// list.
    pub fn append(&self, value: T) -> ListOperation<T> {
        let id = Uuid::new_v4();
        ListOperation::Append(id, value)
    }

    /// Construct a [ListOperation] representing prepending the given object to this
    /// list.
    pub fn prepend(&self, value: T) -> ListOperation<T> {
        let id = Uuid::new_v4();
        ListOperation::Prepend(id, value)
    }

    /// Construct a [ListOperation] representing inserting the given object at the
    /// given location in this list.
    pub fn insert(&self, location: ZenoIndex, value: T) -> ListOperation<T> {
        let id = Uuid::new_v4();
        ListOperation::Insert(location, id, value)
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

    #[test]
    fn test_list() {
        let mut list: List<i64> = List::default();

        // Test Append.

        list.process_event(TransitionEvent::new_tick_event(list.append(5)));

        list.process_event(TransitionEvent::new_tick_event(list.append(3)));

        list.process_event(TransitionEvent::new_tick_event(list.append(143)));

        // Test Prepend.

        list.process_event(TransitionEvent::new_tick_event(list.prepend(99)));

        {
            let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
            assert_eq!(vec![99, 5, 3, 143], result);
        }

        // Test Insert.
        {
            let locations: Vec<ZenoIndex> = list.iter().map(|d| d.location).collect();

            list.process_event(TransitionEvent::new_tick_event(
                list.insert(ZenoIndex::new_between(&locations[2], &locations[3]), 44),
            ));

            list.process_event(TransitionEvent::new_tick_event(
                list.insert(ZenoIndex::new_between(&locations[0], &locations[1]), 23),
            ));

            list.process_event(TransitionEvent::new_tick_event(
                list.insert(ZenoIndex::new_between(&locations[1], &locations[2]), 84),
            ));

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
                assert_eq!(vec![99, 23, 5, 84, 3, 44, 143], result);
            }
        }

        // Test Delete.
        {
            let uuids: Vec<Uuid> = list.iter().map(|d| d.id).collect();

            list.process_event(TransitionEvent::new_tick_event(list.delete(uuids[2])));

            list.process_event(TransitionEvent::new_tick_event(list.delete(uuids[3])));

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
                assert_eq!(vec![99, 23, 3, 44, 143], result);
            }
        }

        // Test Move.
        {
            let uuids: Vec<Uuid> = list.iter().map(|d| d.id).collect();
            let locations: Vec<ZenoIndex> = list.iter().map(|d| d.location).collect();

            list.process_event(TransitionEvent::new_tick_event(list.move_item(
                uuids[0],
                ZenoIndex::new_between(&locations[2], &locations[3]),
            )));

            list.process_event(TransitionEvent::new_tick_event(
                list.move_item(uuids[4], ZenoIndex::new_before(&locations[0])),
            ));

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
                assert_eq!(vec![143, 23, 3, 99, 44], result);
            }
        }
    }
}
