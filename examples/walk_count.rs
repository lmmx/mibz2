//! The staged cooperative walk's counting instrument
//! (docs/journal/2026-11-10-staged-traverse-count.md): no clock, run under callgrind.
//!
//! - `walk_count extract FILE OUT INDEX...`: the blocks numbered INDEX (from 0, across the
//!   file's streams) as a stream of their own, their bits untouched (a header at their
//!   level, their bits, an end marker and their combined CRC), so that a decoder arm can
//!   be counted on exactly those blocks.
//! - `walk_count spans FILE`: each block's index, bits, stored CRC, level, rows and class
//!   (T: a team's, at least `MIN_COOP` rows; S: the leader's solo), one JSON line each: the
//!   team screen's fixture map.
//! - `walk_count control FILE MODE SEGMENTS`: the same-segmentation single-caller control.
//!   Every block of at least `MIN_COOP` rows: its move-to-front and one-core build, then
//!   its walk on this thread alone, `MODE` one of
//!   - `old`: the team's walk, unstaged (`coop::traverse`), one share of `SEGMENTS`
//!     (`coop_research::walk_one_caller`);
//!   - `staged`: the same, staged (`coop::traverse_staged`);
//!   - `room`: the one-core room walk (`bwt::room_segments`, staged by #171) at `SEGMENTS`;
//!
//!   then the segments assembled and RLE1 undone. Prints the blocks, rows and decoded
//!   bytes, and a hash of the output (the same for every mode, or a mode is wrong).
use mibz2::bwt::{self, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, rle, Kernels};

struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl BitWriter {
    fn put(&mut self, v: u64, bits: u32) {
        for i in (0..bits).rev() {
            self.acc = self.acc << 1 | (v >> i & 1);
            self.n += 1;
            if self.n == 8 {
                self.out.push(self.acc as u8);
                self.acc = 0;
                self.n = 0;
            }
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.out.push((self.acc << (8 - self.n)) as u8);
        }
        self.out
    }
}

fn extract(file: &str, out: &str, want: &[u64]) {
    let data = std::fs::read(file).unwrap();
    let spans: Vec<_> = mibz2::scan(&data)
        .map(|s| s.unwrap())
        .filter(|s| want.contains(&s.index))
        .collect();
    assert_eq!(spans.len(), want.len(), "blocks not found");
    let level = spans[0].level;
    assert!(spans.iter().all(|s| s.level == level));
    let mut w = BitWriter {
        out: format!("BZh{level}").into_bytes(),
        acc: 0,
        n: 0,
    };
    let mut crc: u32 = 0;
    for s in &spans {
        let mut b = s.start_bit;
        while b < s.end_bit {
            let take = (s.end_bit - b).min(32);
            let mut v = 0u64;
            for i in 0..take {
                let bit = b + i;
                v = v << 1 | (data[(bit / 8) as usize] >> (7 - bit % 8) & 1) as u64;
            }
            w.put(v, take as u32);
            b += take;
        }
        crc = crc.rotate_left(1) ^ s.crc;
        println!(
            "block {}: bits {}..{}, crc {:08x}",
            s.index, s.start_bit, s.end_bit, s.crc
        );
    }
    w.put(0x1772_4538_5090, 48);
    w.put(crc as u64, 32);
    std::fs::write(out, w.finish()).unwrap();
}

#[inline(never)]
fn walk(
    mode: &str,
    tt: &mut TVector,
    orig_ptr: u32,
    s: &mut SegScratch,
    segments: usize,
    out: &mut Vec<u8>,
) {
    match mode {
        "old" | "staged" => {
            mibz2::coop_research::walk_one_caller(tt, orig_ptr, s, segments, mode == "staged");
            s.assemble(out);
        }
        "room" => bwt::walk_segments_room_with_prefetch::<32, { bwt::PF_L1 }>(
            tt, orig_ptr, out, s, segments,
        ),
        _ => panic!("MODE: old, staged or room"),
    }
}

fn control(file: &str, mode: &str, segments: usize) {
    let data = std::fs::read(file).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    let (mut tt, mut s, mut walked, mut out) = (
        TVector::new(),
        SegScratch::default(),
        Vec::new(),
        Vec::new(),
    );
    let (mut blocks, mut rows, mut decoded) = (0u64, 0u64, 0u64);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let n = mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        if n >= bwt::MIN_COOP {
            bwt::build(&ll[..n], &counts, &mut tt);
            walk(mode, &mut tt, b.orig_ptr, &mut s, segments, &mut walked);
            out.clear();
            rle::unrle(&walked, &mut out, k).unwrap();
            for &x in &out {
                h = (h ^ x as u64).wrapping_mul(0x100_0000_01b3);
            }
            blocks += 1;
            rows += n as u64;
            decoded += out.len() as u64;
        }
        syms = b.syms;
    }
    println!("mode {mode} segments {segments}: blocks {blocks} rows {rows} decoded {decoded} hash {h:016x}");
}

/// Every block's span and class, one JSON line each: its index, bits, stored CRC, level,
/// rows and class (`T`: at least `MIN_COOP` rows, walked by a team; `S`: below, the
/// leader's solo path): the team screen's fixture map
fn spans(file: &str) {
    let data = std::fs::read(file).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    let spans: Vec<_> = mibz2::scan(&data).map(|s| s.unwrap()).collect();
    let mut i = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let n = mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let s = &spans[i];
        let class = if n >= bwt::MIN_COOP { "T" } else { "S" };
        println!(
            "{{\"index\":{},\"start_bit\":{},\"end_bit\":{},\"crc\":\"{:08x}\",\"stream\":{},\"level\":{},\"rows\":{n},\"class\":\"{class}\"}}",
            s.index, s.start_bit, s.end_bit, s.crc, s.stream, s.level
        );
        syms = b.syms;
        i += 1;
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("extract") => {
            let idx: Vec<u64> = a[4..].iter().map(|x| x.parse().unwrap()).collect();
            extract(&a[2], &a[3], &idx)
        }
        Some("control") => control(&a[2], &a[3], a[4].parse().unwrap()),
        Some("spans") => spans(&a[2]),
        _ => eprintln!(
            "walk_count extract FILE OUT INDEX... | walk_count control FILE MODE SEGMENTS"
        ),
    }
}
