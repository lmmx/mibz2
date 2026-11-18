//! Research-only sibling module of the pinned crate, compiled in scratch.
//! Emit the actual run frontend's records. The dense MTF column and Plain output are separate oracles.
use crate::bwt::{InverseBwt, Plain};
use crate::runs::{front, takes_runs, Runs};
use crate::stream::{Event, Scanner, SliceSource};
use crate::{mtf, Kernels};
use std::{env, fs, path::Path};

pub(super) fn run() {
    let args: Vec<_> = env::args().collect();
    let data = fs::read(&args[1]).unwrap();
    let directory = Path::new(&args[2]);
    let stride: usize = args[3].parse().unwrap();
    fs::create_dir_all(directory).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut storage = Vec::new();
    let mut column = Vec::new();
    let kernels = Kernels::detect();
    let mut packed = Runs::default();
    let plain_dir = args.get(4).map(Path::new);
    if let Some(dir) = plain_dir { fs::create_dir_all(dir).unwrap(); }
    let mut plain = Plain::default();
    let mut expected = Vec::new();
    let mut counts = [0; 256];
    let mut block = 0;
    println!("block,index,primary,symbols,run_front,n,capped");
    while let Event::Block { index, symbols, .. } = scanner.next(storage).unwrap() {
        let n = mtf::mtf_scalar(&symbols, &mut column, &mut counts).unwrap();
        if n >= 65536 {
            if block % stride == 0 {
                assert!(takes_runs(&symbols));
                let run_rows = front(&symbols, &mut packed, kernels).unwrap();
                assert_eq!(run_rows, n);
                let records = packed.records();
                let bytes: Vec<u8> = records.iter().flat_map(|r| r.to_le_bytes()).collect();
                println!("{block},{index},{},{},1,{run_rows},{}", symbols.orig_ptr,
                         symbols.syms.len(), records.len());
                fs::write(directory.join(format!("{block:04}.runs")), bytes).unwrap();
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
