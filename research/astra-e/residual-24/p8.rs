#[allow(dead_code)]
#[path="constructor.rs"]
mod kernel;
include!("adapter.inc.rs");
pub type Candidate = Packed<8>;
