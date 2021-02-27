# Designing with state machines

When working with state machines, situations often come up where there 
are multiple ways to represent the same state. We saw this already 
with the `ToDoListItem` [example](derive.md), where the state could be 
either:

1. A single `Atom` of a `ToDoListItem` struct with primitive types for 
   fields, or
2. A struct of two `Atom` fields, with each atom containing a 
   primitive type, implementing `StateMachine` through the `derive` 
   macro.

Both approaches are functionally equivalent when it comes to 
representing data *at rest*. They differ only in how merges are 
handled when (in this case) multiple fields are modified at the same 
time. For the to-do list example, we decided that #2 was the right 
approach, but there may be other data types for which #1 is the best 
approach.

This points to an important aspect of Aper that is central to its 
design philosophy. Aper's focus is on helping you to express the 
**semantics of merging concurrent modifications** to the same data 
structure. The underlying data structures Aper provides are just a 
means to that end.