//! Preserve dynamic per-lane path growth during complete phrase discovery.
#[allow(dead_code,unexpected_cfgs)]
#[path="family.rs"]
mod family;
pub type Candidate = family::Dispatch<2,2,3>;
