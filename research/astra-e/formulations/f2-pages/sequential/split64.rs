//! Page-major construction with the original two-array, 64-row lookup.
#[allow(dead_code)]
#[path="candidate.rs"]
mod implementation;
pub type Candidate = implementation::PageCandidate<64,true>;
