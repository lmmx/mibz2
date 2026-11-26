//! Eight-byte certificate arm.
#[path = "candidate.rs"]
mod shared;
pub type Candidate = shared::Solver<true>;
