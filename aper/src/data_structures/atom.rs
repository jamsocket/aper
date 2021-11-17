use crate::{StateMachine, Transition};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fmt::Debug;

/// A [StateMachine] representing a value which is "atomic" from
/// the perspective of managing state: it is only ever changed by
/// completely replacing it.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Atom<T: Clone + PartialEq + Debug + Unpin + Send + Sync> {
    value: T,
}

impl<T: Send + Sync> Atom<T>
where
    T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug,
{
    /// Create a new [Atom] with a given initial value.
    pub fn new(initial: T) -> Self {
        Atom { value: initial }
    }

    /// Retrieve the current value of the atom.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Return a transition which, when processed, will replace the value of the atom
    /// with the value provided.
    pub fn replace(&self, replacement: T) -> ReplaceAtom<T> {
        ReplaceAtom(replacement)
    }
}

impl<T: Send + Sync> StateMachine for Atom<T>
where
    T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug,
{
    type Transition = ReplaceAtom<T>;
    type Conflict = Infallible;

    fn apply(&mut self, transition_event: Self::Transition) -> Result<(), Infallible> {
        let ReplaceAtom(v) = transition_event;
        self.value = v;
        Ok(())
    }
}

impl<T: Send + Sync> Default for Atom<T>
where
    T: Default + 'static + Clone + PartialEq + Debug + Unpin + Send + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Atom::new(Default::default())
    }
}

/// Represents a transition used to change the value of an [Atom].
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ReplaceAtom<T: Clone + PartialEq + Debug + Unpin>(T);

impl<T: Send + Sync> Transition for ReplaceAtom<T> where
    T: 'static + Clone + PartialEq + Debug + Unpin + Serialize + DeserializeOwned + Send
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace() {
        let mut atom = Atom::new(5);
        assert_eq!(5, *atom.value());

        atom.apply(atom.replace(8)).unwrap();

        assert_eq!(8, *atom.value());
    }
}
