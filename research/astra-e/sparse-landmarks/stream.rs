pub use mibz2::{bwt, runs, Kernels};
use mibz2::{
    decode_block,
    stream::{Event, Scanner, SliceSource},
    Scratch,
};
mod sparse;
pub use sparse::Table;
use sparse::{paid, Plan};
#[allow(dead_code)]
mod original {
    include!(env!("EDGE_TABLE_SOURCE"));
}
include!("../edge-budget/gate.rs");
use std::io::{BufReader, Read};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let mut reference = BufReader::new(std::fs::File::open(&args[2]).unwrap());
    let mut scan = Scanner::new(SliceSource(&data));
    let mut kernel = original::Kernel::default();
    let mut fallback = Scratch::<bwt::LazyCells>::default();
    let k = Kernels::detect();
    let mut r = runs::Runs::default();
    let (mut plan, mut inv, mut out, mut want) =
        (Plan::default(), Vec::new(), Vec::new(), Vec::new());
    let mut largest_capacity = 0;
    let (mut blocks, mut admitted, mut bytes) = (0, 0, 0);
    while let Event::Block { symbols, .. } = scan.next(Vec::new()).unwrap() {
        blocks += 1;
        out.clear();
        let mut use_edge = false;
        if runs::takes_runs(&symbols) {
            let n = runs::front(&symbols, &mut r, k).unwrap();
            if r.records().len() * 20 < 3 * n || (n >= 65536 && fits(r.records(), symbols.orig_ptr))
            {
                let t = kernel.table(r.records(), n, symbols.orig_ptr);
                paid::<8, 7>(&mut plan, &t, &mut inv);
                mibz2::rle::unrle(&inv, &mut out, k).unwrap();
                largest_capacity = largest_capacity.max(plan.capacity_bytes());
                admitted += 1;
                use_edge = true;
            }
        }
        if !use_edge {
            decode_block(&symbols, k, &mut fallback, &mut out).unwrap();
        }
        assert_eq!(mibz2::crc::crc32(&out, k), symbols.crc);
        want.resize(out.len(), 0);
        reference.read_exact(&mut want).unwrap();
        assert!(out == want);
        bytes += out.len();
    }
    let mut extra = [0];
    assert_eq!(reference.read(&mut extra).unwrap(), 0);
    println!("blocks={blocks} paid_landmark_blocks={admitted} exact_output_bytes={bytes} block_crcs={blocks} scanner_stream_crcs=PASS max_reused_descriptor_capacity_bytes={largest_capacity}");
}
