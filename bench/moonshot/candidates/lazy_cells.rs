//! The null candidate: 0.5.6's own kernel, LazyCells as shipped. Scored against itself it must
//! pass the correctness gate and time at 1.0 within its interval: the scorer's own check.
pub type Candidate = mibz2::bwt::LazyCells;
