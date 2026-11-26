//! Exact marked reconstruction through bounded select on the stable byte queues.
#[allow(dead_code)]
#[path = "target/kernel.rs"]
mod kernel;
include!("target/gate.rs");
include!("target/adapter.inc.rs");
