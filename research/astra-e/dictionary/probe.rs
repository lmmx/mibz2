#[path = "candidate.rs"]
mod dictionary;
pub type Candidate = dictionary::DictionaryKernel<true>;
