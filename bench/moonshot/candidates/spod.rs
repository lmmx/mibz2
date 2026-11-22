//! Force SPOD on every column, including cases outside the streaming admission gate.
use mibz2::bwt::{InverseBwt, OpenCells};

#[derive(Default)]
pub struct Candidate(OpenCells);

impl InverseBwt for Candidate {
    fn invert(&mut self, column: &[u8], _: &[u32; 256], primary: u32, out: &mut Vec<u8>) {
        self.0.check_spod_column(column, primary, out);
    }
}
