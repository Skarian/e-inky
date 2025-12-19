//! CREngine-NG build glue.
//!
//! This crate currently focuses on compiling and packaging the upstream C/C++ engine. Safe Rust
//! bindings will be layered on top in follow-up work.

/// Anchor symbol exported by the Rust side of the crate to ensure the cdylib/staticlib has at least
/// one well-known symbol.
#[no_mangle]
pub extern "C" fn crengine_link_anchor() {}
