//! Safe diagnostic wrapper around the frozen production trait. Not an adoption benchmark.
#![forbid(unsafe_code)]

use mibz2::bwt::{Cells, CellsCounts, InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, runs, Kernels, Scratch};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::time::Instant;

const REV: &str = "57f51bbc0ca62ac3ee1cc148a1010842eea6222a";

trait Route: InverseBwt {
    fn counts(&self) -> CellsCounts {
        CellsCounts::default()
    }
    const NAME: &'static str;
}
impl Route for Segmented {
    const NAME: &'static str = "segmented";
}
impl Route for Cells {
    fn counts(&self) -> CellsCounts {
        self.counts()
    }
    const NAME: &'static str = "cells";
}

fn route(before: CellsCounts, after: CellsCounts, fallback: &'static str) -> &'static str {
    if after.admitted > before.admitted {
        "admitted"
    } else if after.density > before.density {
        "density"
    } else if after.floor > before.floor {
        "floor"
    } else if after.budget > before.budget {
        "budget"
    } else if after.invalid > before.invalid {
        "invalid"
    } else {
        fallback
    }
}

#[derive(Default)]
struct Observation {
    timed: bool,
    n: usize,
    route: &'static str,
    ns: Option<u128>,
    calls: usize,
}
thread_local! { static OBS: RefCell<Observation> = RefCell::new(Observation::default()); }

#[derive(Default)]
struct Observed<I: Route>(I);
impl<I: Route> Observed<I> {
    fn invoke<T>(&mut self, n: usize, fallback: &'static str, f: impl FnOnce(&mut I) -> T) -> T {
        let before = self.0.counts();
        let start = OBS.with(|o| o.borrow().timed.then(Instant::now));
        let result = f(&mut self.0);
        let ns = start.map(|t| t.elapsed().as_nanos());
        let selected = route(before, self.0.counts(), fallback);
        OBS.with(|o| {
            let mut o = o.borrow_mut();
            o.n = n;
            o.route = selected;
            o.ns = ns;
            o.calls += 1;
        });
        result
    }
}
impl<I: Route> InverseBwt for Observed<I> {
    fn reads_runs(&self) -> bool {
        self.0.reads_runs()
    }
    fn invert(&mut self, ll: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.invoke(ll.len(), "column", |i| i.invert(ll, c, p, out));
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.invoke(ll.len(), "column", |i| i.invert_in_place(ll, c, p))
    }
    fn invert_runs(
        &mut self,
        r: &runs::Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> bool {
        self.invoke(n, "records", |i| i.invert_runs(r, n, p, out, k))
    }
}

#[derive(Serialize)]
struct Block {
    block: usize,
    bit: u64,
    rows: usize,
    bytes: usize,
    route: &'static str,
    sha256: String,
    scan_ns: Option<u128>,
    decode_ns: Option<u128>,
    inversion_ns: Option<u128>,
}

fn pass<I: Route>(data: &[u8], timed: bool) -> Vec<Block> {
    let mut sc = Scanner::new(SliceSource(data));
    let mut scratch = Scratch::<Observed<I>>::default();
    let mut syms = Vec::new();
    let mut out = Vec::new();
    let mut blocks = Vec::new();
    let k = Kernels::detect();
    loop {
        let scan_start = timed.then(Instant::now);
        let event = sc.next(std::mem::take(&mut syms)).expect("scanner");
        let scan_ns = scan_start.map(|t| t.elapsed().as_nanos());
        let Event::Block { symbols, bit, .. } = event else {
            break;
        };
        OBS.with(|o| {
            *o.borrow_mut() = Observation {
                timed,
                ..Observation::default()
            }
        });
        out.clear();
        let start = timed.then(Instant::now);
        decode_block(&symbols, k, &mut scratch, &mut out).expect("block/CRC");
        let decode_ns = start.map(|t| t.elapsed().as_nanos());
        let (rows, route, inversion_ns) = OBS.with(|o| {
            let o = o.borrow();
            assert_eq!(o.calls, 1, "one actual inversion per valid block");
            (o.n, o.route, o.ns)
        });
        blocks.push(Block {
            block: blocks.len(),
            bit,
            rows,
            bytes: out.len(),
            route,
            sha256: format!("{:x}", Sha256::digest(&out)),
            scan_ns,
            decode_ns,
            inversion_ns,
        });
        syms = symbols.syms;
    }
    blocks
}

fn parity(a: &[Block], b: &[Block]) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        assert_eq!(
            (a.bit, a.rows, a.bytes, &a.sha256),
            (b.bit, b.rows, b.bytes, &b.sha256)
        );
    }
}

fn emit<I: Route>(data: &[u8], timed: bool, round: usize, oracle: &[Block]) {
    let blocks = pass::<I>(data, timed);
    parity(&blocks, oracle);
    for block in &blocks {
        println!(
            "{}",
            serde_json::json!({"kind": "block", "arm": I::NAME, "round": round, "data": block})
        );
    }
}

fn warm(data: &[u8], repeats: usize) {
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let mut records = runs::Runs::default();
    let (mut cells, mut seg) = (Cells::default(), Segmented::default());
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let k = Kernels::detect();
    let mut block = 0;
    while let Event::Block { symbols, bit, .. } =
        sc.next(std::mem::take(&mut syms)).expect("scanner")
    {
        if runs::takes_runs(&symbols) {
            let n = runs::front(&symbols, &mut records, k).expect("front");
            // Match the archived i9 census's minimum length. Never select by measured time.
            if n >= 65_536 {
                let before = cells.counts();
                let (mut ca, mut se) = (u128::MAX, u128::MAX);
                for r in 0..repeats {
                    let mut run_cells = || {
                        let t = Instant::now();
                        assert!(cells.invert_runs(&records, n, symbols.orig_ptr, &mut a, k));
                        ca = ca.min(t.elapsed().as_nanos());
                    };
                    let mut run_seg = || {
                        let t = Instant::now();
                        assert!(seg.invert_runs(&records, n, symbols.orig_ptr, &mut b, k));
                        se = se.min(t.elapsed().as_nanos());
                    };
                    if (r + block) % 2 == 0 {
                        run_cells();
                        run_seg();
                    } else {
                        run_seg();
                        run_cells();
                    }
                    assert_eq!(a, b);
                }
                println!(
                    "{}",
                    serde_json::json!({"kind":"warm", "block":block, "bit":bit,
                    "rows":n, "route":route(before, cells.counts(), "records"),
                    "repeats":repeats, "cells_min_ns":ca, "segmented_min_ns":se,
                    "parity":true})
                );
            }
        }
        syms = symbols.syms;
        block += 1;
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 3,
        "usage: cells-gap census|trace|warm FILE [PASSES=3]"
    );
    let mode = &args[1];
    assert!(["census", "trace", "warm"].contains(&mode.as_str()));
    let passes: usize = args.get(3).map_or(3, |s| s.parse().expect("passes"));
    assert!(passes > 0);
    let data = std::fs::read(&args[2]).expect("file");
    println!(
        "{}",
        serde_json::json!({"kind":"meta", "schema":1, "revision":REV,
        "mode":mode, "input_sha256":format!("{:x}", Sha256::digest(&data)),
        "timing_is_diagnostic_only":true})
    );
    if mode == "warm" {
        warm(&data, passes);
        return;
    }
    let oracle = pass::<Segmented>(&data, false);
    if mode == "census" {
        emit::<Cells>(&data, false, 0, &oracle);
    } else {
        for round in 0..passes {
            if round % 2 == 0 {
                emit::<Cells>(&data, true, round, &oracle);
                emit::<Segmented>(&data, true, round, &oracle);
            } else {
                emit::<Segmented>(&data, true, round, &oracle);
                emit::<Cells>(&data, true, round, &oracle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn real_decoder_parity_and_route_coverage() {
        let mut seed = 123u64;
        let random: Vec<_> = (0..70_000)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as u8
            })
            .collect();
        let mut admitted = false;
        let mut column = false;
        for word in [
            b"abacaba".repeat(20_000),
            b"0123456".repeat(20_000),
            vec![42; 100_000],
            random,
        ] {
            let mut z = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::best());
            z.write_all(&word).unwrap();
            let data = z.finish().unwrap();
            let cells = pass::<Cells>(&data, false);
            parity(&cells, &pass::<Segmented>(&data, false));
            let traced = pass::<Cells>(&data, true);
            parity(&cells, &traced);
            assert!(traced
                .iter()
                .all(|b| b.inversion_ns.unwrap() <= b.decode_ns.unwrap()));
            // Correctness smoke only: no duration is asserted or used as performance evidence.
            warm(&data, 2);
            assert_eq!(cells.iter().map(|b| b.bytes).sum::<usize>(), word.len());
            assert_eq!(cells.len(), 1);
            assert_eq!(cells[0].sha256, format!("{:x}", Sha256::digest(&word)));
            assert!(cells
                .iter()
                .all(|b| b.inversion_ns.is_none() && b.decode_ns.is_none()));
            admitted |= cells.iter().any(|b| b.route == "admitted");
            column |= cells.iter().any(|b| b.route == "column");
        }
        assert!(admitted && column);
    }
}
