//! Frozen scalar-cap worker.
#[allow(dead_code)]
#[path="target/generated/scalar.rs"]
mod kernel;
include!("target/generated/gate.rs");
include!("target/generated/adapter.inc.rs");
