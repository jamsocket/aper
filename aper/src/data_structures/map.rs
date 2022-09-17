use crate::{NeverConflict, StateMachine};
use im_rc::OrdMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Map<
    T: Serialize + DeserializeOwned + Ord + PartialEq + Clone + Debug + 'static,
    V: StateMachine,
> {
    #[serde(bound = "")]
    inner: OrdMap<T, V>,
}

impl<
        T: Serialize + DeserializeOwned + Ord + PartialEq + Clone + Debug + 'static,
        V: StateMachine + PartialEq,
    > Map<T, V>
{
    pub fn new() -> Self {
        Map {
            inner: OrdMap::default(),
        }
    }

    pub fn insert(&self, key: T, value: V) -> MapTransition<T, V> {
        MapTransition {
            key,
            value: Some(value),
        }
    }

    pub fn delete(&self, key: T) -> MapTransition<T, V> {
        MapTransition { key, value: None }
    }

    pub fn get(&self, key: &T) -> Option<&V> {
        self.inner.get(key)
    }

    /// Returns an iterator over [ListItem] views into this list.
    pub fn iter(&self) -> impl Iterator<Item = (&T, &V)> {
        self.inner.iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MapTransition<T: PartialEq, V: PartialEq> {
    key: T,
    value: Option<V>,
}

impl<
        T: Serialize + DeserializeOwned + Ord + PartialEq + Clone + Debug + 'static,
        V: StateMachine + PartialEq,
    > StateMachine for Map<T, V>
{
    type Transition = MapTransition<T, V>;
    type Conflict = NeverConflict;

    fn apply(&self, transition: &Self::Transition) -> Result<Self, Self::Conflict> {
        match &transition.value {
            Some(v) => {
                let mut c = self.inner.clone();
                c.insert(transition.key.clone(), v.clone());
                Ok(Map { inner: c })
            }
            None => {
                let mut c = self.inner.clone();
                c.remove(&transition.key);
                Ok(Map { inner: c })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::data_structures::Atom;

    use super::*;

    #[test]
    fn map_is_empty() {
        let map: Map<String, Atom<u32>> = Map::new();

        let values: Vec<(&String, &Atom<u32>)> = map.iter().collect();

        assert!(values.is_empty());
    }

    #[test]
    fn map_inserts() {
        let mut map: Map<String, Atom<u32>> = Map::new();

        let transition = map.insert("foo".into(), Atom::new(44));
        map = map.apply(&transition).unwrap();

        let values: Vec<(&String, &Atom<u32>)> = map.iter().collect();
        assert_eq!(vec![(&"foo".to_string(), &Atom::new(44))], values);
    }
}
