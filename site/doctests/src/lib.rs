#[macro_use]
extern crate doc_comment;

extern crate aper;

// Guide book
doctest!("../../book/src/01-introduction.md");
doctest!("../../book/src/02-one-way-sync.md");
doctest!("../../book/src/03-bidirectional-sync.md");

// Website
doctest!("../../website/index.md");
