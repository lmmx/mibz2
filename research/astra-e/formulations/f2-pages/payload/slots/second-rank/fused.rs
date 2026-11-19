//! Build second-token rank masks while publishing runs, then scan the masks.
#[allow(dead_code,unexpected_cfgs)]
#[path="family.rs"]
mod family;
pub type Candidate = family::Dispatch<2,2,2>;
