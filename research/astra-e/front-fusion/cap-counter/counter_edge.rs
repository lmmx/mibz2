//! Frozen scalar first-cap and retained-queue counter factorial arm.
const SCALAR_CAP: bool = false;
const RECOVER: bool = true;
#[allow(dead_code)]
#[path = "kernel.rs"]
pub mod kernel;
include!("../../edge-budget/gate.rs");
include!("adapter.inc.rs");
