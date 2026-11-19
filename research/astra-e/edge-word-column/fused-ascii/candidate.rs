#[allow(dead_code)]
#[path = "generated/kernel.rs"]
pub mod kernel;
include!("../../edge-budget/gate.rs");
include!("generated/adapter.inc.rs");
