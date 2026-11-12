//! A census of the proposed 30-entry compact navigation layout (external report 156), with a
//! reference round trip. Untimed: it counts and checks, and measures nothing about speed.
//!
//!     cargo run --release --example census30 -- OUT.jsonl FILE.bz2 [FILE.bz2 ...]
//!
//! **The layout (report 156, §3–§6):** one aligned 64-byte record per 30 rows (in F order).
//! - **Payload:** 30 `u16` fields in bytes 0–59; a `u32` header in bytes 60–63.
//! - **The fields:** an offset 0..=65533 on an ordinary record; 0xFFFE a segment start;
//!   0xFFFF a RAW row (or the parking record).
//! - **The header:**
//!   - ordinary: bits 0–19 the encoded successor base B, bits 20–27 the record's F byte;
//!   - RAW: bits 0–19 the RAW group's index, bit 28 set.
//! - **The labels:** row r is labelled phi(r) = 32·⌊r/30⌋ + r mod 30, so the field is at
//!   byte 2·phi(r) and the header at 2·(phi & !31) + 60.
//! - **The successors:** stored in encoded coordinates. The packed token is
//!   q = phi(row) | carry << 20, and an ordinary step is q' = header + field.
//! - **A RAW group:** 128 bytes, 32 `u32` tokens (30 used), each phi(Ψ(r)) | F[r] << 20.
//! - **Parking:** one record of 0xFFFF fields after the last, at label 32·G.
//!
//! **What it records, per block:**
//! - the rows and the used alphabet σ;
//! - the groups: ordinary, RAW from a crossing, RAW from a too-wide span;
//! - the partial group's unused fields;
//! - the logical bytes 64(G + 1) + 128R, and bytes a row;
//! - the bound R ≤ min(G, σ⌊M/65534⌋ + σ − 1), with its margin.
//!
//! **What it checks (any failure panics):**
//! - Ψ, from this file's own counting scatter, is increasing within every F bucket;
//! - the label limits: the largest live label and the parking label are below 2^20;
//! - every stored successor's label avoids the two header positions (mod 32 ≤ 29);
//! - every ordinary offset is ≤ 65533, so it is never confused with 0xFFFE or 0xFFFF;
//! - token arithmetic: header + offset = phi(Ψ(r)) | F[r] << 20 with no carry, for every
//!   ordinary row;
//! - the round trip:
//!   - an unoptimised scalar reference walk of the records alone, seeded from L at the
//!     chain's start, emits exactly production's inverse-BWT output (`bwt::Segmented`);
//!   - a second walk does the same with about 512 rows marked 0xFFFE (each saved, and
//!     restored on the slow path), ordinary and RAW rows alike.
//!
//! **What it does not do:** no optimised traversal, no lanes, no direct builder, no timing.
//! The records are built from Ψ, as an oracle builds them.

use mibz2::bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::io::Write;

const ROWS: usize = 30;
const START: u16 = 0xFFFE;
const RAWF: u16 = 0xFFFF;
const MAX_OFF: u32 = 65_533;
const RAW_BIT: u32 = 1 << 28;
const ROW_MASK: u32 = (1 << 20) - 1;

fn phi(r: usize) -> u32 {
    (32 * (r / ROWS) + r % ROWS) as u32
}

struct Records {
    /// 32 u16 a record: fields 0..30, and the header's two halves in 30 and 31
    words: Vec<u16>,
    /// 32 u32 tokens a RAW group
    raw: Vec<u32>,
}

impl Records {
    fn header(&self, x: u32) -> u32 {
        let b = (x & !31) as usize;
        self.words[b + 30] as u32 | (self.words[b + 31] as u32) << 16
    }
    fn set_header(&mut self, g: usize, h: u32) {
        self.words[32 * g + 30] = h as u16;
        self.words[32 * g + 31] = (h >> 16) as u16;
    }
}

#[derive(Default)]
struct Census {
    rows: usize,
    sigma: usize,
    groups: usize,
    ordinary: usize,
    raw_crossing: usize,
    raw_wide: usize,
    partial_unused: usize,
    bytes: usize,
    bound: usize,
    max_label: u32,
    parking_label: u32,
    max_span_ordinary: u32,
    raw_rows: usize,
    starts_marked: usize,
    starts_on_raw: usize,
}

/// The records of one block, from Ψ and F, and its census (the checks panic)
fn encode(ll: &[u8], counts: &[u32; 256]) -> (Records, Census, Vec<usize>) {
    let n = ll.len();
    // Ψ by counting scatter: F-row s holds the L position of its occurrence
    let mut ftab = [0usize; 256];
    let mut cum = 0;
    for c in 0..256 {
        ftab[c] = cum;
        cum += counts[c] as usize;
    }
    assert_eq!(cum, n, "counts sum to n");
    let lo = ftab;
    let mut psi = vec![0usize; n];
    let mut f = vec![0u8; n];
    for (i, &c) in ll.iter().enumerate() {
        let s = ftab[c as usize];
        psi[s] = i;
        f[s] = c;
        ftab[c as usize] += 1;
    }
    for c in 0..256 {
        assert_eq!(ftab[c], lo[c] + counts[c] as usize, "bucket {c} filled");
        for s in lo[c] + 1..ftab[c] {
            assert!(psi[s - 1] < psi[s], "Ψ increasing within bucket {c}");
        }
    }
    let sigma = counts.iter().filter(|&&c| c > 0).count();
    let g_count = n.div_ceil(ROWS);
    let mut rec = Records {
        words: vec![RAWF; 32 * (g_count + 1)],
        raw: Vec::new(),
    };
    let mut c = Census {
        rows: n,
        sigma,
        groups: g_count,
        ..Default::default()
    };
    c.max_label = if n > 0 { phi(n - 1) } else { 0 };
    c.parking_label = (32 * g_count) as u32;
    assert!(
        c.max_label <= ROW_MASK && c.parking_label <= ROW_MASK,
        "20-bit labels"
    );
    for g in 0..g_count {
        let (a, b) = (g * ROWS, ((g + 1) * ROWS).min(n));
        c.partial_unused += ROWS - (b - a);
        let succ: Vec<u32> = (a..b).map(|s| phi(psi[s])).collect();
        for &x in &succ {
            assert!(x & 31 < 30, "a successor on a header position");
        }
        let crossing = f[a..b].iter().any(|&x| x != f[a]);
        let (base, span) = if crossing {
            (0, 0)
        } else {
            // homogeneous: Ψ increasing, so the first is the least, the last the greatest
            (succ[0], succ[succ.len() - 1] - succ[0])
        };
        if !crossing && span <= MAX_OFF {
            c.ordinary += 1;
            c.max_span_ordinary = c.max_span_ordinary.max(span);
            let h = base | (f[a] as u32) << 20;
            rec.set_header(g, h);
            for (k, &x) in succ.iter().enumerate() {
                let o = x - base;
                assert!(o <= MAX_OFF, "an ordinary offset over 65533");
                // the token identity: no carry into the byte field
                assert_eq!(h + o, x | (f[a + k] as u32) << 20, "token arithmetic");
                rec.words[32 * g + k] = o as u16;
            }
        } else {
            if crossing {
                c.raw_crossing += 1;
            } else {
                c.raw_wide += 1;
            }
            c.raw_rows += b - a;
            let idx = rec.raw.len() / 32;
            assert!(idx as u32 <= ROW_MASK);
            rec.set_header(g, idx as u32 | RAW_BIT);
            for (k, &x) in succ.iter().enumerate() {
                rec.words[32 * g + k] = RAWF;
                rec.raw.push(x | (f[a + k] as u32) << 20);
            }
            rec.raw.resize(32 * (idx + 1), 0);
        }
    }
    // parking: its fields are already 0xFFFF; its header has the RAW bit set, and no group
    rec.set_header(g_count, RAW_BIT | ROW_MASK);
    let r = c.raw_crossing + c.raw_wide;
    c.bytes = 64 * (g_count + 1) + 128 * r;
    let m = c.max_label as usize;
    c.bound = g_count.min(sigma * (m / 65_534) + sigma.saturating_sub(1));
    assert!(r <= c.bound, "the RAW-count bound fails: {r} > {}", c.bound);
    // the row each output step visits, for marking starts
    (rec, c, psi)
}

/// The reference walk: n bytes from the records alone, seeded at the chain's start
/// x0 = Ψ(orig_ptr) with its byte L[x0]. With `saved`, a field holding START is a marked
/// start: its original value is restored from `saved` on the slow path.
fn walk(rec: &Records, ll: &[u8], psi0: usize, n: usize, saved: &[(u32, u16)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(n);
    let mut q = phi(psi0) | (ll[psi0] as u32) << 20;
    for _ in 0..n {
        out.push((q >> 20) as u8);
        let x = q & ROW_MASK;
        let mut o = rec.words[x as usize];
        if o == START {
            // the slow path tests START first, then restores the row's own field
            let i = saved
                .binary_search_by_key(&x, |s| s.0)
                .expect("a marked start");
            o = saved[i].1;
        }
        q = if o < START {
            rec.header(x) + o as u32
        } else {
            debug_assert_eq!(o, RAWF);
            let h = rec.header(x);
            assert!(h & RAW_BIT != 0, "a 0xFFFF field on an ordinary record");
            let gi = (h & ROW_MASK) as usize;
            rec.raw[32 * gi + (x & 31) as usize]
        };
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_path = &args[1];
    let mut jl = std::fs::File::create(out_path).unwrap();
    let k = Kernels::detect();
    for file in &args[2..] {
        let data = std::fs::read(file).unwrap();
        let mut sc = Scanner::new(SliceSource(&data));
        let mut syms = Vec::new();
        let mut inv = Segmented::default();
        let (mut ll, mut counts, mut want) = (Vec::new(), [0u32; 256], Vec::new());
        let mut blk = 0usize;
        let mut tot = Census::default();
        while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
            let n = mibz2::mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
            inv.invert(&ll, &counts, symbols.orig_ptr, &mut want);
            assert_eq!(want.len(), n);
            let (mut rec, mut c, psi) = encode(&ll, &counts);
            // the walk from the records alone equals production's output
            let x0 = psi[symbols.orig_ptr as usize];
            assert!(
                walk(&rec, &ll, x0, n, &[]) == want,
                "{file} block {blk}: the reference walk differs"
            );
            // again with about 512 rows along the chain marked as segment starts
            let step = (n / 512).max(1);
            let mut saved = Vec::new();
            let mut x = x0;
            for i in 0..n {
                if i % step == step / 2 {
                    let lab = phi(x);
                    if saved.iter().all(|s: &(u32, u16)| s.0 != lab) {
                        let w = rec.words[lab as usize];
                        c.starts_on_raw += (w == RAWF) as usize;
                        saved.push((lab, w));
                    }
                }
                x = psi[x];
            }
            for &(lab, _) in &saved {
                rec.words[lab as usize] = START;
            }
            saved.sort_unstable();
            c.starts_marked = saved.len();
            assert!(
                walk(&rec, &ll, x0, n, &saved) == want,
                "{file} block {blk}: the walk with marked starts differs"
            );
            for &(lab, w) in &saved {
                rec.words[lab as usize] = w;
            }
            writeln!(
                jl,
                "{{\"file\":{:?},\"block\":{},\"rows\":{},\"sigma\":{},\"groups\":{},\"ordinary\":{},\"raw_crossing\":{},\"raw_wide\":{},\"raw_rows\":{},\"partial_unused_fields\":{},\"bytes\":{},\"bytes_a_row\":{:.4},\"raw_bound\":{},\"max_label\":{},\"parking_label\":{},\"max_span_ordinary\":{},\"starts_marked\":{},\"starts_on_raw\":{}}}",
                file,
                blk,
                c.rows,
                c.sigma,
                c.groups,
                c.ordinary,
                c.raw_crossing,
                c.raw_wide,
                c.raw_rows,
                c.partial_unused,
                c.bytes,
                c.bytes as f64 / c.rows.max(1) as f64,
                c.bound,
                c.max_label,
                c.parking_label,
                c.max_span_ordinary,
                c.starts_marked,
                c.starts_on_raw
            )
            .unwrap();
            tot.rows += c.rows;
            tot.groups += c.groups;
            tot.ordinary += c.ordinary;
            tot.raw_crossing += c.raw_crossing;
            tot.raw_wide += c.raw_wide;
            tot.raw_rows += c.raw_rows;
            tot.bytes += c.bytes;
            tot.starts_marked += c.starts_marked;
            tot.starts_on_raw += c.starts_on_raw;
            blk += 1;
            syms = symbols.syms;
        }
        println!(
            "{file}: {blk} blocks, {} rows, groups {} (ordinary {}, RAW crossing {}, RAW wide {}); RAW rows {:.3}%; {:.4} bytes a row; starts marked {} ({} on RAW rows); every check and both round trips exact",
            tot.rows,
            tot.groups,
            tot.ordinary,
            tot.raw_crossing,
            tot.raw_wide,
            100.0 * tot.raw_rows as f64 / tot.rows.max(1) as f64,
            tot.bytes as f64 / tot.rows.max(1) as f64,
            tot.starts_marked,
            tot.starts_on_raw
        );
    }
}
