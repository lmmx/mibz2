//! Represent and validate last-observed target run ordinals.
#[allow(dead_code,unexpected_cfgs)]
#[path="family.rs"]
mod family;
pub type Candidate = family::Dispatch<2,2,2>;
