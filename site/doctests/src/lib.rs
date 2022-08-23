#[macro_use]
extern crate doc_comment;

extern crate aper;

// Guide book
doctest!("../../book/src/01-introduction.md");
doctest!("../../book/src/02-managing-state.md");
doctest!("../../book/src/02a-building.md");
doctest!("../../book/src/02b-atoms.md");
doctest!("../../book/src/02c-derive.md");
doctest!("../../book/src/02d-lists.md");
doctest!("../../book/src/02e-designing.md");
doctest!("../../book/src/02f-implementing.md");

// Website
doctest!("../../website/index.md");
