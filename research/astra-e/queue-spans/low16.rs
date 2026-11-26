//! Full low-u16 row queues with page boundaries shared across byte buckets.
#[path = "candidate.rs"]
mod variants;
pub type Candidate = variants::Variant<4>;
