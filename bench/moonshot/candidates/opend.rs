//! Astra's open-return decoder behind LazyCells' density line (`opend`, #565, #590): the arm the
//! i9 measured at 0.940 (7/28 at 10 threads) and 0.919 (8/32) against 0.5.6. A worked example
//! of a candidate whose code lives elsewhere (the journal) and reads the run records.
pub type Candidate = crate::arms::OpenGated;
