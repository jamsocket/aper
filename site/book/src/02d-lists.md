# Lists

Another built-in data structure is the `List`. Lists are ordered sequence that you can iterate over, and arbitrarily insert/remove from. The values in a list are themselves state machines (if you never want to modify the inner state of values on a list, you can use `Constant` as a wrapper around the values.)

Let's use the `ToDoListItem` from the last section to create a `ToDoList`

```rust,noplaypen
use aper::data_structures::List;
# use aper::StateMachine;
# use aper::data_structures::Atom;
# use serde::{Serialize, Deserialize};
# use std::default::Default;
# 
# #[derive(StateMachine, Serialize, Deserialize, Debug, Clone, PartialEq)]
# struct ToDoListItem {
#     done: Atom<bool>,
#     label: Atom<String>,
# }
# 
# impl ToDoListItem {
#     pub fn new(label: String) -> Self {
#         ToDoListItem {
#             done: Atom::new(false),
#             label: Atom::new(label),
#         }
#     }
# }

fn main() {
	let mut to_do_list: List<ToDoListItem> = List::default();

	// Initially, the list is empty. We need to add things to it.

	// Append generates and returns an identifier which we can later
	// use to identify the record.
	// The methods `append`, `prepend`, and `insert` of `List`
	// return a `(id, transition)` pair, where the `id` can be used
	// to refer to the element after it has been inserted.
	let (dog_food_id, dog_food_transition) = to_do_list.append(
			ToDoListItem::new("Get dog food".to_string())
	);

	to_do_list = to_do_list.apply(&dog_food_transition).unwrap();

	let (lunch_id, lunch_transition) = to_do_list.append(
			ToDoListItem::new("Make lunch".to_string())
	);

	to_do_list = to_do_list.apply(&lunch_transition).unwrap();

	let emphasize_dog_food = to_do_list.map_item(dog_food_id,
		|it| it.map_label(|lbl| lbl.replace("Get DOG FOOD!".to_string()
		)));

	to_do_list = to_do_list.apply(&emphasize_dog_food).unwrap();

	let mark_lunch_done = to_do_list.map_item(lunch_id,
		|it| it.map_done(|done| done.replace(true)));
}
```