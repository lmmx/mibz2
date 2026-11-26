//! Frozen edge worker with the in-place four-forward emitter.
#[allow(dead_code)]
#[path="target/generated/edge-four.rs"]
mod kernel;
include!("target/generated/gate.rs");
include!("target/generated/adapter.inc.rs");
