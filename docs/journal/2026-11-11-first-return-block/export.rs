//! Test-only export inside an isolated copy of the frozen library. No new unsafe.
#![forbid(unsafe_code)]

use crate::bwt::{Cells, InverseBwt, Segmented};
use crate::stream::{Event, Scanner, SliceSource};
use crate::{crc, decode_block, mtf, rle, runs, Kernels, Scratch};
use std::fs;
use std::path::PathBuf;

#[test]
fn export_first_qualifying_block() {
    let input = std::env::var("FIRST_RETURN_CORPUS").expect("corpus path");
    let dest = PathBuf::from(std::env::var("FIRST_RETURN_EXPORT").expect("export path"));
    fs::create_dir(&dest).expect("fresh export directory");
    let data = fs::read(input).expect("corpus");
    let mut scanner = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut scratch = Scratch::<Segmented>::default();
    let mut records = runs::Runs::default();
    let mut preceding_bytes = 0;
    let mut block = 0;
    loop {
        let Event::Block { symbols, bit, .. } = scanner.next(Vec::new()).expect("scan")
        else {
            panic!("no qualifying block");
        };
        let n = runs::front(&symbols, &mut records, k).expect("run front");
        let mut output = Vec::new();
        let checked_crc = decode_block(&symbols, k, &mut scratch, &mut output).expect("baseline CRC");
        if n < 65_536 {
            preceding_bytes += output.len();
            block += 1;
            continue;
        }
        let mut baseline_rle1 = Vec::new();
        assert!(Segmented::default().invert_runs(&records, n, symbols.orig_ptr, &mut baseline_rle1, k));
        assert_eq!(baseline_rle1.len(), n);
        let mut expanded = Vec::new();
        rle::unrle(&baseline_rle1, &mut expanded, k).expect("RLE1");
        assert_eq!(expanded, output);
        assert_eq!(crc::crc32(&expanded, k), checked_crc);
        assert_eq!(checked_crc, symbols.crc);
        let mut column = Vec::new();
        let mut counts = [0; 256];
        assert_eq!(mtf::mtf(&symbols, &mut column, &mut counts, k).unwrap(), n);
        let mut row = 0;
        let mut run_bytes = Vec::new();
        for &record in records.records() {
            let byte = (record >> 8) as u8;
            let len = (record & 255) as usize;
            assert!((1..=127).contains(&len));
            assert!(column[row..row + len].iter().all(|&c| c == byte));
            row += len;
            run_bytes.extend_from_slice(&record.to_le_bytes());
        }
        assert_eq!(row, n);
        let mut cells = Cells::default();
        let mut separate_comparator = Vec::new();
        assert!(cells.invert_runs(&records, n, symbols.orig_ptr, &mut separate_comparator, k));
        assert_eq!(separate_comparator, baseline_rle1);
        let admitted = cells.counts().admitted;
        fs::write(dest.join("runs.u16le"), run_bytes).unwrap();
        fs::write(dest.join("column.bin"), &column).unwrap();
        fs::write(dest.join("baseline-rle1.bin"), &baseline_rle1).unwrap();
        fs::write(dest.join("baseline-output.bin"), &output).unwrap();
        fs::write(dest.join("native.json"), format!(
            "{{\"block\":{block},\"bit\":{bit},\"n\":{n},\"primary\":{},\"records\":{},\"output_bytes\":{},\"preceding_output_bytes\":{preceding_bytes},\"crc\":{checked_crc},\"native_takes_runs\":{},\"cells_comparator_admitted\":{admitted},\"avx2\":{},\"clmul\":{},\"column_capacity_bytes\":{},\"rle1_capacity_bytes\":{},\"output_capacity_bytes\":{}}}\n",
            symbols.orig_ptr, records.records().len(), output.len(), runs::takes_runs(&symbols), k.avx2, k.clmul,
            column.capacity(), baseline_rle1.capacity(), output.capacity())).unwrap();
        break;
    }
}
