# `aper_derive`: derive macro for [Aper](https://aper.dev)

This crate exists because procedural macros (currently) need their own
crate. The macro itself is re-exposed through the core Aper crate, which
is probably what you want to use instead of depending on this crate
directly.