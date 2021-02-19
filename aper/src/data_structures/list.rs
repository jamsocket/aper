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

                self.items.insert(location, id);
                self.pool.insert(id, value);
            }
            _ => unimplemented!()
        }
    }
}

impl<T: 'static + Serialize + for<'de> Deserialize<'de> + Unpin + Send + Clone + PartialEq + Debug> List<T> {
    pub fn append(&self, value: T) -> ListOperation<T> {
        let id = Uuid::new_v4();
        ListOperation::Append(id, value)
    }

    pub fn insert(&self, location: ZenoIndex, value: T) -> ListOperation<T> {
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

        list.process_event(TransitionEvent::new_tick_event(
            list.append(5)));

        list.process_event(TransitionEvent::new_tick_event(
            list.append(3)));

        list.process_event(TransitionEvent::new_tick_event(
            list.append(143)));

        {
            let result: Vec<i64> = list.iter().map(|d| *d.value).collect();
            assert_eq!(vec![5, 3, 143], result);
        }


    }
}