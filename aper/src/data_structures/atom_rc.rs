use crate::{NeverConflict, StateMachine};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::rc::Rc;

/// A [StateMachine] representing a value which is "atomic" from
/// the perspective of managing state: it is only ever changed by
/// completely replacing it.
#[derive(Clone, PartialEq, Debug)]
pub struct AtomRc<T> {
    value: Rc<T>,
}

impl<T: Serialize> Serialize for AtomRc<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for AtomRc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(AtomRc {
            value: Rc::new(T::deserialize(deserializer)?),
        })
    }
}

impl<T: PartialEq + Debug> AtomRc<T> {
    /// Create a new [Atom] with a given initial value.
    pub fn new(initial: T) -> Self {
        AtomRc {
            value: Rc::new(initial),
        }
    }

    /// Retrieve the current value of the atom.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Return a transition which, when processed, will replace the value of the atom
    /// with the value provided.
    pub fn replace(&self, replacement: T) -> ReplaceAtomRc<T> {
        ReplaceAtomRc(Rc::new(replacement))
    }
}

/// Represents a transition used to change the value of an [Atom].
#[derive(Clone, PartialEq, Debug)]
pub struct ReplaceAtomRc<T: PartialEq + Debug>(Rc<T>);

impl<T: Serialize + Debug + PartialEq> Serialize for ReplaceAtomRc<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + Debug + PartialEq> Deserialize<'de> for ReplaceAtomRc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ReplaceAtomRc(Rc::new(T::deserialize(deserializer)?)))
    }
}

impl<T: Debug + Clone + Serialize + DeserializeOwned + PartialEq + 'static> StateMachine for AtomRc<T> {
    type Transition = ReplaceAtomRc<T>;
    type Conflict = NeverConflict;

    fn apply(&self, transition_event: &Self::Transition) -> Result<Self, NeverConflict> {
        let ReplaceAtomRc(v) = transition_event;
        Ok(AtomRc { value: v.clone() })
    }
}

impl<T: Default + PartialEq + Debug> Default for AtomRc<T> {
    fn default() -> Self {
        AtomRc::new(Default::default())
    }
}
