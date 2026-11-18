//! Research input adapter for the pinned public scanner and scalar MTF implementation.
//! Emits a dense column for the counting oracles. This allocation is outside the proposed algorithm.
use mibz2::bwt::{InverseBwt, Plain};
use mibz2::stream::{Event, Scanner, SliceSource};
use std::{env, fs, path::Path};

fn main() {
    let args: Vec<_> = env::args().collect();
    let data = fs::read(&args[1]).unwrap();
    let directory = Path::new(&args[2]);
    let stride: usize = args[3].parse().unwrap();
    fs::create_dir_all(directory).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut storage = Vec::new();
    let mut column = Vec::new();
    let plain_dir = args.get(4).map(Path::new);
    if let Some(dir) = plain_dir { fs::create_dir_all(dir).unwrap(); }
    let mut plain = Plain::default();
    let mut expected = Vec::new();
    let mut counts = [0; 256];
    let mut block = 0;
    println!("block,index,primary,symbols,run_front");
    while let Event::Block { index, symbols, .. } = scanner.next(storage).unwrap() {
        let n = mibz2::mtf::mtf_scalar(&symbols, &mut column, &mut counts).unwrap();
        if n >= 65536 {
            if block % stride == 0 {
                println!("{block},{index},{},{},{}", symbols.orig_ptr, symbols.syms.len(),
                         u8::from(mibz2::runs::takes_runs(&symbols)));
                fs::write(directory.join(format!("{block:04}.L")), &column).unwrap();
                if let Some(dir) = plain_dir {
                    plain.invert(&column, &counts, symbols.orig_ptr, &mut expected);
                    fs::write(dir.join(format!("{block:04}.T")), &expected).unwrap();
                }
            }
            block += 1;
        }
        storage = symbols.syms;
    }
    assert_eq!(block, 233);
}
