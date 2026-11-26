#[allow(dead_code)]
#[path = "candidate.rs"]
mod tested;
pub type Candidate = tested::AnchorKernel<true>;
