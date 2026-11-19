#[allow(dead_code)]
#[path = "candidate.rs"]
mod slot;
pub type Candidate = slot::FinalSlot<true>;
