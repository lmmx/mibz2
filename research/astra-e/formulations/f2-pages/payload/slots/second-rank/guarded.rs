//! Enable POPCNT once before discovering the second phrase trie.
#[allow(dead_code,unexpected_cfgs)]
#[path="family.rs"]
mod family;
pub type Candidate = family::Dispatch<2,2,1>;
