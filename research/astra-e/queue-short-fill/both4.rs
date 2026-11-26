//! both predecessor, short-width specialization through 4.
const SCALAR_CAP:bool=true;
const RECOVER:bool=true;
#[allow(dead_code)]
#[path="target/generated/both4.rs"]
pub mod kernel;
include!("target/generated/gate.rs");
include!("target/generated/adapter.inc.rs");
