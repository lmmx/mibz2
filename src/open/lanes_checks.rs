// The lanes' walks (`lanes.rs`) against the serial walk and the plain inverse: arbitrary
// tables, every small column at every row, arbitrary and periodic columns, and the admitted
// blocks.
use super::*;
use crate::bwt::{InverseBwt, Plain};
use std::panic::{catch_unwind, AssertUnwindSafe};

const WALKS: [Walk; 3] = [Walk::TwoPass, Walk::Pages, Walk::Pool];

fn plain(l: &[u8], p: u32) -> Vec<u8> {
    let mut counts = [0u32; 256];
    for &c in l {
        counts[c as usize] += 1;
    }
    let mut out = Vec::new();
    Plain::default().invert(l, &counts, p, &mut out);
    out
}

struct Lcg(u64);
impl Lcg {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n.max(1)
    }
}

/// Arbitrary tables: a dictionary of `words` words (0 to 40 bytes, across the 16-byte line),
/// `m` rows on a permutation (or, with `arbitrary`, any map), narrow or wide codes
struct Case {
    edges: Vec<u32>,
    extra: Vec<u32>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    bits: u32,
    mark: u32,
    n: usize,
}

fn case(g: &mut Lcg, max_m: usize, max_n: usize, arbitrary: bool, one_cycle: bool) -> Case {
    let words = g.below(40);
    let empty = g.below(50) == 0;
    let mut spelling = Vec::new();
    let mut offsets = vec![0u32];
    for _ in 0..words {
        let len = match g.below(4) {
            0 => 2 + g.below(15),
            1 => 15 + g.below(4),
            2 => 17 + g.below(24),
            _ => usize::from(!empty) * (1 + g.below(40)),
        };
        spelling.extend((0..len).map(|_| g.below(256) as u8));
        offsets.push(spelling.len() as u32);
    }
    let m = 1 + g.below(max_m);
    let bits = (usize::BITS - (m - 1).leading_zeros()).max(1);
    let wide = g.below(4) == 0;
    let codes = 256 + words;
    let mut perm: Vec<u32> = (0..m as u32).collect();
    if one_cycle {
        // Sattolo: one cycle through every row
        for i in (1..m).rev() {
            perm.swap(i, g.below(i));
        }
    } else {
        for i in (1..m).rev() {
            perm.swap(i, g.below(i + 1));
        }
    }
    let (mut edges, mut extra) = (vec![0u32; m], Vec::new());
    for (r, e) in edges.iter_mut().enumerate() {
        let to = if arbitrary {
            g.below(m) as u32
        } else {
            perm[r]
        };
        let code = if g.below(3) == 0 {
            g.below(256)
        } else {
            g.below(codes)
        } as u32;
        if wide {
            extra.push(code);
            *e = to;
        } else {
            *e = (code << bits) | to;
        }
    }
    let mark = g.below(m) as u32;
    let n = 1 + g.below(max_n);
    Case {
        edges,
        extra,
        spelling,
        offsets,
        bits,
        mark,
        n,
    }
}

/// The serial walk's bytes, or `None` if it panics
fn serial(c: &Case, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) -> Option<Vec<u8>> {
    catch_unwind(AssertUnwindSafe(|| {
        let mut out = vec![0x5A; c.n / 3];
        inline_words(&c.spelling, &c.offsets, inline16, lens);
        emit(
            &c.edges,
            &c.extra,
            &c.spelling,
            &c.offsets,
            inline16,
            lens,
            c.n,
            c.mark,
            c.bits,
            &mut out,
        );
        out
    }))
    .ok()
}

/// The walk `how`'s bytes (or `None` if it panics), whether the lanes took it, and the tables
/// restored exactly
fn by_lanes(
    lanes: &mut lanes::Lanes,
    how: Walk,
    c: &mut Case,
    dirty: &[u8],
) -> (Option<Vec<u8>>, bool) {
    let (edges, extra) = (c.edges.clone(), c.extra.clone());
    let (mut inline16, mut lens) = (Vec::new(), Vec::new());
    inline_words(&c.spelling, &c.offsets, &mut inline16, &mut lens);
    let (i0, l0) = (inline16.clone(), lens.clone());
    let mut took = false;
    let got = catch_unwind(AssertUnwindSafe(|| {
        let mut out = dirty.to_vec();
        took = emit_lanes(
            lanes,
            how,
            &mut c.edges,
            &mut c.extra,
            &c.spelling,
            &c.offsets,
            &mut inline16,
            &mut lens,
            c.n,
            c.mark,
            c.bits,
            &mut out,
        );
        out
    }))
    .ok();
    if got.is_some() {
        assert_eq!(c.edges, edges, "{how:?}: edges restored");
        assert_eq!(c.extra, extra, "{how:?}: extra restored");
        assert_eq!(inline16, i0, "{how:?}: inline16 restored");
        assert_eq!(lens, l0, "{how:?}: lens restored");
    }
    (got.map(|o| if took { o } else { Vec::new() }), took)
}

/// Each walk by lanes against the serial walk on arbitrary small tables: on a permutation,
/// the same bytes whenever the serial walk gives bytes (an empty walk panics in both); on an
/// arbitrary map, a walk that ends (the bound), the tables restored
#[test]
fn lanes_match_serial_on_arbitrary_tables() {
    let mut g = Lcg(0x6c61_6e65);
    let mut lanes = lanes::Lanes::default();
    let (mut inline16, mut lens) = (Vec::new(), Vec::new());
    let (mut cases, mut taken, mut abandoned) = (0usize, 0usize, 0usize);
    for i in 0..60_000 {
        let arbitrary = g.below(6) == 0;
        let span = if g.below(2) == 0 { 64 } else { 9000 };
        let mut c = case(&mut g, 300, span, arbitrary, i % 3 == 0);
        let want = serial(&c, &mut inline16, &mut lens);
        let dirty: Vec<u8> = (0..g.below(2 * c.n + 40)).map(|_| 0xA5).collect();
        for how in WALKS {
            let (got, took) = by_lanes(&mut lanes, how, &mut c, &dirty);
            if arbitrary {
                abandoned += usize::from(!took);
                continue;
            }
            if let Some(want) = &want {
                assert!(took, "case {i} {how:?}: a permutation is taken");
                assert_eq!(got.as_ref(), Some(want), "case {i} {how:?}");
                taken += 1;
            }
        }
        cases += 1;
    }
    println!("{cases} cases, {taken} walks by lanes, {abandoned} arbitrary maps abandoned");
    assert!(taken > 100_000);
}

/// The same on larger tables, one cycle or several, across many 4 KiB pages: dead `out`, the
/// overflow, evictions
#[test]
fn lanes_match_serial_across_pages() {
    let mut g = Lcg(0x7061_6765);
    let mut lanes = lanes::Lanes::default();
    let (mut inline16, mut lens) = (Vec::new(), Vec::new());
    let (mut evictions, mut peak) = (0, 0);
    for i in 0..400 {
        let mut c = case(&mut g, 40_000, 400_000, false, i % 2 == 0);
        let want = serial(&c, &mut inline16, &mut lens).expect("a permutation walks");
        for how in WALKS {
            let (got, took) = by_lanes(&mut lanes, how, &mut c, &[]);
            assert!(took, "case {i} {how:?}");
            assert_eq!(got.as_ref(), Some(&want), "case {i} {how:?}");
            if how == Walk::Pages {
                evictions += lanes.evictions;
                peak = peak.max(lanes.peak_slots);
            }
        }
    }
    println!("pages: {evictions} evictions, at most {peak} overflow pages");
    assert!(evictions > 0);
}

/// Every binary column through 10 rows and every ternary through 6, at every row and at limits
/// 0, 1, 4, 16 and 24, by each walk with the lanes from one row: exactly the plain inverse
#[test]
fn every_small_column_by_lanes() {
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let mut cases = 0usize;
    for (alphabet, top) in [(2u32, 10u32), (3, 6)] {
        for n in 1..=top {
            for code in 0..alphabet.pow(n) {
                let mut x = code;
                let l: Vec<u8> = (0..n)
                    .map(|_| {
                        let b = (x % alphabet) as u8;
                        x /= alphabet;
                        b
                    })
                    .collect();
                let r = Runs::of_column(&l);
                for p in 0..n {
                    let want = plain(&l, p);
                    for limit in [0, 1, 4, 16, 24] {
                        for how in WALKS {
                            out.clear();
                            out.extend(want.iter().map(|b| !b));
                            k.run_walk(how, 1, r.records(), l.len(), p, limit, &mut out);
                            assert_eq!(out, want, "{l:?} p {p} limit {limit} {how:?}");
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    println!("{cases} small cases");
}

fn runs_column(g: &mut Lcg, n: usize, alphabet: usize, mean: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    while l.len() < n {
        let c = g.below(alphabet) as u8;
        let w = 1 + g.below(2 * mean);
        l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
    }
    l
}

/// The BWT last column of `t` (rotations sorted), and the row of `t` itself
fn bwt(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    let mut rot: Vec<usize> = (0..n).collect();
    rot.sort_by(|&a, &b| {
        (0..n)
            .map(|i| t[(a + i) % n])
            .cmp((0..n).map(|i| t[(b + i) % n]))
            .then(a.cmp(&b))
    });
    let l = rot.iter().map(|&r| t[(r + n - 1) % n]).collect();
    let p = rot.iter().position(|&r| r == 0).unwrap() as u32;
    (l, p)
}

/// Arbitrary columns (runs of 1 to 10,000 rows, alphabets of 1 to 256: many cycles, the
/// marked one long or short) and BWTs of periodic texts (the walk repeated), by each walk:
/// exactly the plain inverse
#[test]
fn arbitrary_and_periodic_columns_by_lanes() {
    let mut g = Lcg(0x636f_6c73);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let mut cases = 0;
    for i in 0..240 {
        let n = match i % 4 {
            0 => 1 + g.below(5000),
            1 => 1 + g.below(60_000),
            2 => 1 + g.below(300_000),
            _ => 900_000 - g.below(1000),
        };
        let alphabet = [1, 2, 4, 11, 64, 256][g.below(6)];
        let mean = [1, 2, 4, 8, 30, 300, 5000][g.below(7)];
        let l = runs_column(&mut g, n, alphabet, mean);
        let r = Runs::of_column(&l);
        let p = g.below(n) as u32;
        let want = plain(&l, p);
        for limit in [16, 24] {
            for how in WALKS {
                out.clear();
                out.extend(want.iter().map(|b| !b));
                k.run_walk(how, 1, r.records(), n, p, limit, &mut out);
                assert_eq!(out, want, "case {i} n {n} a {alphabet} mean {mean} {how:?}");
                cases += 1;
            }
        }
    }
    for i in 0..300 {
        let sigma = 1 + g.below(4);
        let u: Vec<u8> = (0..1 + g.below(40))
            .map(|_| b"abcd"[g.below(sigma)])
            .collect();
        let t: Vec<u8> = u
            .iter()
            .copied()
            .cycle()
            .take(u.len() * (1 + g.below(50)))
            .collect();
        let (l, p) = bwt(&t);
        let r = Runs::of_column(&l);
        let want = plain(&l, p);
        assert_eq!(want, t);
        for how in WALKS {
            out.clear();
            k.run_walk(how, 1, r.records(), l.len(), p, LIMIT, &mut out);
            assert_eq!(out, want, "periodic {i} {how:?}");
            cases += 1;
        }
    }
    println!("{cases} cases");
}

/// (ignored; `MIBZ2_ADMITTED_BWT`) the six admitted Wikidata columns at their primary index,
/// by each walk with the lanes as shipped: exactly the plain inverse
#[test]
#[ignore]
fn lanes_admitted_blocks() {
    let dir = std::env::var("MIBZ2_ADMITTED_BWT").expect("MIBZ2_ADMITTED_BWT");
    let manifest = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut k = Kernel::default();
    let mut blocks = 0;
    for part in manifest.split("\"file\": \"").skip(1) {
        let file = &part[..part.find('"').unwrap()];
        let at = part.find("\"primary_index\": ").unwrap() + 17;
        let p: u32 = part[at..]
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let l = std::fs::read(format!("{dir}/{file}")).unwrap();
        let r = Runs::of_column(&l);
        let want = plain(&l, p);
        for limit in [16, 24] {
            for how in [Walk::Serial, Walk::TwoPass, Walk::Pages, Walk::Pool] {
                let mut out = Vec::new();
                let s = k.run_walk(how, LANES_FROM, r.records(), l.len(), p, limit, &mut out);
                assert_eq!(out, want, "{file} {how:?}");
                if how == Walk::Pages {
                    eprintln!(
                        "{file} limit {limit}: n {}, m {}, thick {}, words {}; pages: {} \
                         overflow pages at most ({} KiB), {} evictions; out capacity {} KiB",
                        l.len(),
                        s.retained,
                        s.thick,
                        s.words,
                        k.lanes.peak_slots,
                        k.lanes.peak_slots * lanes::PAGE / 1024,
                        k.lanes.evictions,
                        out.capacity() / 1024
                    );
                }
            }
        }
        blocks += 1;
    }
    assert_eq!(blocks, 6);
}

/// (ignored; timing, `MIBZ2_ADMITTED_BWT`) the admitted blocks, hot, one thread: the
/// construction and each walk, best of 31 per block, the walks interleaved
#[test]
#[ignore]
fn lanes_timing() {
    use std::time::Instant;
    let dir = std::env::var("MIBZ2_ADMITTED_BWT").expect("MIBZ2_ADMITTED_BWT");
    let limit: u32 = std::env::var("MIBZ2_LIMIT").map_or(LIMIT, |s| s.parse().unwrap());
    let manifest = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut k = Kernel::default();
    let walks = [Walk::Serial, Walk::TwoPass, Walk::Pages, Walk::Pool];
    let flush = std::env::var("MIBZ2_FLUSH").is_ok();
    let mut junk = vec![0u8; if flush { 32 << 20 } else { 0 }];
    eprintln!("limit {limit}; block, construct, then {walks:?} (ms, best of 31)");
    let mut sums = [0f64; 5];
    for part in manifest.split("\"file\": \"").skip(1) {
        let file = &part[..part.find('"').unwrap()];
        let at = part.find("\"primary_index\": ").unwrap() + 17;
        let p: u32 = part[at..]
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let l = std::fs::read(format!("{dir}/{file}")).unwrap();
        let r = Runs::of_column(&l);
        let n = l.len();
        let mut best = [f64::INFINITY; 5];
        let mut out = Vec::new();
        for round in 0..31 {
            // each walk straight after its own construction, as a decode runs them
            for i in 0..4 {
                let w = (i + round) % 4;
                if flush {
                    // another kernel's working set between blocks, as in a decode
                    for (x, v) in junk.iter_mut().enumerate() {
                        *v = v.wrapping_add(x as u8);
                    }
                }
                let s = Instant::now();
                let (_, mark, bits) = k.construct::<true, true>(r.records(), n as u32, p, limit);
                best[0] = best[0].min(s.elapsed().as_secs_f64());
                let s = Instant::now();
                k.walk_with(walks[w], LANES_FROM, n, mark, bits, &mut out);
                best[w + 1] = best[w + 1].min(s.elapsed().as_secs_f64());
            }
        }
        for (s, b) in sums.iter_mut().zip(best) {
            *s += b;
        }
        eprintln!(
            "{file}\t{}",
            best.map(|b| format!("{:.3}", b * 1e3)).join("\t")
        );
    }
    eprintln!(
        "sum\t{}",
        sums.map(|b| format!("{:.3}", b * 1e3)).join("\t")
    );
}
