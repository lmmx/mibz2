//! D4-delta, a counting probe (src/d4_delta.rs;
//! docs/journal/2026-11-10-d4-delta-count-registration.md). Untimed: nothing here is a timing.
//!
//! ```text
//! d4_delta check FILE.bz2 ...      every block: the D4 walks (production's choice, the room
//!                                  walk forced, the plain walk) against bwt::walk byte for
//!                                  byte, then RLE1 and the block's CRC
//! d4_delta census FILE.bz2 ...     per file: pieces at levels 1, 2, 4, records, the table's
//!                                  bytes and the doubling scratch, per block and in all
//! d4_delta count FILE.bz2 BLOCKS   for callgrind: the first BLOCKS blocks' builds and walks
//!                                  (production's build and room walk, and D4's), each in a
//!                                  function of its own, whose inclusive instruction counts
//!                                  are the routines' costs
//! ```

use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::d4_delta::{D4Table, WalkScratch, LANES, SEGMENTS};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

/// Each block of the file: its last column, its counts, `orig_ptr` and CRC
fn blocks(data: &[u8], mut f: impl FnMut(&[u8], &[u32; 256], u32, u32)) {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        f(&ll, &counts, b.orig_ptr, b.crc);
        syms = b.syms;
    }
}

fn check(path: &str, data: &[u8]) {
    let k = Kernels::detect();
    let (mut tt, mut d, mut w) = (TVector::new(), D4Table::new(), WalkScratch::default());
    let (mut want, mut got, mut out) = (Vec::new(), Vec::new(), Vec::new());
    let (mut nblocks, mut rows, mut walks) = (0, 0u64, 0u64);
    blocks(data, |ll, counts, orig, crc_want| {
        build(ll, counts, &mut tt);
        walk(&tt, orig, &mut want);
        let mut same = |got: &Vec<u8>, what: &str| {
            assert!(
                got == &want,
                "{path}: block {nblocks}: {what} differs from bwt::walk"
            );
            walks += 1;
        };
        assert!(
            d.build_for_walk(ll, counts, orig),
            "{path}: the build refused"
        );
        d.walk(&mut got, &mut w);
        same(&got, "the walk");
        d.walk_plain(&mut got, &mut w);
        same(&got, "the plain walk");
        assert!(d.build(ll, counts, orig, SEGMENTS));
        d.walk_room::<LANES>(&mut got, &mut w);
        same(&got, "the room walk, forced");
        d.walk_plain(&mut got, &mut w);
        same(&got, "the plain walk, 512 segments");
        out.clear();
        rle::unrle(&got, &mut out, k).unwrap();
        assert_eq!(
            crc::crc32(&out, k),
            crc_want,
            "{path}: block {nblocks}: CRC"
        );
        nblocks += 1;
        rows += ll.len() as u64;
    });
    println!(
        "{path}: {nblocks} blocks, {rows} rows: the D4 walks equal bwt::walk's on every block \
         ({walks} walks), and RLE1's output matches every block's CRC"
    );
}

/// Heights of tunnel stretches along the plain walk (post hoc, descriptive): the steps over
/// which the path and a neighbour path stay in one piece
#[derive(Default)]
struct Heights {
    steps: u64,
    bytes: u64,
    stretches: u64,
    /// steps and bytes in stretches of at least 33 steps, and their count
    tall_steps: u64,
    tall_bytes: u64,
    tall: u64,
    /// the same at a threshold of 9 steps (at least 33 bytes)
    tall9_steps: u64,
    tall9_bytes: u64,
    tall9: u64,
    /// steps in stretches by height class: 1, 2-8, 9-32, 33+
    by: [u64; 4],
}

impl Heights {
    fn add(&mut self, h: u64, b: u64) {
        self.stretches += 1;
        if h >= 33 {
            self.tall_steps += h;
            self.tall_bytes += b;
            self.tall += 1;
        }
        if h >= 9 {
            self.tall9_steps += h;
            self.tall9_bytes += b;
            self.tall9 += 1;
        }
        self.by[match h {
            1 => 0,
            2..=8 => 1,
            9..=32 => 2,
            _ => 3,
        }] += h;
    }
    fn merge(&mut self, o: &Heights) {
        self.steps += o.steps;
        self.bytes += o.bytes;
        self.stretches += o.stretches;
        self.tall_steps += o.tall_steps;
        self.tall_bytes += o.tall_bytes;
        self.tall += o.tall;
        self.tall9_steps += o.tall9_steps;
        self.tall9_bytes += o.tall9_bytes;
        self.tall9 += o.tall9;
        for i in 0..4 {
            self.by[i] += o.by[i];
        }
    }
}

fn heights(d: &D4Table) -> Heights {
    let mut h = Heights::default();
    let mut path = Vec::new();
    d.trace(&mut path);
    // a stretch: consecutive steps whose rows keep a neighbour (the next row, or the one
    // before) in their own piece, the same side throughout
    let (mut len, mut bytes, mut right, mut left) = (0u64, 0u64, false, false);
    for &(row, n) in &path {
        let (r, l) = (
            d.same_piece(row, row + 1),
            row > 0 && d.same_piece(row, row - 1),
        );
        let (r2, l2) = (right && r, left && l);
        if len > 0 && (r2 || l2) {
            (right, left) = (r2, l2);
        } else {
            if len > 0 {
                h.add(len, bytes);
            }
            (len, bytes, right, left) = (0, 0, r, l);
        }
        len += 1;
        bytes += n as u64;
        h.steps += 1;
        h.bytes += n as u64;
        if !(right || left) {
            h.add(len, bytes);
            len = 0;
            bytes = 0;
        }
    }
    if len > 0 {
        h.add(len, bytes);
    }
    h
}

fn census(path: &str, data: &[u8], first: usize) {
    let mut d = D4Table::new();
    let (mut all, mut nblocks) = (mibz2::d4_delta::Census::default(), 0u64);
    let (mut over, mut full, mut maxcap) = (0u64, 0u64, 0u64);
    let mut hs = Heights::default();
    let mut hfirst = Heights::default();
    let mut un = [0u64; 4];
    let mut fulls: Vec<(u64, u64, u64, u64, u64)> = Vec::new();
    let name = std::path::Path::new(path)
        .file_name()
        .map_or(path.into(), |f| f.to_string_lossy());
    blocks(data, |ll, counts, orig, _| {
        assert!(d.build(ll, counts, orig, SEGMENTS));
        let c = d.census();
        let cap = c.capacity();
        if (nblocks as usize) < first {
            println!(
                "| {name} block {nblocks} | {} | {:.4} | {:.4} | {:.4} | {} | {} | {} | {} | {} | {} |",
                c.rows,
                c.r1 as f64 / c.rows as f64,
                c.r2 as f64 / c.rows as f64,
                c.r4 as f64 / c.rows as f64,
                c.records - c.r4,
                c.used(),
                cap,
                cap + 8 * c.rows.div_ceil(64),
                c.scratch_bytes,
                c.records_alloc,
            );
        }
        let ((a, pa), (b, pb)) = d.unsplit();
        un[0] += a;
        un[1] += pa;
        un[2] += b;
        un[3] += pb;
        let h = heights(&d);
        if (nblocks as usize) < first {
            hfirst.merge(&h);
        }
        hs.merge(&h);
        if c.rows >= 899_000 {
            fulls.push((cap, c.r1, c.r2, c.r4, c.used()));
            full += 1;
            maxcap = maxcap.max(cap);
            if cap > 1_200_000 {
                over += 1;
            }
        }
        all.rows += c.rows;
        all.r1 += c.r1;
        all.r2 += c.r2;
        all.r4 += c.r4;
        all.records += c.records;
        nblocks += 1;
    });
    let n = all.rows as f64;
    println!(
        "{name}: {nblocks} blocks, {} rows: r1 {:.4} n, r2 {:.4} n, r4 {:.4} n, records {:.4} n; \
         full blocks {full}, admitted (capacity <= 1.2 MB) {}, largest capacity {maxcap} B",
        all.rows,
        all.r1 as f64 / n,
        all.r2 as f64 / n,
        all.r4 as f64 / n,
        all.records as f64 / n,
        full - over,
    );
    if !fulls.is_empty() {
        let med = |f: fn(&(u64, u64, u64, u64, u64)) -> u64| {
            let mut v: Vec<u64> = fulls.iter().map(f).collect();
            v.sort_unstable();
            let q = |p: usize| v[(v.len() - 1) * p / 100];
            format!("{} / {} / {} / {} / {}", q(0), q(10), q(50), q(90), q(100))
        };
        println!(
            "{name}: full blocks (min / p10 / median / p90 / max): r1 {}; r2 {}; r4 {}; used B {}; admitted capacity B {}",
            med(|x| x.1),
            med(|x| x.2),
            med(|x| x.3),
            med(|x| x.4),
            med(|x| x.0)
        );
    }
    println!(
        "{name}: parents passing whole, 1 -> 2: {:.3}; 2 -> 4: {:.3} (a split at a singleton counts as a split)",
        un[0] as f64 / un[1] as f64,
        un[2] as f64 / un[3] as f64
    );
    for (what, h) in [("first blocks", &hfirst), ("all blocks", &hs)] {
        let rows = h.bytes as f64;
        println!(
            "{name} ({what}) heights, post hoc: steps {:.4} a row; stretches {}; steps by height 1 / 2-8 / 9-32 / 33+: {:.3} / {:.3} / {:.3} / {:.3}; \
             bytes in stretches of 33+ steps {:.1}% (accesses a row, one a tall stretch: {:.4}); \
             bytes in stretches of 9+ steps (33+ bytes) {:.1}% (accesses a row: {:.4})",
            h.steps as f64 / rows,
            h.stretches,
            h.by[0] as f64 / h.steps as f64,
            h.by[1] as f64 / h.steps as f64,
            h.by[2] as f64 / h.steps as f64,
            h.by[3] as f64 / h.steps as f64,
            100.0 * h.tall_bytes as f64 / rows,
            (h.steps - h.tall_steps + h.tall) as f64 / rows,
            100.0 * h.tall9_bytes as f64 / rows,
            (h.steps - h.tall9_steps + h.tall9) as f64 / rows,
        );
    }
}

#[inline(never)]
fn count_t_build(ll: &[u8], counts: &[u32; 256], tt: &mut TVector) {
    build(ll, counts, tt)
}
#[inline(never)]
fn count_t_walk(tt: &mut TVector, orig: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    walk_segments_room(tt, orig, out, s)
}
#[inline(never)]
fn count_d4_build(ll: &[u8], counts: &[u32; 256], orig: u32, d: &mut D4Table) {
    assert!(d.build(ll, counts, orig, SEGMENTS));
}
#[inline(never)]
fn count_d4_walk(d: &D4Table, out: &mut Vec<u8>, w: &mut WalkScratch) {
    d.walk_room::<LANES>(out, w)
}

#[inline(never)]
fn phase_build(ll: &[u8], counts: &[u32; 256], orig: u32, d: &mut D4Table, upto: u8) {
    d.build_partial(ll, counts, orig, upto);
}

/// The build's steps by difference (post hoc, descriptive): `phase_build` is called once a
/// block for each of steps 1 to 5 (each time from the start), as `phase_1` ... `phase_5`
#[inline(never)]
fn phase_1(ll: &[u8], c: &[u32; 256], o: u32, d: &mut D4Table) {
    phase_build(ll, c, o, d, 1)
}
#[inline(never)]
fn phase_2(ll: &[u8], c: &[u32; 256], o: u32, d: &mut D4Table) {
    phase_build(ll, c, o, d, 2)
}
#[inline(never)]
fn phase_3(ll: &[u8], c: &[u32; 256], o: u32, d: &mut D4Table) {
    phase_build(ll, c, o, d, 3)
}
#[inline(never)]
fn phase_4(ll: &[u8], c: &[u32; 256], o: u32, d: &mut D4Table) {
    phase_build(ll, c, o, d, 4)
}
#[inline(never)]
fn phase_5(ll: &[u8], c: &[u32; 256], o: u32, d: &mut D4Table) {
    phase_build(ll, c, o, d, 5)
}

fn phases(path: &str, data: &[u8], max: usize) {
    let mut d = D4Table::new();
    let (mut nblocks, mut rows) = (0, 0);
    blocks(data, |ll, counts, orig, _| {
        if nblocks == max {
            return;
        }
        phase_1(ll, counts, orig, &mut d);
        phase_2(ll, counts, orig, &mut d);
        phase_3(ll, counts, orig, &mut d);
        phase_4(ll, counts, orig, &mut d);
        phase_5(ll, counts, orig, &mut d);
        count_d4_build(ll, counts, orig, &mut d);
        nblocks += 1;
        rows += ll.len();
    });
    println!("{path}: {nblocks} blocks, {rows} rows: partial builds counted");
}

fn count(path: &str, data: &[u8], max: usize) {
    let (mut tt, mut ts, mut d, mut w) = (
        TVector::new(),
        SegScratch::default(),
        D4Table::new(),
        WalkScratch::default(),
    );
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let (mut nblocks, mut rows) = (0, 0);
    blocks(data, |ll, counts, orig, _| {
        if nblocks == max {
            return;
        }
        count_t_build(ll, counts, &mut tt);
        count_t_walk(&mut tt, orig, &mut a, &mut ts);
        count_d4_build(ll, counts, orig, &mut d);
        count_d4_walk(&d, &mut b, &mut w);
        assert!(a == b, "{path}: the D4 walk differs");
        nblocks += 1;
        rows += ll.len();
    });
    println!("{path}: {nblocks} blocks, {rows} rows counted");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: d4_delta check|census|count FILE.bz2 ...";
    let cmd = args.get(1).expect(usage).as_str();
    assert!(args.len() > 2, "{usage}");
    if cmd == "census" {
        println!(
            "| block | rows | r1/n | r2/n | r4/n | split records | used B | admitted capacity B \
             | with a 20 B directory | doubling scratch B (allocated) | records allocated B |"
        );
        println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
    }
    for path in &args[2..] {
        let data = std::fs::read(path).unwrap();
        match cmd {
            "check" => check(path, &data),
            "census" => census(path, &data, 3),
            "phases" => {
                phases(path, &data, args.get(3).map_or(3, |s| s.parse().unwrap()));
                break;
            }
            "count" => {
                count(path, &data, args.get(3).map_or(3, |s| s.parse().unwrap()));
                break;
            }
            _ => panic!("{usage}"),
        }
    }
}
