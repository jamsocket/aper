use crate::StateMachine;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A [StateMachine] representing a value which is "atomic" from
/// the perspective of managing state: it is only ever changed by
/// completely replacing it.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Atom<T: Clone + PartialEq + Debug + Unpin> {
    value: T,
}

impl<T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug> Atom<T> {
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

impl<T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug>
    StateMachine for Atom<T>
{
    type Transition = ReplaceAtom<T>;

    fn apply(&mut self, transition_event: Self::Transition) {
        let ReplaceAtom(v) = transition_event;
        self.value = v;
    }
}

impl<
        T: Default + 'static + Clone + PartialEq + Debug + Unpin + Send + Serialize + DeserializeOwned,
    > Default for Atom<T>
{
    fn default() -> Self {
        Atom::new(Default::default())
    }
}

/// Represents a transition used to change the value of an [Atom].
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ReplaceAtom<T: Clone + PartialEq + Debug + Unpin>(T);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace() {
        let mut atom = Atom::new(5);
        assert_eq!(5, *atom.value());

        atom.apply(atom.replace(8));

        assert_eq!(8, *atom.value());
    }
}
