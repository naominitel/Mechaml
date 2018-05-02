//! Bindings over the OCaml standard library types and functions.
//!
//! Those bindings are hardcoded for the moment but will likely be derived
//! automatially from type definitions in the future.
//!
//! Writing `use mechaml::stdlib::pervasives::*` at the beginning of any Rust
//! module manipulating OCaml data is generally a good idea.

// Public modules, more or less map to the OCaml modules.
#[macro_use] pub mod pervasives;
#[macro_use] pub mod list;

// Private modules, just to structure the Rust code, do not match anything in
// the OCaml module hierarhy.
mod option;
