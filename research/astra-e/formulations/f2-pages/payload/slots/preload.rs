//! Return eight loaded compact run responses to their fixed discovery slots.
#[allow(dead_code,unexpected_cfgs)]
#[path="candidate.rs"]
mod family;
pub type Candidate = family::Dispatch<2,2>;
