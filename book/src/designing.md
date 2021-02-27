# Designing with state machines

When working with state machines, situations often come up where there are multiple ways to represent the same state. We saw this already with the `ToDoListItem` example, where the state could be either:

1. A single `Atom` of a `ToDoListItem` struct with primitive types for fields, or
2. A struct with two `Atom` fields, with each atom containing a primitive type.

Both approaches are exactly equivalent when it comes to representing data *at rest*. They differ only in how merges are handled when (in this case) multiple fields are modified at the same time. For the to-do list example, we decided that #2 was the right approach, but there may be other data types for which #1 is the best approach.

This points out an important aspect of Aper that is central to its philosophy. Aper's focus is on providing **semantics for describing what happens when concurrent modifications happen to the same data structure**. The underlying storage Aper uses to do this is incidental.