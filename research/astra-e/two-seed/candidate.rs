//! Frozen i9 edge worker with an in-place two-forward-stream emitter.
//! two-forward-stream boundary-directed emitter. Prepare with build_worker.py.
#[allow(dead_code)]
#[path="target/generated/two-seed.rs"]
pub mod kernel;
include!("target/generated/gate.rs");
include!("target/generated/adapter.inc.rs");
