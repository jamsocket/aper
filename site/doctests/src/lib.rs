#[macro_use]
extern crate doc_comment;

extern crate aper;

// Guide book
doctest!("../../book/src/01-introduction.md");

// Website
doctest!("../../website/index.md");
