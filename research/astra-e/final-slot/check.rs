//! Independent monolithic-RLE and complete marked-block tests for the final-slot repair.
#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
#[path = "../../../docs/journal/2026-11-18-open-return/open_return.rs"] mod open_return;
#[path = "../../../bench/moonshot/src/arms.rs"] mod arms;
#[path = "../../../tests/common/mod.rs"] mod common;
mod candidate;
mod repair;

fn raw(encoded: &[u8], extra_capacity: usize, k: Kernels, desired: Option<repair::Strategy>) {
    let mut data = Vec::with_capacity(encoded.len()+extra_capacity);
    data.extend_from_slice(encoded);
    let mut expected = Vec::new();
    let reference = mibz2::rle::unrle(encoded,&mut expected,k);
    let mut edits = repair::Directory::default();
    let strategy = edits.apply::<true>(&mut data,k,&mut repair::Costs::default());
    match strategy {
        Err(kind) => {
            assert_eq!(reference,Err(kind));
            assert_eq!(data,encoded);
        }
        Ok(which) => {
            if let Some(want) = desired { assert_eq!(which,want); }
            if which != repair::Strategy::Applied {
                // Reproduce the no-reinvert fallback, preserving any late syntax error.
                assert_eq!(data,encoded);
                let restored = std::mem::take(&mut data);
                assert_eq!(mibz2::rle::unrle(&restored,&mut data,k),reference);
            } else {
                assert_eq!(reference,Ok(()));
            }
            if reference.is_ok() { assert_eq!(data,expected); }
        }
    }
}

fn marked(input: &[u8], want: Option<mibz2::ErrorKind>) {
    use mibz2::{stream::{Event,Scanner,SliceSource},bwt::InverseBwt};
    let mut reader = Scanner::new(SliceSource(input));
    let Event::Block {mut symbols,..} = reader.next(Vec::new()).unwrap() else { panic!() };
    let mut runs = runs::Runs::default();
    let n = runs::front(&symbols,&mut runs,Kernels::scalar()).unwrap();
    assert!(arms::under_density(runs.records().len(),n));
    let mut kernel = candidate::Candidate::default();
    for k in [Kernels::scalar(),Kernels::detect()] {
        let mut nonempty = b"prior output".to_vec();
        assert!(kernel.expand_runs(&runs,n,symbols.orig_ptr,&mut nonempty,k).is_none());
        assert_eq!(&nonempty,b"prior output");
        if !k.avx2() {
            assert!(kernel.expand_runs(&runs,n,symbols.orig_ptr,&mut Vec::new(),k).is_none());
        }
        let mut scratch = mibz2::Scratch::<candidate::Candidate>::default();
        let mut slot = Vec::new();
        for prefix in [b"".as_slice(),b"retained caller prefix".as_slice()] {
            slot.clear();
            slot.extend_from_slice(prefix);
            let result = mibz2::decode_block(&symbols,k,&mut scratch,&mut slot);
            if let Some(kind) = want.as_ref() {
                assert_eq!(result,Err(kind.clone()));
                assert_eq!(&slot,prefix);
            } else {
                assert_eq!(result,Ok(symbols.crc));
                assert_eq!(&slot[..prefix.len()],prefix);
                assert_eq!(&slot[prefix.len()..],&common::oracle(input).unwrap());
                slot.truncate(prefix.len());
                symbols.crc ^= 1;
                assert_eq!(mibz2::decode_block(&symbols,k,&mut scratch,&mut slot),
                           Err(mibz2::ErrorKind::BlockCrc {stored:symbols.crc,computed:symbols.crc^1}));
                assert_eq!(&slot,prefix);
                symbols.crc ^= 1;
                assert_eq!(mibz2::decode_block(&symbols,k,&mut scratch,&mut slot), Ok(symbols.crc));
                assert_eq!(&slot[prefix.len()..],&common::oracle(input).unwrap());
            }
        }
    }
}

fn main() {
    let mut tests = 0;
    for k in [Kernels::scalar(),Kernels::detect()] {
        for mut i in 0..6usize.pow(5) {
            let mut data = vec![1,0];
            for j in 0..5 {
                let c = j+16;
                data.extend_from_slice(&[c,c,c,c,[0,1,2,3,4,255][i%6]]);
                if i&1 == 1 { data.extend_from_slice(b"literal span") }
                i /= 6;
            }
            raw(&data,1300,k,Some(repair::Strategy::Applied));
            raw(&data,0,k,None);
            tests += 2;
        }
        for lead in 0..256 {
            let mut data = (0..lead).map(|i| (i%13) as u8).collect::<Vec<_>>();
            data.extend_from_slice(b"zzzz");
            raw(&data,0,k,None);
            tests += 1;
        }
        raw(&[],0,k,Some(repair::Strategy::Applied));
        raw(&[b'x';5],0,k,Some(repair::Strategy::NeedsCapacity));
        let mut dense = [8,8,8,8,1].repeat(514);
        raw(&dense,0,k,Some(repair::Strategy::DirectoryFull));
        dense.extend_from_slice(&[9;4]);
        raw(&dense,0,k,Some(repair::Strategy::DirectoryFull));
        let oversized = (0usize..1<<20).map(|i| (i%13) as u8).collect::<Vec<_>>();
        raw(&oversized,0,k,Some(repair::Strategy::Oversize));
        tests += 5;
    }
    println!("{tests} scalar and detected repair/capacity comparisons passed");
    let bad = common::craft(9,&[common::Spec::of(vec![b'A';5*1024+4],2)],None);
    marked(&bad,Some(mibz2::ErrorKind::RunUnfinished));
    let periodic = common::craft(9,&[common::Spec::of(vec![b'A';5*1024],999)],None);
    marked(&periodic,None);
    let finite = common::craft(9,&[common::Spec::of(vec![b'A';5],0)],None);
    // Explicit plain repair exercises the five-byte case; such a tiny column
    // is above opend's density gate and correctly goes to its ordinary fallback.
    assert!(common::oracle(&finite).is_ok());
    marked(&std::fs::read(std::env::args().nth(1).unwrap()).unwrap(),None);
    println!("Frozen and crafted integrated cases passed libbzip2 bytes, many LF cycles, nonempty and scalar fallback, wrong CRC, retry, late unfinished run and rollback");
}
