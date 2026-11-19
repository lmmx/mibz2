//! Publish cached terminal suffixes during private phrase dictionary construction.
#[allow(dead_code,unexpected_cfgs)]
#[path="family.rs"]
mod family;
pub type Candidate = family::Dispatch<2,2,3>;
