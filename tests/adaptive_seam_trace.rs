#![cfg(feature = "trace")]

//! Isolated trace-feature integration test for adaptive seam window maintenance.
//!
//! Runs a single `StreamDecoder` in its own test binary so process-wide `mibz2::trace`
//! state (`trace::set_level` / `trace::take`) cannot observe concurrent decoders from other
//! library tests.

use bzip2::write::BzEncoder;
use bzip2::Compression;
use mibz2::trace::{self, Kind};
use mibz2::{Kernels, StreamDecoder};
use std::io::Write;
use std::sync::Arc;

/// Under `--features trace`, every `InputGot` and `WindowResume` span closes with a
/// matching `WindowWorkEnd`, and `window` capacity stays bounded by one chunk plus seam.
#[test]
fn traced_window_maintenance_spans_balance_and_stay_bounded() {
    let mut x = 0x243f_6a88_85a3_08d3u64;
    let mut raw = vec![0u8; 3_000_000];
    for b in &mut raw {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *b = (x & 0x1f) as u8;
    }
    let mut enc = BzEncoder::new(Vec::new(), Compression::new(9));
    enc.write_all(&raw).unwrap();
    let bz = enc.finish().unwrap();

    trace::set_level(1);
    let dec = Arc::new(StreamDecoder::new(2, 4, Kernels::detect()));
    let dec_w = Arc::clone(&dec);
    let bz_w = bz.clone();
    let feed = std::thread::spawn(move || {
        for c in bz_w.chunks(256 << 10) {
            dec_w.write(c.to_vec()).unwrap();
        }
        dec_w.close();
    });
    let mut out = Vec::new();
    let mut tmp = vec![0u8; 65_536];
    loop {
        let n = dec.read(&mut tmp).unwrap();
        if n == 0 {
            break;
        }
        out.extend_from_slice(&tmp[..n]);
    }
    feed.join().unwrap();
    drop(dec);
    assert_eq!(out, raw);

    let traces = trace::take();
    trace::set_level(0);
    let scanner = traces
        .iter()
        .find(|t| t.name == "mibz2-scan")
        .expect("scanner trace");
    let mut open = false;
    let mut spans = 0usize;
    let mut max_cap = 0u64;
    for m in &scanner.marks {
        match m.kind {
            Kind::InputGot | Kind::WindowResume => {
                assert!(!open, "nested window maintenance span");
                open = true;
            }
            Kind::WindowWorkEnd => {
                assert!(open, "stray WindowWorkEnd");
                open = false;
                spans += 1;
            }
            Kind::Window => {
                max_cap = max_cap.max(m.cap);
            }
            _ => {}
        }
    }
    assert!(!open && spans > 0, "unclosed or empty spans={spans}");
    assert!(
        max_cap <= (4 << 20),
        "expected bounded window cap on 256 KiB chunks, got {max_cap}"
    );
}
