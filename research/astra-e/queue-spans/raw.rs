//! The same compact queue without joining neighboring retained runs.
#[path = "candidate.rs"]
mod variants;
pub type Candidate = variants::Variant<2>;
