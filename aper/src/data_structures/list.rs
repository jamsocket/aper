use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{StateMachine, TransitionEvent};
use crate::data_structures::ZenoIndex;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
enum ListOperation<T> {
    Insert(ZenoIndex, Uuid, T),
    Append(Uuid, T),
    Prepend(Uuid, T),
    Delete(Uuid),
    Move(Uuid, ZenoIndex),
}

struct ListItem<'a, T> {
    pub value: &'a T,
    pub location: ZenoIndex,
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
struct List<T: 'static + Unpin + Send + Clone + PartialEq + Debug> {
    items: BTreeMap<ZenoIndex, Uuid>,
    items_inv: BTreeMap<Uuid, ZenoIndex>,
    pool: HashMap<Uuid, T>,
}

impl<T: 'static + Serialize + for<'de> Deserialize<'de> + Unpin + Send + Clone + PartialEq + Debug> StateMachine for List<T> {
    type Transition = ListOperation<T>;

    fn process_event(&mut self, transition_event: TransitionEvent<Self::Transition>) {
        match transition_event.transition {
            ListOperation::Append(id, value) => {
                let location = if let Some((last_location, _)) = self.items.iter().next_back() {
                    ZenoIndex::new_after(last_location)
                } else {
                    ZenoIndex::default()
                };
                self.insert_at_location(location, id, value)
            }
            ListOperation::Prepend(id, value) => {
                let location = if let Some((first_location, _)) = self.items.iter().next() {
                    ZenoIndex::new_before(first_location)
                } else {
                    ZenoIndex::default()
                };
                self.insert_at_location(location, id, value)
            }
            ListOperation::Insert(location, id, value) => {
                self.insert_at_location(location, id, value)
            }
            ListOperation::Delete(id) => {
                self.delete_by_uuid(id)
            }
            _ => unimplemented!()
        }
    }
}

impl<T: 'static + Serialize + for<'de> Deserialize<'de> + Unpin + Send + Clone + PartialEq + Debug> List<T> {
    fn insert_at_location(&mut self, location: ZenoIndex, id: Uuid, value: T) {
        self.items.insert(location.clone(), id);
        self.items_inv.insert(id, location);
        self.pool.insert(id, value);
    }

    fn delete_by_uuid(&mut self, id: Uuid) {
        if let Some(location) = self.items_inv.remove(&id) {
            self.items.remove(&location);
        }
        self.pool.remove(&id);
    }

    pub fn append(&self, value: T) -> ListOperation<T> {
        let id = Uuid::new_v4();
        ListOperation::Append(id, value)
    }

    pub fn prepend(&self, value: T) -> ListOperation<T> {
        let id = Uuid::new_v4();
        ListOperation::Prepend(id, value)
    }

    fn insert(&self, location: ZenoIndex, value: T) -> ListOperation<T> {
        let id = Uuid::new_v4();
        ListOperation::Insert(location, id, value)
    }

    pub fn delete(&self, id: Uuid) -> ListOperation<T> {
        ListOperation::Delete(id)
    }

    pub fn move_item(&self, id: Uuid, new_location: ZenoIndex) -> ListOperation<T> {
        ListOperation::Move(id, new_location)
    }

    pub fn iter(&self) -> impl Iterator<Item=ListItem<T>> {
        self.items.iter().map(
            move |(location, id)| ListItem {
                location: location.clone(),
                id: id.clone(),
                value: &self.pool[id]
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        let mut list: List<i64> = List::default();

        // Test Append.

        list.process_event(TransitionEvent::new_tick_event(
            list.append(5)));

        list.process_event(TransitionEvent::new_tick_event(
            list.append(3)));

        list.process_event(TransitionEvent::new_tick_event(
            list.append(143)));

        // Test Prepend.

        list.process_event(TransitionEvent::new_tick_event(
            list.prepend(99)));

        {
            let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
            assert_eq!(vec![99, 5, 3, 143], result);
        }

        // Test Insert.
        {
        let locations: Vec<ZenoIndex> = list.iter().map(|d| d.location).collect();

        list.process_event(TransitionEvent::new_tick_event(
            list.insert(
                ZenoIndex::new_between(&locations[2], &locations[3]),
                44
            )
        ));

        list.process_event(TransitionEvent::new_tick_event(
            list.insert(
                ZenoIndex::new_between(&locations[0], &locations[1]),
                23
            )
        ));

        list.process_event(TransitionEvent::new_tick_event(
            list.insert(
                ZenoIndex::new_between(&locations[1], &locations[2]),
                84
            )
        ));

        {
            let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
            assert_eq!(vec![99, 23, 5, 84, 3, 44, 143], result);
        }
    }

        {
            let uuids: Vec<Uuid> = list.iter().map(|d| d.id).collect();

            list.process_event(TransitionEvent::new_tick_event(
                list.delete(uuids[2])
            ));

            list.process_event(TransitionEvent::new_tick_event(
                list.delete(uuids[3])
            ));

            {
                let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
                assert_eq!(vec![99, 23, 3, 44, 143], result);
            }
        }

    }
}