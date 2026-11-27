//! Research only (`moonshot`): NSPC, non-splitting affine contraction, as an inverse-BWT walk
//! kernel, timed against the shipped kernels on exact BWT columns, on one thread
//! (docs/journal/2026-11-20-nspc-prototype.md). Never on a production path.
//!
//! ```text
//! nspc DIR ROUNDS [BLOCK...]      # DIR holds block-<id>.L and manifest.json
//! ```
//!
//! **NSPC.** The column's maximal runs are cut into capped records of M = 2^b rows. For each
//! record, (init_b, init_off) locates LF of its first row: the maximal run holding it, and the
//! offset in it (one two-pointer merge in codomain order). A record is then contracted, on its
//! own: while its rows all land inside one maximal run (they never split) and fewer than K
//! steps are taken, LF is applied to the whole record at once, an affine step, and that run's
//! byte appended to the record's word. The walk emits a record's word (up to K bytes) a step,
//! then moves to the record and offset its rows land in. A record shorter than tau rows is not
//! contracted (stride 1). Records are 16 bytes: the word, the target, the stride, the offset
//! and the record's own length.
//!
//! **Arms**, each a block's run records (or column) in, its RLE1-coded text out, every output
//! checked byte for byte against a dense-LF walk:
//! - `dense`: an LF table from the column, walked one chain (the reference);
//! - `plain`, `segmented`: the crate's column kernels (`bwt::Plain`, `bwt::Segmented`);
//! - `lazy`, `open`: `LazyCells` and `OpenCells` (production's) on the 127-capped records;
//! - `nspc M K tau`: construction (from the same 127-capped records) and walk, timed apart.
//!
//! Rounds run every arm once, the arms' order rotated round by round; each arm keeps its
//! buffers from round to round, as a worker does from block to block. Medians and the round
//! ratios against `open` are printed as CSV.

use std::time::Instant;

use mibz2::bwt::{InverseBwt, LazyCells, OpenCells, Plain, Segmented};
use mibz2::runs::Runs;
use mibz2::Kernels;

/// A walk record: the word (stream order w0.. from the top byte down), the target record, the
/// stride, the offset added at the target, and this record's own length
#[derive(Clone, Copy, Default)]
#[repr(C, align(16))]
struct Rec {
    word: u64,
    target: u32,
    stride: u8,
    off: u8,
    len: u8,
    _pad: u8,
}

const _: () = assert!(std::mem::size_of::<Rec>() == 16);

#[derive(Default)]
struct Nspc {
    max_start: Vec<u32>,
    max_len: Vec<u32>,
    max_sym: Vec<u8>,
    capped_base: Vec<u32>,
    order: Vec<u32>,
    init_b: Vec<u32>,
    init_off: Vec<u32>,
    table: Vec<Rec>,
    /// the 8-byte layout (M = 64, K <= 4): the word (w0 in the top byte) in bits 0-31, then
    /// meta = target << 14 | (len - 1) << 8 | off << 2 | (stride - 1)
    table8: Vec<u64>,
    out: Vec<u8>,
    /// counts from the last build: dependent contraction steps and their scans
    q_con: u64,
    q_scan: u64,
}

impl Nspc {
    /// The scratch and the table, in bytes, as last built
    fn bytes(&self) -> (usize, usize) {
        let scratch = self.max_start.len() * 4
            + self.max_len.len() * 4
            + self.max_sym.len()
            + self.capped_base.len() * 4
            + self.order.len() * 4
            + self.init_b.len() * 4
            + self.init_off.len() * 4;
        (scratch, self.table.len() * 16 + self.table8.len() * 8)
    }

    /// The walk table from a block's 127-capped run records
    fn build(&mut self, records: &[u16], shift: u32, k: usize, tau: u32, packed: bool) {
        let m = 1u32 << shift;
        let mask = m - 1;
        // maximal runs: adjacent records of one byte coalesced
        self.max_start.clear();
        self.max_len.clear();
        self.max_sym.clear();
        let mut counts = [0u32; 256];
        let mut pos = 0u32;
        for &rec in records {
            let (c, l) = ((rec >> 8) as u8, (rec & 0xff) as u32);
            counts[c as usize] += l;
            if let Some(&last) = self.max_sym.last() {
                if last == c {
                    *self.max_len.last_mut().unwrap() += l;
                    pos += l;
                    continue;
                }
            }
            self.max_start.push(pos);
            self.max_len.push(l);
            self.max_sym.push(c);
            pos += l;
        }
        let r = self.max_len.len();
        // capped records
        self.capped_base.clear();
        let mut cid = 0u32;
        let mut runs_of = [0u32; 256];
        for b in 0..r {
            self.capped_base.push(cid);
            cid += self.max_len[b].div_ceil(m);
            runs_of[self.max_sym[b] as usize] += 1;
        }
        let rm = cid as usize;
        // maximal runs in codomain order: a stable counting sort by byte
        let mut at = [0u32; 256];
        let mut acc = 0;
        for c in 0..256 {
            at[c] = acc;
            acc += runs_of[c];
        }
        self.order.clear();
        self.order.resize(r, 0);
        for b in 0..r {
            let c = self.max_sym[b] as usize;
            self.order[at[c] as usize] = b as u32;
            at[c] += 1;
        }
        // (init_b, init_off): the two-pointer merge, codomains rising
        self.init_b.clear();
        self.init_b.resize(rm, 0);
        self.init_off.clear();
        self.init_off.resize(rm, 0);
        let (ms, ml) = (&self.max_start, &self.max_len);
        let mut cod = 0u32;
        let mut bp = 0usize;
        for &b in &self.order {
            let b = b as usize;
            let base = self.capped_base[b] as usize;
            let mut left = ml[b];
            let mut j = 0;
            while left > 0 {
                let chunk = left.min(m);
                while ms[bp] + ml[bp] <= cod {
                    bp += 1;
                }
                self.init_b[base + j] = bp as u32;
                self.init_off[base + j] = cod - ms[bp];
                cod += chunk;
                left -= chunk;
                j += 1;
            }
        }
        debug_assert_eq!(cod as usize, pos as usize);
        // contraction, record by record
        self.table.clear();
        self.table8.clear();
        if packed {
            assert!(
                shift == 6 && k <= 4 && rm < 1 << 18,
                "the 8-byte layout's bounds"
            );
            self.table8.resize(rm, 0);
        } else {
            self.table.resize(rm, Rec::default());
        }
        let (mut q_con, mut q_scan) = (0u64, 0u64);
        let (ib, io, cb, sy) = (
            &self.init_b,
            &self.init_off,
            &self.capped_base,
            &self.max_sym,
        );
        let mut a = 0usize;
        for b in 0..r {
            let sym = sy[b];
            let mut left = ml[b];
            while left > 0 {
                let la = left.min(m);
                left -= la;
                let mut word = (sym as u64) << 56;
                let mut s = 1usize;
                let (mut cur_b, mut cur_off) = (ib[a] as usize, io[a]);
                if la >= tau {
                    while s < k && cur_off + la <= ml[cur_b] {
                        word |= (sy[cur_b] as u64) << (56 - 8 * s);
                        s += 1;
                        let c = cb[cur_b] as usize + (cur_off >> shift) as usize;
                        let mut nb = ib[c] as usize;
                        let mut noff = io[c] + (cur_off & mask);
                        q_con += 1;
                        while noff >= ml[nb] {
                            noff -= ml[nb];
                            nb += 1;
                            q_scan += 1;
                        }
                        cur_b = nb;
                        cur_off = noff;
                    }
                }
                let target = cb[cur_b] + (cur_off >> shift);
                if packed {
                    let meta =
                        target << 14 | (la - 1) << 8 | (cur_off & mask) << 2 | (s as u32 - 1);
                    self.table8[a] = (meta as u64) << 32 | (word >> 32);
                } else {
                    self.table[a] = Rec {
                        word,
                        target,
                        stride: s as u8,
                        off: (cur_off & mask) as u8,
                        len: la as u8,
                        _pad: 0,
                    };
                }
                a += 1;
            }
        }
        self.q_con = q_con;
        self.q_scan = q_scan;
    }

    /// The block's RLE1-coded text from the table, walked from `p`; returns it as
    /// `self.out[8..8 + n]` (8 bytes of headroom below, for the unconditional 8-byte stores)
    fn walk(&mut self, n: usize, p: u32, shift: u32) -> (usize, usize) {
        // the maximal run, then the capped record, holding p
        let b = self.max_start.partition_point(|&s| s <= p) - 1;
        let d = p - self.max_start[b];
        let mut a = self.capped_base[b] as usize + (d >> shift) as usize;
        let mut off = d & ((1 << shift) - 1);
        self.out.clear();
        self.out.resize(n + 8, 0);
        let t = &self.table[..];
        assert!(!t.is_empty());
        let out = self.out.as_mut_ptr();
        let mut pos = n + 8;
        let (mut steps, mut scans) = (0usize, 0usize);
        // SAFETY: every target is a record index below t.len() and the scan stays inside the
        // target's maximal run or the next records (the table tiles [0, n)); pos > 8 before
        // each 8-byte store at pos - 8, which ends at pos <= n + 8
        unsafe {
            while pos > 8 {
                let rec = *t.get_unchecked(a);
                std::ptr::write_unaligned(out.add(pos - 8) as *mut u64, rec.word.to_le());
                pos -= rec.stride as usize;
                steps += 1;
                let mut na = rec.target as usize;
                let mut noff = rec.off as u32 + off;
                loop {
                    let l = t.get_unchecked(na).len as u32;
                    if noff < l {
                        break;
                    }
                    noff -= l;
                    na += 1;
                    scans += 1;
                }
                a = na;
                off = noff;
            }
        }
        (steps, scans)
    }
}

impl Nspc {
    /// `walk` on the 8-byte layout (M = 64)
    fn walk8(&mut self, n: usize, p: u32) -> (usize, usize) {
        let b = self.max_start.partition_point(|&s| s <= p) - 1;
        let d = p - self.max_start[b];
        let mut a = self.capped_base[b] as usize + (d >> 6) as usize;
        let mut off = d & 63;
        self.out.clear();
        self.out.resize(n + 8, 0);
        let t = &self.table8[..];
        assert!(!t.is_empty());
        let out = self.out.as_mut_ptr();
        let mut pos = n + 8;
        let (mut steps, mut scans) = (0usize, 0usize);
        // SAFETY: as `walk`'s; each 4-byte store at pos - 4 with pos > 8
        unsafe {
            while pos > 8 {
                let rec = *t.get_unchecked(a);
                std::ptr::write_unaligned(out.add(pos - 4) as *mut u32, (rec as u32).to_le());
                let meta = (rec >> 32) as u32;
                pos -= (meta & 3) as usize + 1;
                steps += 1;
                let mut na = (meta >> 14) as usize;
                let mut noff = (meta >> 2 & 63) + off;
                loop {
                    let l = (*t.get_unchecked(na) >> 40 & 63) as u32 + 1;
                    if noff < l {
                        break;
                    }
                    noff -= l;
                    na += 1;
                    scans += 1;
                }
                a = na;
                off = noff;
            }
        }
        (steps, scans)
    }
}

/// The dense-LF reference: LF(i) << 8 | L[i], walked one chain from p, the text written last
/// byte first
#[derive(Default)]
struct Dense {
    tt: Vec<u32>,
}

impl Dense {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        let n = ll.len();
        let mut occ = [0u32; 256];
        let mut acc = 0;
        for c in 0..256 {
            occ[c] = acc;
            acc += counts[c];
        }
        self.tt.clear();
        self.tt.reserve(n);
        for &c in ll {
            self.tt.push(occ[c as usize] << 8 | c as u32);
            occ[c as usize] += 1;
        }
        out.clear();
        out.resize(n, 0);
        let mut x = p as usize;
        for t in (0..n).rev() {
            let v = self.tt[x];
            out[t] = v as u8;
            x = (v >> 8) as usize;
        }
    }
}

struct Block {
    id: String,
    ll: Vec<u8>,
    counts: [u32; 256],
    p: u32,
    runs: Runs,
}

/// The manifest's primary index for `file`, read without a JSON parser (its own format)
fn primary(manifest: &str, file: &str) -> u32 {
    let at = manifest
        .find(&format!("\"file\": \"{file}\""))
        .expect("block in manifest");
    let rest = &manifest[at..];
    let k = rest.find("\"primary_index\":").unwrap() + "\"primary_index\":".len();
    rest[k..]
        .trim_start()
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn quantile(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let i = ((v.len() - 1) as f64 * q).round() as usize;
    v[i]
}

#[derive(Clone, Copy, PartialEq)]
enum Arm {
    Dense,
    Plain,
    Segmented,
    Lazy,
    Open,
    Nspc {
        shift: u32,
        k: usize,
        tau: u32,
    },
    /// M = 64, K = 4, the 8-byte record
    Nspc8 {
        tau: u32,
    },
}

impl Arm {
    fn name(&self) -> String {
        match self {
            Arm::Dense => "dense".into(),
            Arm::Plain => "plain".into(),
            Arm::Segmented => "segmented".into(),
            Arm::Lazy => "lazy".into(),
            Arm::Open => "open".into(),
            Arm::Nspc { shift, k, tau } => format!("nspc_M{}_K{}_t{}", 1 << shift, k, tau),
            Arm::Nspc8 { tau } => format!("nspc8B_M64_K4_t{tau}"),
        }
    }
}

#[derive(Default)]
struct Kernels5 {
    dense: Dense,
    plain: Plain,
    seg: Segmented,
    lazy: LazyCells,
    open: OpenCells,
    nspc: Nspc,
    out: Vec<u8>,
}

impl Kernels5 {
    /// One run of `arm` on `b`: (construction s, walk s) (a fused kernel's time all in the
    /// first) and the output, checked against `truth`
    fn run(&mut self, arm: Arm, b: &Block, k: Kernels, truth: Option<&[u8]>) -> (f64, f64) {
        let n = b.ll.len();
        let t0 = Instant::now();
        let (c, w, ok) = match arm {
            Arm::Dense => {
                self.dense.invert(&b.ll, &b.counts, b.p, &mut self.out);
                (
                    t0.elapsed().as_secs_f64(),
                    0.0,
                    truth.map_or(true, |t| t == &self.out[..]),
                )
            }
            Arm::Plain => {
                self.plain.invert(&b.ll, &b.counts, b.p, &mut self.out);
                (
                    t0.elapsed().as_secs_f64(),
                    0.0,
                    truth.map_or(true, |t| t == &self.out[..]),
                )
            }
            Arm::Segmented => {
                self.seg.invert(&b.ll, &b.counts, b.p, &mut self.out);
                (
                    t0.elapsed().as_secs_f64(),
                    0.0,
                    truth.map_or(true, |t| t == &self.out[..]),
                )
            }
            Arm::Lazy => {
                assert!(self.lazy.invert_runs(&b.runs, n, b.p, &mut self.out, k));
                (
                    t0.elapsed().as_secs_f64(),
                    0.0,
                    truth.map_or(true, |t| t == &self.out[..]),
                )
            }
            Arm::Open => {
                assert!(self.open.invert_runs(&b.runs, n, b.p, &mut self.out, k));
                (
                    t0.elapsed().as_secs_f64(),
                    0.0,
                    truth.map_or(true, |t| t == &self.out[..]),
                )
            }
            Arm::Nspc { shift, k: kk, tau } => {
                self.nspc.build(b.runs.records(), shift, kk, tau, false);
                let c = t0.elapsed().as_secs_f64();
                let t1 = Instant::now();
                std::hint::black_box(self.nspc.walk(n, b.p, shift));
                let w = t1.elapsed().as_secs_f64();
                (c, w, truth.map_or(true, |t| t == &self.nspc.out[8..8 + n]))
            }
            Arm::Nspc8 { tau } => {
                self.nspc.build(b.runs.records(), 6, 4, tau, true);
                let c = t0.elapsed().as_secs_f64();
                let t1 = Instant::now();
                std::hint::black_box(self.nspc.walk8(n, b.p));
                let w = t1.elapsed().as_secs_f64();
                (c, w, truth.map_or(true, |t| t == &self.nspc.out[8..8 + n]))
            }
        };
        assert!(
            ok,
            "{}: {} output differs from the dense-LF walk",
            b.id,
            arm.name()
        );
        (c, w)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(&args[1]);
    let rounds: usize = args.get(2).map_or(31, |s| s.parse().unwrap());
    let ids: Vec<String> = if args.len() > 3 {
        args[3..].to_vec()
    } else {
        ["0", "2", "85", "143", "207", "210"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    let manifest = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    let k = Kernels::detect();

    let mut arms = vec![Arm::Dense, Arm::Plain, Arm::Segmented, Arm::Lazy, Arm::Open];
    for shift in [5u32, 6, 7] {
        for kk in [1usize, 4, 8] {
            for tau in [1u32, 2, 4] {
                if kk == 1 && tau > 1 {
                    continue; // tau is moot without contraction
                }
                arms.push(Arm::Nspc { shift, k: kk, tau });
            }
        }
    }
    for tau in [1u32, 2, 4] {
        arms.push(Arm::Nspc8 { tau });
    }
    // NSPC_ONLY=<arm>: that arm alone (the cachegrind runs: two round counts, differenced)
    if let Ok(only) = std::env::var("NSPC_ONLY") {
        arms.retain(|a| a.name() == only);
        assert_eq!(arms.len(), 1, "NSPC_ONLY names one arm");
    }

    println!("block,arm,construct_ms,walk_ms,total_ms,total_p10_ms,total_p90_ms,ratio_open_med,ratio_open_p10,ratio_open_p90,table_bytes,scratch_bytes,steps,scans,q_con,q_scan,rows");
    for id in &ids {
        let file = format!("block-{id}.L");
        let ll = std::fs::read(dir.join(&file)).unwrap();
        let mut counts = [0u32; 256];
        for &c in &ll {
            counts[c as usize] += 1;
        }
        let b = Block {
            id: id.clone(),
            runs: Runs::of_column(&ll),
            p: primary(&manifest, &file),
            counts,
            ll,
        };
        let n = b.ll.len();
        let mut ks = Kernels5::default();
        // the reference, then every arm checked against it once (and NSPC's counts kept)
        let mut truth = Vec::new();
        ks.dense.invert(&b.ll, &b.counts, b.p, &mut truth);
        let mut meta = Vec::new();
        for &arm in &arms {
            ks.run(arm, &b, k, Some(&truth));
            if matches!(arm, Arm::Nspc { .. } | Arm::Nspc8 { .. }) {
                let (steps, scans) = match arm {
                    Arm::Nspc { shift, .. } => ks.nspc.walk(n, b.p, shift),
                    _ => ks.nspc.walk8(n, b.p),
                };
                let (scratch, table) = ks.nspc.bytes();
                meta.push((table, scratch, steps, scans, ks.nspc.q_con, ks.nspc.q_scan));
            } else {
                meta.push((0, 0, 0, 0, 0, 0));
            }
        }
        eprintln!(
            "block {id}: n={n} p={} records={}: all {} arms byte-exact",
            b.p,
            b.runs.records().len(),
            arms.len()
        );
        // timed rounds, the order rotated
        let na = arms.len();
        let mut con = vec![Vec::new(); na];
        let mut wal = vec![Vec::new(); na];
        let open_i = arms.iter().position(|a| *a == Arm::Open).unwrap_or(0);
        for round in 0..rounds + 2 {
            let mut this = vec![(0.0, 0.0); na];
            for j in 0..na {
                let i = (j + round) % na;
                // the check stays on every round: its cost is outside the timed span
                this[i] = ks.run(arms[i], &b, k, if round < 2 { Some(&truth) } else { None });
            }
            if round >= 2 {
                for i in 0..na {
                    con[i].push(this[i].0);
                    wal[i].push(this[i].1);
                }
            }
        }
        let open_tot: Vec<f64> = con[open_i]
            .iter()
            .zip(&wal[open_i])
            .map(|(a, b)| a + b)
            .collect();
        for i in 0..na {
            let mut tot: Vec<f64> = con[i].iter().zip(&wal[i]).map(|(a, b)| a + b).collect();
            let mut ratio: Vec<f64> = tot.iter().zip(&open_tot).map(|(a, b)| a / b).collect();
            let (cm, wm) = (median(&mut con[i].clone()), median(&mut wal[i].clone()));
            let m = meta[i];
            println!(
                "{id},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{},{},{},{n}",
                arms[i].name(),
                cm * 1e3,
                wm * 1e3,
                median(&mut tot.clone()) * 1e3,
                quantile(&mut tot, 0.1) * 1e3,
                quantile(&mut tot, 0.9) * 1e3,
                median(&mut ratio.clone()),
                quantile(&mut ratio, 0.1),
                quantile(&mut ratio, 0.9),
                m.0,
                m.1,
                m.2,
                m.3,
                m.4,
                m.5
            );
        }
    }
}
