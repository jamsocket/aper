#[macro_use]
extern crate doc_comment;

extern crate aper;

// Guide book
doctest!("../../book/src/atoms.md");
doctest!("../../book/src/building.md");
doctest!("../../book/src/derive.md");
doctest!("../../book/src/implementing.md");
doctest!("../../book/src/lists.md");

// Website
doctest!("../../website/index.md");
