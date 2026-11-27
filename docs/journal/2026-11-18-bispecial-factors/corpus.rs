//! Copy into an example subdirectory together with contexts.rs, as documented in the README.
#[allow(dead_code)]
mod contexts;

use mibz2::{
    bwt::{InverseBwt, Plain},
    crc::crc32,
    rle::unrle,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
use std::{env, fs, io::Write};

fn main() {
    let args: Vec<_> = env::args().collect();
    let compressed = fs::read(&args[1]).unwrap();
    let mut decoded = std::io::BufWriter::new(fs::File::create(&args[2]).unwrap());
    let k = Kernels::detect();
    let mut scanner = Scanner::new(SliceSource(&compressed));
    let mut plain = Plain::default();
    let (mut symbols_buf, mut last, mut expected, mut finished) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut counts = [0; 256];
    println!("block,n,sigma,depths,right_special,bispecial,forests,components_minus_one,chords,weight_fields,positive_edges,support_bits,weight_bytes,support_tape_bytes,terminal_classes,marked_period,output_bytes");
    while let Event::Block { symbols, index, .. } = scanner.next(symbols_buf).unwrap() {
        mibz2::mtf::mtf(&symbols, &mut last, &mut counts, k).unwrap();
        plain.invert(&last, &counts, symbols.orig_ptr, &mut expected);
        let (bytes, totals) = contexts::check_block(&last, symbols.orig_ptr as usize);
        assert_eq!(bytes, expected, "pre-RLE bytes for block {index}");
        finished.clear();
        unrle(&bytes, &mut finished, k).unwrap();
        assert_eq!(crc32(&finished, k), symbols.crc, "CRC for block {index}");
        decoded.write_all(&finished).unwrap();
        let sigma = counts.iter().filter(|&&x| x != 0).count();
        print!("{index},{},{sigma}", last.len());
        for t in totals {
            print!(",{t}");
        }
        println!(",{}", finished.len());
        symbols_buf = symbols.syms;
    }
}
