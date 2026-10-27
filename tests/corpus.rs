//! The public API on data that has nothing to do with Wikidata: a text corpus (this
//! crate's own sources and README), random bytes, highly repetitive data and binary
//! records, each compressed by reference bzip2 (libbzip2 1.0.8) at levels 1, 6 and 9 and
//! decoded every way the crate offers.

mod common;
use bunzip::{BzReader, ErrorKind, Kernels};
use common::*;
use std::io::Read;

/// A reader giving at most 1-7 bytes a call (and, with `fail_at`, an error there)
struct Trickle<'a> {
    data: &'a [u8],
    at: usize,
    fail_at: Option<usize>,
}

impl Read for Trickle<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.fail_at.is_some_and(|f| self.at >= f) {
            return Err(std::io::Error::other("the source failed"));
        }
        let n = buf.len().min(1 + self.at % 7).min(self.data.len() - self.at);
        buf[..n].copy_from_slice(&self.data[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

#[test]
fn every_entry_point() {
    for (name, data) in corpora(1) {
        for level in [1, 6, 9] {
            let z = compress(&data, level);
            let ctx = format!("{name} level {level}");
            assert!(bunzip::decode(&z).unwrap() == data, "{ctx}: decode");
            assert!(bunzip::decode_with::<bunzip::bwt::Plain>(&z, Kernels::scalar()).unwrap() == data, "{ctx}: scalar plain");
            assert!(bunzip::decode_parallel(&z, 3).unwrap() == data, "{ctx}: decode_parallel");
            let mut out = Vec::new();
            BzReader::new(&z[..]).read_to_end(&mut out).unwrap();
            assert!(out == data, "{ctx}: BzReader");
            let mut out = Vec::new();
            BzReader::parallel(std::io::Cursor::new(z.clone()), 4).read_to_end(&mut out).unwrap();
            assert!(out == data, "{ctx}: BzReader::parallel");
            // the blocks found, then each decoded alone, in any order
            let spans: Vec<_> = bunzip::scan(&z).collect::<Result<_, _>>().unwrap();
            assert!(!spans.is_empty() || data.is_empty());
            let mut parts: Vec<Vec<u8>> = vec![Vec::new(); spans.len()];
            for (i, s) in spans.iter().enumerate().rev() {
                let r = bunzip::decode_blocks(&z, s.start_bit, s.end_bit, s.level, Kernels::detect(), &mut parts[i]).unwrap();
                assert_eq!(r.blocks, vec![(s.start_bit, s.end_bit, s.crc)]);
            }
            assert!(parts.concat() == data, "{ctx}: scan + decode_blocks");
        }
    }
}

#[test]
fn readers_on_short_reads_and_failing_sources() {
    let (_, data) = corpora(1).swap_remove(0);
    let z = compress(&data[..400_000], 9);
    let mut out = Vec::new();
    BzReader::new(Trickle { data: &z, at: 0, fail_at: None }).read_to_end(&mut out).unwrap();
    assert!(out == data[..400_000]);
    // a source that fails: its own error, not a decode error
    let e = BzReader::new(Trickle { data: &z, at: 0, fail_at: Some(z.len() / 2) }).read_to_end(&mut Vec::new()).unwrap_err();
    assert_eq!(e.to_string(), "the source failed");
    let owned: &'static [u8] = Box::leak(z.clone().into_boxed_slice());
    let e = BzReader::parallel(Trickle { data: owned, at: 0, fail_at: Some(z.len() / 2) }, 2)
        .read_to_end(&mut Vec::new())
        .unwrap_err();
    assert_eq!(e.to_string(), "the source failed");
    // a bad stream: InvalidData, wrapping the crate's error
    let mut bad = z.clone();
    bad[z.len() / 2] ^= 1;
    for r in [BzReader::new(&bad[..]).read_to_end(&mut Vec::new()), BzReader::parallel(std::io::Cursor::new(bad.clone()), 2).read_to_end(&mut Vec::new())] {
        let e = r.unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
        let inner = e.into_inner().unwrap().downcast::<bunzip::Error>().unwrap();
        assert!(matches!(inner.kind, ErrorKind::BlockCrc { .. } | ErrorKind::BadHuffmanCode | ErrorKind::BadOrigPtr
                                     | ErrorKind::BlockTooLong | ErrorKind::RunUnfinished | ErrorKind::SelectorOverrun),
                "{inner}");
    }
}
