#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/mod.rs"]
mod cand_r5_nobi;

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

// ============================================================================
// 1. Block Data Structures & Corpus Loaders
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    route_class: u8, // 0=normal, 1=dense, 2=tiny, 3=column
    records: Vec<u16>,
    expected: Option<Vec<u8>>, // populated for 6 admitted + stress + non-wikidata
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in recs {
        let b = (rec >> 8) as u8;
        let len = (rec & 255) as usize;
        l.resize(l.len() + len, b);
    }
    assert_eq!(l.len(), n);
    l
}

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos + n - 1) % n]);
    }
    (l, p)
}

fn plain_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    let mut rank_in_l = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        rank_in_l[i] = counts[b as usize];
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for k in (0..n).rev() {
        let b = l[x];
        out[k] = b;
        x = (starts[b as usize] + rank_in_l[x]) as usize;
    }
    out
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let expected = plain_invert(&l, p);
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                route_class: 0,
                records,
                expected: Some(expected),
            }
        })
        .collect()
}

fn load_bin_corpus(path: &str, prefix: &str, with_expected: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0", "bad magic in {path}");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::with_capacity(num);
    for _ in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
        let route_class = raw[off + 16];
        let _takes_runs = raw[off + 17];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        let expected = if with_expected {
            let l = expand_records(&records, n);
            Some(plain_invert(&l, p))
        } else {
            None
        };
        blocks.push(Block {
            name: format!("{prefix}-{idx}"),
            n,
            p,
            route_class,
            records,
            expected,
        });
    }
    assert_eq!(off, raw.len());
    blocks
}

fn make_dense_300k() -> Block {
    let n = 300_000usize;
    let mut text = Vec::with_capacity(n);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        text.push(if i % 9 == 0 {
            b' '
        } else {
            b'a' + ((s >> 33) % 26) as u8
        });
    }
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| {
        let (ia, ib) = (a as usize, b as usize);
        for k in 0..64 {
            let ca = text[(ia + k) % n];
            let cb = text[(ib + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        ia.cmp(&ib)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos as usize + n - 1) % n]);
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    Block {
        name: "dense-300k".into(),
        n,
        p,
        route_class: 3,
        records: records_of(&l),
        expected: Some(expected),
    }
}

fn make_spiral_200k() -> Block {
    let n = 200_000usize;
    let mut l = vec![b'a'; n];
    l[0] = b'c';
    l[n - 1] = b'b';
    let p = 0u32;
    let expected = plain_invert(&l, p);
    Block {
        name: "spiral-200k".into(),
        n,
        p,
        route_class: 2,
        records: records_of(&l),
        expected: Some(expected),
    }
}

fn make_periodic(name: &str, pat: &[u8], reps: usize) -> Block {
    let n = pat.len() * reps;
    let mut text = Vec::with_capacity(n);
    for _ in 0..reps {
        text.extend_from_slice(pat);
    }
    let mut rots: Vec<usize> = (0..pat.len()).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..pat.len() {
            let ca = pat[(a + k) % pat.len()];
            let cb = pat[(b + k) % pat.len()];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (grp, &r) in rots.iter().enumerate() {
        let prev_b = pat[(r + pat.len() - 1) % pat.len()];
        if r == 0 {
            p = (grp * reps) as u32;
        }
        for _ in 0..reps {
            l.push(prev_b);
        }
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    Block {
        name: name.into(),
        n,
        p,
        route_class: 2,
        records: records_of(&l),
        expected: Some(expected),
    }
}

// ============================================================================
// 2. Exact Zoo Kernels: Plain-Walk & Interleaved-Walk (Seg8-FL) + Specialists
// ============================================================================

#[derive(Default)]
struct PlainRunWalk {
    fl: Vec<u32>,
}

impl PlainRunWalk {
    #[inline(never)]
    fn build_fl(&mut self, records: &[u16], n: usize) -> bool {
        let mut counts = [0u32; 256];
        for &rec in records {
            counts[(rec >> 8) as usize] += u32::from(rec & 255);
        }
        let mut starts = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            starts[c] = tot;
            tot += counts[c];
        }
        if tot as usize != n {
            return false;
        }
        if self.fl.len() < n {
            self.fl.resize(n, 0);
        }
        let fl = &mut self.fl[..n];
        let mut pos = 0usize;
        for &rec in records {
            let b = (rec >> 8) as usize;
            let len = (rec & 255) as usize;
            let s0 = starts[b] as usize;
            starts[b] += len as u32;
            let mut i = pos as u32;
            let bu = b as u32;
            for slot in &mut fl[s0..s0 + len] {
                *slot = (i << 8) | bu;
                i += 1;
            }
            pos += len;
        }
        true
    }

    #[inline(never)]
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n {
            return false;
        }
        // Fast O(1) constant-symbol check when single record tiles n exactly
        if records.len() == 1 {
            if usize::from(records[0] & 255) != n {
                return false;
            }
            let b = (records[0] >> 8) as u8;
            out.clear();
            out.resize(n, b);
            return true;
        }
        if !self.build_fl(records, n) {
            return false;
        }
        out.resize(n, 0);
        let fl = &self.fl[..n];
        let mut x = p as usize;
        for slot in &mut out[..n] {
            let v = fl[x];
            *slot = v as u8;
            x = (v >> 8) as usize;
        }
        true
    }

    #[inline(never)]
    fn invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        assert!(self.try_invert(records, n, p, out), "PlainRunWalk invalid tiling/bounds");
    }
}

/// Genuine 8-way interleaved forward-FL walk (pure safe Rust, O(K) start-bitmap clear,
/// exact forward-FL fallback matching libbzip2 / mibz2::bwt::InverseBwt / cand_r5_head).
#[derive(Default)]
struct InterleavedWalk {
    plain: PlainRunWalk,
    start_bits: Vec<u64>,
    segs: [Vec<u8>; 8],
}

impl InterleavedWalk {
    #[inline(never)]
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n {
            return false;
        }
        if n < 4096 || records.len() == 1 {
            return self.plain.try_invert(records, n, p, out);
        }
        if !self.plain.build_fl(records, n) {
            return false;
        }
        let fl = &self.plain.fl[..n];

        let words = (n + 63) / 64;
        if self.start_bits.len() < words {
            self.start_bits.resize(words, 0);
        }

        // Choose K = 8 distinct starting rows, with starts[0] = p.
        let mut starts = [0usize; 8];
        let stride = (n / 8).max(1);
        for c in 0..8 {
            let mut s = (p as usize + c * stride) % n;
            while (self.start_bits[s >> 6] >> (s & 63)) & 1 != 0 {
                s = (s + 1) % n;
            }
            starts[c] = s;
            self.start_bits[s >> 6] |= 1u64 << (s & 63);
        }

        for seg in &mut self.segs {
            seg.clear();
        }
        let mut cur = starts;
        let mut end_row = [usize::MAX; 8];
        let mut active = [0usize, 1, 2, 3, 4, 5, 6, 7];
        let mut n_active = 8usize;
        let mut total_steps = 0usize;

        while n_active > 0 && total_steps <= n {
            let mut i = 0;
            while i < n_active {
                let c = active[i];
                let v = fl[cur[c]];
                self.segs[c].push(v as u8);
                let nx = (v >> 8) as usize;
                cur[c] = nx;
                total_steps += 1;
                if (self.start_bits[nx >> 6] >> (nx & 63)) & 1 != 0 {
                    end_row[c] = nx;
                    n_active -= 1;
                    active[i] = active[n_active];
                } else {
                    i += 1;
                }
            }
        }

        // Clear only the 8 set bits (O(1) vs O(n) memset)
        for &s in &starts {
            self.start_bits[s >> 6] &= !(1u64 << (s & 63));
        }

        // Stitch cycle forward from c = 0 (starts[0] == p)
        let mut order = [0usize; 8];
        let mut n_ord = 0usize;
        let mut stitched_len = 0usize;
        let mut cur_c = 0usize;
        let mut closed = false;
        while n_ord < 8 {
            if end_row[cur_c] == usize::MAX {
                break;
            }
            order[n_ord] = cur_c;
            n_ord += 1;
            stitched_len += self.segs[cur_c].len();
            let er = end_row[cur_c];
            if er == starts[0] {
                closed = true;
                break;
            }
            if let Some(next_c) = starts.iter().position(|&s| s == er) {
                cur_c = next_c;
            } else {
                break;
            }
        }

        if closed && stitched_len > 0 && stitched_len <= n {
            out.clear();
            out.reserve(n);
            for &c in &order[..n_ord] {
                out.extend_from_slice(&self.segs[c]);
            }
            while out.len() < n {
                let take = out.len().min(n - out.len());
                out.extend_from_within(..take);
            }
            debug_assert_eq!(out.len(), n);
            return true;
        }

        // Exact forward-FL fallback for unclosed / malformed multi-cycle columns
        out.resize(n, 0);
        let mut x = p as usize;
        for slot in &mut out[..n] {
            let v = fl[x];
            *slot = v as u8;
            x = (v >> 8) as usize;
        }
        true
    }

    #[inline(never)]
    fn invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        assert!(self.try_invert(records, n, p, out), "InterleavedWalk invalid tiling/bounds");
    }
}

/// Specialist 1 (`Spec-TinyDense`, MSC 05 + 68):
/// - O(1) recognizer before S1:
///   (a) Single-symbol / tiny (`n < 32_768`): `PlainRunWalk`
///   (b) Ultra-sparse periodic (`recs <= 64 && n >= 4096`): short-cycle exact probe (<= 256 steps)
///       using run-rank binary/linear lookup, tiling exact periods `d | n`, else fallback to `R5-Head`!
///   (c) Over-dense / column-route (`recs * 25 >= n * 7`, i.e. `r/n >= 0.28`): `InterleavedWalk`
///   (d) Dense band (`recs * 20 >= n * 3`): `R5-Head` with `Cap::dense(n)`, fallback `InterleavedWalk`
///   (e) Normal band: `R5-Head` verbatim.
#[derive(Default)]
struct SpecTinyDense {
    open: cand_r5_head::Kernel,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl SpecTinyDense {
    /// O(r_small) exact LF step without allocating O(n) table, triggered ONLY on the ultra-sparse
    /// regime (`records.len() * 64 < n && n >= 4096`) and aborting immediately if maximal runs > 64.
    fn try_short_periodic(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if records.len() * 64 >= n || n < 4096 {
            return false;
        }
        // Merge maximal runs (at most 64)
        let mut sym = [0u8; 64];
        let mut r_start = [0u32; 64];
        let mut r_len = [0u32; 64];
        let mut r_rank = [0u32; 64];
        let mut counts = [0u32; 256];
        let mut nr = 0usize;
        let mut pos = 0u32;
        for &rec in records {
            let b = (rec >> 8) as u8;
            let l = u32::from(rec & 255);
            if nr > 0 && sym[nr - 1] == b {
                r_len[nr - 1] += l;
            } else {
                if nr >= 64 {
                    return false;
                }
                sym[nr] = b;
                r_start[nr] = pos;
                r_len[nr] = l;
                r_rank[nr] = counts[b as usize];
                nr += 1;
            }
            counts[b as usize] += l;
            pos += l;
        }
        let mut c_start = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            c_start[c] = tot;
            tot += counts[c];
        }
        if tot as usize != n {
            return false;
        }
        // Probe up to 64 steps to detect short periodic cycle closing at `p`
        let mut cyc = [0u8; 64];
        let mut x = p;
        for step in 0..64usize {
            let mut lo = 0usize;
            let mut hi = nr;
            while lo + 1 < hi {
                let mid = (lo + hi) / 2;
                if r_start[mid] <= x {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let b = sym[lo];
            cyc[step] = b;
            x = c_start[b as usize] + r_rank[lo] + (x - r_start[lo]);
            if x == p {
                let d = step + 1;
                if n % d == 0 {
                    out.clear();
                    out.reserve(n);
                    for k in (0..d).rev() {
                        out.push(cyc[k]);
                    }
                    while out.len() < n {
                        let take = out.len().min(n - out.len());
                        out.extend_from_within(..take);
                    }
                    return true;
                }
                return false;
            }
        }
        false
    }

    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .run_entry(records, n, p, cand_r5_head::Cap::NONE, out)
                .expect("fallback");
            return 0;
        }
        if n < 32_768 {
            self.seg.plain.invert(records, n, p, out);
            return 1; // tiny -> PlainRunWalk
        }
        if records.len() * 64 < n && Self::try_short_periodic(records, n, p, out) {
            return 2; // periodic specialist
        }
        if records.len() * 25 >= n * 7 {
            self.seg.invert(records, n, p, out);
            return 3; // high-density / column -> InterleavedWalk
        }
        if records.len() * 20 >= n * 3 {
            if self
                .open
                .run_entry(records, n, p, cand_r5_head::Cap::dense(n), out)
                .is_some()
            {
                return 4; // dense-band Open
            }
            self.seg.invert(records, n, p, out);
            return 3; // Cap::dense refused -> InterleavedWalk
        }
        self.open
            .run_entry(records, n, p, cand_r5_head::Cap::NONE, out)
            .expect("open");
        0 // normal Open R5-Head
    }
}

/// Specialist 2 (`Spec-Adaptive`, MSC 68 + 52 + 90):
/// Same exact tiny/periodic/dense/column routing as `SpecTinyDense`, PLUS
/// a physics-based L2-residency selector between `μ=1` (`NobiW16`) and `μ=2` (`Head`):
/// when `records.len() < nobi_max_recs` (default `60_000`, predicting `m ≲ 95k` so `edges[..m]`
/// fits comfortably in L2), `μ=1` avoids double S4 XOR-scatter cost; above that threshold
/// `μ=2` (`R5-Head`) hides L2/L3 walk latency in S5. Zero-cost O(1) recognizer (`1` comparison),
/// unconditional exact fallback to `R5-Head`.
struct SpecAdaptive {
    open_head: cand_r5_head::Kernel,
    open_nobi: cand_r5_nobi::Kernel,
    seg: InterleavedWalk,
    nobi_max_recs: usize,
    limit_low: u32,
    limit_hi: u32,
    force_fallback: bool,
}

impl SpecAdaptive {
    fn new(nobi_max_recs: usize, limit_low: u32, limit_hi: u32) -> Self {
        Self {
            open_head: cand_r5_head::Kernel::default(),
            open_nobi: cand_r5_nobi::Kernel::default(),
            seg: InterleavedWalk::default(),
            nobi_max_recs,
            limit_low,
            limit_hi,
            force_fallback: false,
        }
    }

    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open_head
                .run_entry(records, n, p, cand_r5_head::Cap::NONE, out)
                .expect("fallback");
            return 0;
        }
        if n < 32_768 {
            self.seg.plain.invert(records, n, p, out);
            return 1;
        }
        if records.len() * 64 < n && SpecTinyDense::try_short_periodic(records, n, p, out) {
            return 2;
        }
        if records.len() * 25 >= n * 7 {
            self.seg.invert(records, n, p, out);
            return 3;
        }
        if records.len() * 20 >= n * 3 {
            if self
                .open_head
                .try_run::<true, true>(records, n, p, self.limit_hi, cand_r5_head::Cap::dense(n), out)
                .is_some()
            {
                return 4;
            }
            self.seg.invert(records, n, p, out);
            return 3;
        }
        if records.len() < self.nobi_max_recs {
            self.open_nobi
                .try_run::<true, true>(records, n, p, self.limit_low, cand_r5_nobi::Cap::NONE, out)
                .expect("nobi");
            return 5;
        }
        self.open_head
            .try_run::<true, true>(records, n, p, self.limit_hi, cand_r5_head::Cap::NONE, out)
            .expect("head");
        0
    }
}

fn count_maximal_runs(records: &[u16]) -> usize {
    let mut r = 0usize;
    let mut prev: u16 = 0xFFFF;
    for &rec in records {
        let b = rec >> 8;
        if b != prev {
            r += 1;
            prev = b;
        }
    }
    r
}

// ============================================================================
// 3. Exact 7-Region Classifier (matching working-backwards/costmap.py verbatim)
// ============================================================================

const REGION_NAMES: [&str; 7] = [
    "guid",
    "sha1",
    "entity-id",
    "number",
    "json-key",
    "text",
    "structure",
];

#[inline]
fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

#[inline]
fn is_lo_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

#[inline]
fn is_alnum(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}

fn classify_text_regions(t: &[u8]) -> (Vec<u8>, usize, usize) {
    let n = t.len();
    let mut cls = vec![6u8; n]; // default: structure
    // Pass 0: text (alpha or >= 128)
    for (i, &b) in t.iter().enumerate() {
        let lo = b | 32;
        if (b'a'..=b'z').contains(&lo) || b >= 128 {
            cls[i] = 5;
        }
    }
    // Apply patterns in reverse priority order (matching costmap.py reversed(pat)):
    // 4: json-key: "[A-Za-z_-]+"(?=:)
    let mut i = 0;
    while i < n {
        if t[i] == b'"' {
            let mut j = i + 1;
            while j < n && (t[j].is_ascii_alphabetic() || t[j] == b'_' || t[j] == b'-') {
                j += 1;
            }
            if j > i + 1 && j + 1 < n && t[j] == b'"' && t[j + 1] == b':' {
                for slot in &mut cls[i..=j] {
                    *slot = 4;
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    // 3: number: [+-]?[0-9][0-9.:TZ+-]*
    i = 0;
    while i < n {
        let start = i;
        let has_sign = (t[i] == b'+' || t[i] == b'-') && i + 1 < n && t[i + 1].is_ascii_digit();
        if has_sign || t[i].is_ascii_digit() {
            let mut j = if has_sign { i + 2 } else { i + 1 };
            while j < n
                && (t[j].is_ascii_digit()
                    || matches!(t[j], b'.' | b':' | b'T' | b'Z' | b'+' | b'-'))
            {
                j += 1;
            }
            for slot in &mut cls[start..j] {
                *slot = 3;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    // 2: entity-id: (?<![A-Za-z0-9])[QPL][0-9]+(?:-[SF][0-9]+)?
    i = 0;
    while i < n {
        if matches!(t[i], b'Q' | b'P' | b'L')
            && (i == 0 || !is_alnum(t[i - 1]))
            && i + 1 < n
            && t[i + 1].is_ascii_digit()
        {
            let mut j = i + 2;
            while j < n && t[j].is_ascii_digit() {
                j += 1;
            }
            if j + 2 < n
                && t[j] == b'-'
                && matches!(t[j + 1], b'S' | b'F')
                && t[j + 2].is_ascii_digit()
            {
                let mut k = j + 3;
                while k < n && t[k].is_ascii_digit() {
                    k += 1;
                }
                j = k;
            }
            for slot in &mut cls[i..j] {
                *slot = 2;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    // 1: sha1: (?<![0-9a-f])[0-9a-f]{40}(?![0-9a-f])
    let mut sha1_total = 0usize;
    let mut sha1_set: HashSet<[u8; 40]> = HashSet::new();
    i = 0;
    while i + 40 <= n {
        if is_lo_hex(t[i]) && (i == 0 || !is_lo_hex(t[i - 1])) {
            let mut j = i;
            while j < n && is_lo_hex(t[j]) {
                j += 1;
            }
            if j - i == 40 {
                for slot in &mut cls[i..j] {
                    *slot = 1;
                }
                sha1_total += 1;
                let mut arr = [0u8; 40];
                arr.copy_from_slice(&t[i..j]);
                sha1_set.insert(arr);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    // 0: guid: [0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}
    i = 0;
    while i + 36 <= n {
        if t[i + 8] == b'-'
            && t[i + 13] == b'-'
            && t[i + 18] == b'-'
            && t[i + 23] == b'-'
            && t[i..i + 8].iter().all(|&b| is_hex(b))
            && t[i + 9..i + 13].iter().all(|&b| is_hex(b))
            && t[i + 14..i + 18].iter().all(|&b| is_hex(b))
            && t[i + 19..i + 23].iter().all(|&b| is_hex(b))
            && t[i + 24..i + 36].iter().all(|&b| is_hex(b))
        {
            for slot in &mut cls[i..i + 36] {
                *slot = 0;
            }
            i += 36;
        } else {
            i += 1;
        }
    }
    (cls, sha1_total, sha1_set.len())
}

fn run_analogue_d_region_analysis(six: &[Block]) {
    println!("\n=== ANALOGUE D FAILURE & PER-REGION HEADROOM ANALYSIS (6 ADMITTED BLOCKS) ===");
    let mut k_head = cand_r5_head::Kernel::default();
    let mut out = Vec::with_capacity(900_064);

    for b in six {
        let l = expand_records(&b.records, b.n);
        let n = b.n;
        // Build LF and map each row -> text pos
        let mut counts = [0u32; 256];
        let mut rank_in_l = vec![0u32; n];
        for (i, &ch) in l.iter().enumerate() {
            rank_in_l[i] = counts[ch as usize];
            counts[ch as usize] += 1;
        }
        let mut starts = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            starts[c] = tot;
            tot += counts[c];
        }
        let mut pos_of_row = vec![0usize; n];
        let mut text = vec![0u8; n];
        let mut x = b.p as usize;
        for k in (0..n).rev() {
            let ch = l[x];
            text[k] = ch;
            pos_of_row[x] = k;
            x = (starts[ch as usize] + rank_in_l[x]) as usize;
        }
        let (cls, sha1_tot, sha1_uniq) = classify_text_regions(&text);

        // Compute exact maximal run width per row
        let mut rw = vec![0u32; n];
        let mut i = 0;
        let mut max_runs = 0usize;
        while i < n {
            let ch = l[i];
            let mut j = i + 1;
            while j < n && l[j] == ch {
                j += 1;
            }
            let w = (j - i) as u32;
            for slot in &mut rw[i..j] {
                *slot = w;
            }
            max_runs += 1;
            i = j;
        }

        // Measure R5-Head Pre/Emit medians (30 rounds) to attribute S5 time per region
        let mut pre_v = Vec::with_capacity(30);
        let mut emit_v = Vec::with_capacity(30);
        for _ in 0..30 {
            let (pc, pe) = k_head.run_stages(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out);
            pre_v.push(pc);
            emit_v.push(pe);
        }
        let pre_med = pct(&mut pre_v, 1, 2);
        let emit_med = pct(&mut emit_v, 1, 2);
        let st16 = k_head
            .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out)
            .unwrap();

        // Exact retained set at L=16 (w <= 16 or row == p)
        let mut all_cnt = [0usize; 7];
        let mut ret16_cnt = [0usize; 7];
        let mut w1_cnt = [0usize; 7];
        let mut w2_16_cnt = [0usize; 7];
        let mut ret8_cnt = [0usize; 7];
        let mut ret4_cnt = [0usize; 7];
        for row in 0..n {
            let c = cls[pos_of_row[row]] as usize;
            let w = rw[row];
            let is_p = row == b.p as usize;
            all_cnt[c] += 1;
            if w <= 16 || is_p {
                ret16_cnt[c] += 1;
            }
            if w == 1 {
                w1_cnt[c] += 1;
            } else if w <= 16 {
                w2_16_cnt[c] += 1;
            }
            if w <= 8 || is_p {
                ret8_cnt[c] += 1;
            }
            if w <= 4 || is_p {
                ret4_cnt[c] += 1;
            }
        }
        let m16: usize = ret16_cnt.iter().sum();
        let w1_tot: usize = w1_cnt.iter().sum();
        let w2_16_tot: usize = w2_16_cnt.iter().sum();
        assert_eq!(m16, st16.retained);
        let ns_per_ret = (emit_med * 1000.0) / (m16 as f64);

        println!(
            "\nBLOCK {} | n={} r={} (r/n={:.4}) | L=16: m={} ({:.2}%), thick_strips={} bands={} | Pre={:.1}us Emit={:.1}us ({:.2} ns/ret)",
            b.name, n, max_runs, max_runs as f64 / n as f64,
            m16, 100.0 * m16 as f64 / n as f64,
            st16.thick, st16.bands, pre_med, emit_med, ns_per_ret
        );
        let copy_pct = if sha1_tot > 0 {
            100.0 * (sha1_tot - sha1_uniq) as f64 / sha1_tot as f64
        } else {
            0.0
        };
        println!(
            "  SHA1 census: total={} unique={} copies={} ({:.1}%) | sha1_ret: L16={} -> L8={} -> L4={}",
            sha1_tot, sha1_uniq, sha1_tot.saturating_sub(sha1_uniq), copy_pct,
            ret16_cnt[1], ret8_cnt[1], ret4_cnt[1]
        );
        println!(
            "  {:10} | {:>7} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8}",
            "region", "all%", "ret16%", "w=1%", "w2..16%", "S5_us", "ret4->16"
        );
        for c in 0..7 {
            let s5_c = emit_med * (ret16_cnt[c] as f64 / m16 as f64);
            println!(
                "  {:10} | {:6.2}% | {:7.2}% | {:7.2}% | {:7.2}% | {:7.1} | {:>6}->{:<6}",
                REGION_NAMES[c],
                100.0 * all_cnt[c] as f64 / n as f64,
                100.0 * ret16_cnt[c] as f64 / m16 as f64,
                100.0 * w1_cnt[c] as f64 / w1_tot.max(1) as f64,
                100.0 * w2_16_cnt[c] as f64 / w2_16_tot.max(1) as f64,
                s5_c,
                ret4_cnt[c],
                ret16_cnt[c]
            );
        }
    }
}

// ============================================================================
// 4. Utilities & Kernel Dispatch Enumeration
// ============================================================================

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

const LIMITS: [u32; 8] = [4, 8, 12, 16, 24, 32, 48, 64];
// Zoo layout (18 kernels):
//  0..8:  Open<L, Head>  for L in [4, 8, 12, 16, 24, 32, 48, 64]  (index 3 = R5-Head L=16!)
//  8..16: Open<L, Nobi>  for L in [4, 8, 12, 16, 24, 32, 48, 64]  (index 11 = R5-NobiW16 L=16!)
//  16:    Plain-Walk
//  17:    Interleaved-Walk (Seg8-LF)
const NUM_ZOO: usize = 18;
const IDX_R5_HEAD: usize = 3;
const IDX_R5_NOBI: usize = 11;
const IDX_PLAIN: usize = 16;
const IDX_SEG8: usize = 17;

fn zoo_name(k: usize) -> String {
    if k < 8 {
        if k == IDX_R5_HEAD {
            "R5-Head(L16)".into()
        } else {
            format!("OpenH(L{})", LIMITS[k])
        }
    } else if k < 16 {
        if k == IDX_R5_NOBI {
            "R5-Nobi(L16)".into()
        } else {
            format!("OpenN(L{})", LIMITS[k - 8])
        }
    } else if k == IDX_PLAIN {
        "Plain-Walk".into()
    } else {
        "Seg8-LF".into()
    }
}

struct ZooState {
    head: cand_r5_head::Kernel,
    nobi: cand_r5_nobi::Kernel,
    plain: PlainRunWalk,
    seg8: InterleavedWalk,
}

impl Default for ZooState {
    fn default() -> Self {
        Self {
            head: cand_r5_head::Kernel::default(),
            nobi: cand_r5_nobi::Kernel::default(),
            plain: PlainRunWalk::default(),
            seg8: InterleavedWalk::default(),
        }
    }
}

impl ZooState {
    #[inline(never)]
    fn run(&mut self, k: usize, b: &Block, out: &mut Vec<u8>) {
        if k < 8 {
            self.head
                .try_run::<true, true>(&b.records, b.n, b.p, LIMITS[k], cand_r5_head::Cap::NONE, out)
                .expect("open head");
        } else if k < 16 {
            self.nobi
                .try_run::<true, true>(&b.records, b.n, b.p, LIMITS[k - 8], cand_r5_nobi::Cap::NONE, out)
                .expect("open nobi");
        } else if k == IDX_PLAIN {
            self.plain.invert(&b.records, b.n, b.p, out);
        } else {
            self.seg8.invert(&b.records, b.n, b.p, out);
        }
    }
}

// ============================================================================
// 5. Exhaustive Exactness + Forced Fallback Audit
// ============================================================================

fn run_exactness_audit(six: &[Block], wiki431: &[Block], non_wiki: &[Block]) {
    let mut zoo = ZooState::default();
    let mut spec1 = SpecTinyDense::default();
    let mut spec2 = SpecAdaptive::new(60_000, 24, 16);
    let mut out_ref = Vec::with_capacity(900_064);
    let mut out_k = Vec::with_capacity(900_064);

    // 5a. Small valid BWTs: ALL binary n=1..=12 (8,190) + 2,000 random (n=1..=512, diverse alphabets)
    let mut valid_cnt = 0usize;
    let mut text = Vec::with_capacity(512);
    for n in 1..=12usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n {
                text.push(b'a' + ((mask >> bit) & 1) as u8);
            }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            let blk = Block {
                name: String::new(),
                n,
                p,
                route_class: 2,
                records: recs,
                expected: None,
            };
            // Verify R5-Head, R5-Nobi, OpenH(L4), OpenH(L64), Plain, Seg8, Spec1, Spec2
            for &k in &[0, IDX_R5_HEAD, 7, IDX_R5_NOBI, IDX_PLAIN, IDX_SEG8] {
                zoo.run(k, &blk, &mut out_k);
                assert_eq!(out_k, text, "binary mismatch k={k} n={n} mask={mask}");
            }
            spec1.run_entry(&blk.records, n, p, &mut out_k);
            assert_eq!(out_k, text, "spec1 binary mismatch");
            spec2.run_entry(&blk.records, n, p, &mut out_k);
            assert_eq!(out_k, text, "spec2 binary mismatch");
            valid_cnt += 1;
        }
    }

    let mut rng = 0xA076_1D64_78BD_642Fu64;
    for case in 0..2_000usize {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let n = 1 + ((rng >> 32) as usize % 512);
        let alpha = match case % 7 {
            0 => 2u16,
            1 => 3,
            2 => 4,
            3 => 8,
            4 => 16,
            5 => 26,
            _ => 256,
        };
        text.clear();
        for _ in 0..n {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            text.push(((rng >> 40) % alpha as u64) as u8);
        }
        let (l, p) = bwt_small(&text);
        let blk = Block {
            name: String::new(),
            n,
            p,
            route_class: 2,
            records: records_of(&l),
            expected: None,
        };
        for k in 0..NUM_ZOO {
            zoo.run(k, &blk, &mut out_k);
            assert_eq!(out_k, text, "rand mismatch k={k} case={case} n={n}");
        }
        spec1.run_entry(&blk.records, n, p, &mut out_k);
        assert_eq!(out_k, text);
        spec2.run_entry(&blk.records, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    // 5b. Stress blocks (dense-300k, spiral-200k, periodic-150k-d2, periodic-105k-d3)
    let dense = make_dense_300k();
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);
    let stress = [&dense, &spiral, &per2, &per3];

    for b in six.iter().chain(stress.iter().copied()).chain(non_wiki.iter()) {
        let want = b.expected.as_ref().unwrap();
        for k in 0..NUM_ZOO {
            zoo.run(k, b, &mut out_k);
            assert_eq!(&out_k, want, "mismatch on {} k={}", b.name, zoo_name(k));
        }
        spec1.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "spec1 mismatch on {}", b.name);
        spec2.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "spec2 mismatch on {}", b.name);

        // Forced exact fallback test!
        spec1.force_fallback = true;
        let path1 = spec1.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(path1, 0);
        assert_eq!(&out_k, want, "spec1 forced fallback mismatch on {}", b.name);
        spec1.force_fallback = false;

        spec2.force_fallback = true;
        let path2 = spec2.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(path2, 0);
        assert_eq!(&out_k, want, "spec2 forced fallback mismatch on {}", b.name);
        spec2.force_fallback = false;
    }

    // Verify spiral-200k triggers periodic specialist rejection -> exact fallback to R5-Head!
    let spiral_route = spec1.run_entry(&spiral.records, spiral.n, spiral.p, &mut out_k);
    assert_eq!(spiral_route, 0, "spiral-200k must fail periodic probe and fall back to R5-Head");
    assert_eq!(&out_k, spiral.expected.as_ref().unwrap());
    let per2_route = spec1.run_entry(&per2.records, per2.n, per2.p, &mut out_k);
    assert_eq!(per2_route, 2, "periodic-150k-d2 must hit periodic specialist");
    assert_eq!(&out_k, per2.expected.as_ref().unwrap());

    // 5c. All 431 wikidata.bz2 blocks: verify R5-Head, R5-Nobi, OpenH(L4), OpenH(L64), Plain, Seg8, Spec1, Spec2
    for (idx, b) in wiki431.iter().enumerate() {
        zoo.run(IDX_PLAIN, b, &mut out_ref);
        for &k in &[0, 1, 2, IDX_R5_HEAD, 5, 7, IDX_R5_NOBI, IDX_SEG8] {
            zoo.run(k, b, &mut out_k);
            assert_eq!(out_k, out_ref, "wiki431 mismatch idx={idx} k={}", zoo_name(k));
        }
        spec1.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 spec1 mismatch idx={idx}");
        spec2.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 spec2 mismatch idx={idx}");
    }

    // 5d. 200 malformed / multi-cycle columns: assert exact forward-FL parity against
    //     cand_r5_head (matching libbzip2 / mibz2::bwt::InverseBwt) + broken-tiling rejection
    assert!(!zoo.plain.try_invert(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));
    assert!(!zoo.seg8.try_invert(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));

    let mut lbuf = Vec::with_capacity(16_384);
    for case in 0..200usize {
        let n = if case < 150 { 1 + (case * 37) % 512 } else { 8_192 };
        let alpha = 2 + (case % 12);
        lbuf.clear();
        for _ in 0..n {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            lbuf.push(((rng >> 40) % alpha as u64) as u8);
        }
        let p = ((rng >> 32) as usize % n) as u32;
        let blk = Block {
            name: String::new(),
            n,
            p,
            route_class: 3,
            records: records_of(&lbuf),
            expected: None,
        };
        zoo.run(IDX_R5_HEAD, &blk, &mut out_ref);
        zoo.run(IDX_PLAIN, &blk, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed PlainRunWalk (FL) vs cand_r5_head mismatch case={case}");
        zoo.run(IDX_SEG8, &blk, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed InterleavedWalk (FL) vs cand_r5_head mismatch case={case}");
        spec1.run_entry(&blk.records, blk.n, blk.p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Spec1-TinyDense vs cand_r5_head mismatch case={case}");
        spec2.run_entry(&blk.records, blk.n, blk.p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Spec2-Adaptive vs cand_r5_head mismatch case={case}");
    }

    println!(
        "EXACTNESS PASS: {valid_cnt} small valid BWTs + 200 malformed (forward-FL vs cand_r5_head) + 4 stress (dense/spiral/per2/per3) + {} admitted + {} wikidata.bz2 + {} non-Wikidata + forced fallbacks 100% BYTE-EXACT!",
        six.len(),
        wiki431.len(),
        non_wiki.len()
    );
}

// ============================================================================
// 6. Analogue D 431-Block Structural Census
// ============================================================================

fn run_431_census(wiki431: &[Block], non_wiki: &[Block]) {
    println!("\n=== ANALOGUE D STRUCTURAL CENSUS ACROSS ALL 431 wikidata.bz2 BLOCKS ===");
    let mut kh = cand_r5_head::Kernel::default();
    let mut out = Vec::with_capacity(900_064);
    let rnames = ["normal (r/n<0.15)", "dense (0.15..0.28)", "tiny/sparse", "column-route"];

    let mut cnt = [0usize; 4];
    let mut sum_n = [0usize; 4];
    let mut sum_r = [0usize; 4];
    let mut sum_m = [0usize; 4];
    let mut sum_ns = [0usize; 4];
    let mut sum_bands = [0usize; 4];
    let mut zero_strip = [0usize; 4];
    let mut ideal_m1 = [0usize; 4];
    let mut spirals_cnt = [0usize; 4];
    let mut lt65k_m = [0usize; 4];

    for b in wiki431 {
        let rc = b.route_class as usize;
        let st = kh
            .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out)
            .unwrap();
        let max_r = count_maximal_runs(&b.records);
        cnt[rc] += 1;
        sum_n[rc] += b.n;
        sum_r[rc] += max_r;
        sum_m[rc] += st.retained;
        sum_ns[rc] += st.thick;
        sum_bands[rc] += st.bands;
        if st.thick == 0 {
            zero_strip[rc] += 1;
        }
        if st.retained == 1 {
            ideal_m1[rc] += 1;
        }
        if st.spirals {
            spirals_cnt[rc] += 1;
        }
        if st.retained < 65_536 {
            lt65k_m[rc] += 1;
        }
    }
    println!(
        "{:20} | {:>4} | {:>9} | {:>7} | {:>8} | {:>6} | {:>6} | {:>6} | {:>5} | {:>5} | {:>5}",
        "route_class", "blks", "avg_n", "avg_r/n", "avg_m", "avg_m%", "avg_ns", "avg_bd", "ns=0", "m=1", "m<65k"
    );
    for rc in 0..4 {
        let c = cnt[rc].max(1) as f64;
        println!(
            "{:20} | {:4} | {:9.1} | {:7.4} | {:8.1} | {:5.1}% | {:6.1} | {:6.1} | {:5} | {:5} | {:5}",
            rnames[rc],
            cnt[rc],
            sum_n[rc] as f64 / c,
            sum_r[rc] as f64 / sum_n[rc].max(1) as f64,
            sum_m[rc] as f64 / c,
            100.0 * sum_m[rc] as f64 / sum_n[rc].max(1) as f64,
            sum_ns[rc] as f64 / c,
            sum_bands[rc] as f64 / c,
            zero_strip[rc],
            ideal_m1[rc],
            lt65k_m[rc]
        );
    }
    let tot_sp: usize = spirals_cnt.iter().sum();
    let tot_m1: usize = ideal_m1.iter().sum();
    let tot_z: usize = zero_strip.iter().sum();
    println!("TOTAL: 431 blocks | ideal Analogue-D (m==1): {tot_m1}/431 | zero-thick-strip (ns==0): {tot_z}/431 | spirals: {tot_sp}/431");

    println!("\nNON-WIKIDATA STRUCTURAL CENSUS (L=16):");
    for b in non_wiki {
        let st = kh
            .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out)
            .unwrap();
        let max_r = count_maximal_runs(&b.records);
        println!(
            "  {:10} | n={:6} | r={:5} (r/n={:.4}) | recs={:5} | m={:6} ({:5.2}%) | ns={:4} | bands={:5}",
            b.name,
            b.n,
            max_r,
            max_r as f64 / b.n as f64,
            b.records.len(),
            st.retained,
            100.0 * st.retained as f64 / b.n as f64,
            st.thick,
            st.bands
        );
    }
}

// ============================================================================
// 7. Headroom Oracle Benchmarks (1T, 2T, 4W) on 6 Admitted, 431 Wikidata, Non-Wikidata
// ============================================================================

fn bench_six_admitted(six: &[Block]) {
    println!("\n=== HEADROOM ORACLE v1: 6 ADMITTED BLOCKS (1T 40 rds/blk, A/A + 18-Kernel Zoo) ===");
    let mut zoo = ZooState::default();
    let mut kh_a1 = cand_r5_head::Kernel::default();
    let mut kh_a2 = cand_r5_head::Kernel::default();
    let mut out = Vec::with_capacity(900_064);

    // Warmup
    for b in six {
        kh_a1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out);
        kh_a2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out);
        for k in 0..NUM_ZOO {
            zoo.run(k, b, &mut out);
        }
    }

    // 7a. Interleaved A/A control (R5-Head A1 vs A2, 50 rounds/blk)
    let mut aa_sum = [0.0f64; 2];
    for b in six {
        let mut s: [Vec<f64>; 2] = Default::default();
        for i in 0..100 {
            let m = (i + i / 2) % 2;
            let t0 = Instant::now();
            if m == 0 {
                kh_a1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out);
            } else {
                kh_a2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out);
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        aa_sum[0] += pct(&mut s[0], 1, 2);
        aa_sum[1] += pct(&mut s[1], 1, 2);
    }
    println!(
        "1T A/A Control (R5-Head A1 vs A2, 50 rds): A1={:.1} us | A2={:.1} us ({:.4}x)",
        aa_sum[0],
        aa_sum[1],
        aa_sum[1] / aa_sum[0]
    );

    // 7b. Full 18-kernel Zoo per block (30 interleaved rounds/blk for Open kernels, 10 for Plain/Seg8)
    let mut per_blk_med = vec![[0.0f64; NUM_ZOO]; six.len()];
    let mut sum_k = [0.0f64; NUM_ZOO];
    for (bi, b) in six.iter().enumerate() {
        // Interleave all 16 Open<L, Head/Nobi> kernels (30 rounds)
        let mut samples: [Vec<f64>; 16] = Default::default();
        for i in 0..16 * 30 {
            let k = (i + i / 16) % 16;
            let t0 = Instant::now();
            zoo.run(k, b, &mut out);
            samples[k].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        for k in 0..16 {
            let m = pct(&mut samples[k], 1, 2);
            per_blk_med[bi][k] = m;
            sum_k[k] += m;
        }
        // Plain-Walk & Seg8-LF (10 rounds)
        for &k in &[IDX_PLAIN, IDX_SEG8] {
            let mut sv = Vec::with_capacity(10);
            for _ in 0..10 {
                let t0 = Instant::now();
                zoo.run(k, b, &mut out);
                sv.push(t0.elapsed().as_secs_f64() * 1e6);
            }
            let m = pct(&mut sv, 1, 2);
            per_blk_med[bi][k] = m;
            sum_k[k] += m;
        }
    }

    let base_sum = sum_k[IDX_R5_HEAD];
    println!("\n1T 6-Block Totals across 18-Kernel Zoo (baseline = R5-Head L=16):");
    for k in 0..NUM_ZOO {
        println!(
            "  {:14} | sum={:8.1} us ({:.4}x vs R5-Head)",
            zoo_name(k),
            sum_k[k],
            sum_k[k] / base_sum
        );
    }

    // Per-block hindsight oracle
    let mut oracle_sum = 0.0f64;
    println!("\nPer-block Hindsight Oracle (6 Admitted Blocks, 1T):");
    for (bi, b) in six.iter().enumerate() {
        let mut best_k = 0usize;
        let mut best_t = f64::INFINITY;
        for k in 0..NUM_ZOO {
            if per_blk_med[bi][k] < best_t {
                best_t = per_blk_med[bi][k];
                best_k = k;
            }
        }
        let base_t = per_blk_med[bi][IDX_R5_HEAD];
        let nobi_t = per_blk_med[bi][IDX_R5_NOBI];
        oracle_sum += best_t;
        println!(
            "  {:12} | R5-Head={:7.1} | R5-Nobi={:7.1} ({:.4}x) | Best={:13} {:7.1} ({:.4}x, {:+.1} us)",
            b.name,
            base_t,
            nobi_t,
            nobi_t / base_t,
            zoo_name(best_k),
            best_t,
            best_t / base_t,
            best_t - base_t
        );
    }
    println!(
        "HINDSIGHT_ORACLE_6BLK_1T: Oracle={:.1} us vs R5-Head={:.1} us ({:.4}x, headroom={:+.1} us / {:.2}%)",
        oracle_sum,
        base_sum,
        oracle_sum / base_sum,
        oracle_sum - base_sum,
        100.0 * (base_sum - oracle_sum) / base_sum
    );
}

fn bench_431_wikidata(wiki431: &[Block]) {
    println!("\n=== HEADROOM ORACLE v1: ALL 431 wikidata.bz2 BLOCKS (1T, 3 passes/blk, broken down by route class) ===");
    // Subset of representative Zoo kernels spanning full frontier on all 431 blocks:
    // OpenH(L4, L8, L12, L16=R5-Head, L24, L32, L48, L64), OpenN(L8, L16=R5-Nobi, L32), Plain-Walk, Seg8-LF
    let ks = [0usize, 1, 2, 3, 4, 5, 6, 7, 9, 11, 13, IDX_PLAIN, IDX_SEG8];
    let nk = ks.len();
    let mut zoo = ZooState::default();
    let mut out = Vec::with_capacity(900_064);

    // Warmup on first normal block
    if let Some(b0) = wiki431.iter().find(|b| b.route_class == 0) {
        for &k in &ks {
            zoo.run(k, b0, &mut out);
        }
    }

    let mut sum_by_rc = vec![vec![0.0f64; nk]; 4];
    let mut oracle_by_rc = [0.0f64; 4];
    let mut win_counts = vec![[0usize; 4]; nk];

    // Also measure Specialist 1 (SpecTinyDense) & Specialist 2 (SpecAdaptive) directly on ALL 431 blocks!
    let mut spec1 = SpecTinyDense::default();
    let mut spec2 = SpecAdaptive::new(60_000, 24, 16);
    let mut spec1_by_rc = [0.0f64; 4];
    let mut spec2_by_rc = [0.0f64; 4];
    let mut spec1_paths = [0usize; 6];
    let mut spec2_paths = [0usize; 6];
    let mut head_idx_in_ks = 0usize;
    for (ci, &k) in ks.iter().enumerate() {
        if k == IDX_R5_HEAD {
            head_idx_in_ks = ci;
        }
    }

    for b in wiki431 {
        let rc = b.route_class as usize;
        // 3 rounds per block, take median (keeps total runtime ~20-35s for 431 blocks x 15 arms!)
        let mut best_ci = 0usize;
        let mut best_t = f64::INFINITY;
        for (ci, &k) in ks.iter().enumerate() {
            // Skip Plain-Walk on huge normal/column blocks ONLY? NO — run genuine Plain-Walk?
            // Wait: Plain-Walk on 281 900k blocks x 3 = ~15s, completely doable! Let's run ALL genuinely!
            let mut tv = [0.0f64; 3];
            for r in 0..3 {
                let t0 = Instant::now();
                zoo.run(k, b, &mut out);
                tv[r] = t0.elapsed().as_secs_f64() * 1e6;
            }
            let med = pct(&mut tv, 1, 2);
            sum_by_rc[rc][ci] += med;
            if med < best_t {
                best_t = med;
                best_ci = ci;
            }
        }
        oracle_by_rc[rc] += best_t;
        win_counts[best_ci][rc] += 1;

        let mut tv1 = [0.0f64; 3];
        let mut p1 = 0u8;
        for r in 0..3 {
            let t0 = Instant::now();
            p1 = spec1.run_entry(&b.records, b.n, b.p, &mut out);
            tv1[r] = t0.elapsed().as_secs_f64() * 1e6;
        }
        spec1_by_rc[rc] += pct(&mut tv1, 1, 2);
        spec1_paths[p1 as usize] += 1;

        let mut tv2 = [0.0f64; 3];
        let mut p2 = 0u8;
        for r in 0..3 {
            let t0 = Instant::now();
            p2 = spec2.run_entry(&b.records, b.n, b.p, &mut out);
            tv2[r] = t0.elapsed().as_secs_f64() * 1e6;
        }
        spec2_by_rc[rc] += pct(&mut tv2, 1, 2);
        spec2_paths[p2 as usize] += 1;
    }

    let rnames = ["normal(228)", "dense(5)", "tiny(150)", "column(48)"];
    let tot_head: f64 = (0..4).map(|rc| sum_by_rc[rc][head_idx_in_ks]).sum();
    let tot_oracle: f64 = oracle_by_rc.iter().sum();
    let tot_spec1: f64 = spec1_by_rc.iter().sum();
    let tot_spec2: f64 = spec2_by_rc.iter().sum();

    println!(
        "{:16} | {:>10} | {:>9} | {:>9} | {:>10} | {:>11} | {:>7}",
        "Kernel", rnames[0], rnames[1], rnames[2], rnames[3], "TOTAL_ms", "vs_Head"
    );
    for (ci, &k) in ks.iter().enumerate() {
        let tot: f64 = (0..4).map(|rc| sum_by_rc[rc][ci]).sum();
        println!(
            "{:16} | {:9.2}ms | {:8.2}ms | {:8.2}ms | {:9.2}ms | {:9.2}ms | {:6.4}x  (wins: {}/{}/{}/{})",
            zoo_name(k),
            sum_by_rc[0][ci] / 1e3,
            sum_by_rc[1][ci] / 1e3,
            sum_by_rc[2][ci] / 1e3,
            sum_by_rc[3][ci] / 1e3,
            tot / 1e3,
            tot / tot_head,
            win_counts[ci][0],
            win_counts[ci][1],
            win_counts[ci][2],
            win_counts[ci][3]
        );
    }
    println!(
        "{:16} | {:9.2}ms | {:8.2}ms | {:8.2}ms | {:9.2}ms | {:9.2}ms | {:6.4}x",
        "HINDSIGHT_ORACLE",
        oracle_by_rc[0] / 1e3,
        oracle_by_rc[1] / 1e3,
        oracle_by_rc[2] / 1e3,
        oracle_by_rc[3] / 1e3,
        tot_oracle / 1e3,
        tot_oracle / tot_head
    );
    println!(
        "{:16} | {:9.2}ms | {:8.2}ms | {:8.2}ms | {:9.2}ms | {:9.2}ms | {:6.4}x  (C_rec+C_mem < C_Open? {})",
        "Spec1-TinyDense",
        spec1_by_rc[0] / 1e3,
        spec1_by_rc[1] / 1e3,
        spec1_by_rc[2] / 1e3,
        spec1_by_rc[3] / 1e3,
        tot_spec1 / 1e3,
        tot_spec1 / tot_head,
        if tot_spec1 < tot_head { "YES" } else { "NO" }
    );
    println!(
        "{:16} | {:9.2}ms | {:8.2}ms | {:8.2}ms | {:9.2}ms | {:9.2}ms | {:6.4}x  (C_rec+C_mem < C_Open? {})",
        "Spec2-Adaptive",
        spec2_by_rc[0] / 1e3,
        spec2_by_rc[1] / 1e3,
        spec2_by_rc[2] / 1e3,
        spec2_by_rc[3] / 1e3,
        tot_spec2 / 1e3,
        tot_spec2 / tot_head,
        if tot_spec2 < tot_head { "YES" } else { "NO" }
    );
    println!("Spec1 dispatch paths [open_norm, tiny_plain, periodic, seg8_col, dense_open, nobi]: {:?}", spec1_paths);
    println!("Spec2 dispatch paths [open_norm, tiny_plain, periodic, seg8_col, dense_open, nobi]: {:?}", spec2_paths);

    // Also compute production-Open baseline (R5-Head on normal+dense, Seg8-LF on tiny+column) vs Oracle!
    let prod_proxy = sum_by_rc[0][head_idx_in_ks]
        + sum_by_rc[1][head_idx_in_ks]
        + sum_by_rc[2][nk - 1]
        + sum_by_rc[3][nk - 1];
    println!(
        "Note (Production OpenCells proxy = R5-Head on norm+dense + Seg8 on tiny+col): {:.2} ms | Oracle={:.2} ms ({:.4}x) | Spec1={:.2} ms ({:.4}x) | Spec2={:.2} ms ({:.4}x)",
        prod_proxy / 1e3,
        tot_oracle / 1e3,
        tot_oracle / prod_proxy,
        tot_spec1 / 1e3,
        tot_spec1 / prod_proxy,
        tot_spec2 / 1e3,
        tot_spec2 / prod_proxy
    );
    // Dedicated strictly-interleaved 4-arm A/A & Specialist comparison across all 431 blocks
    // (avoids L2 eviction artifact from Plain-Walk/Seg8-LF's 3.6 MiB buffer!)
    let mut kh_a1 = cand_r5_head::Kernel::default();
    let mut kh_a2 = cand_r5_head::Kernel::default();
    let mut sp1_i = SpecTinyDense::default();
    let mut sp2_i = SpecAdaptive::new(60_000, 24, 16);
    if let Some(b0) = wiki431.iter().find(|b| b.route_class == 0) {
        kh_a1.run_entry(&b0.records, b0.n, b0.p, cand_r5_head::Cap::NONE, &mut out);
        kh_a2.run_entry(&b0.records, b0.n, b0.p, cand_r5_head::Cap::NONE, &mut out);
        sp1_i.run_entry(&b0.records, b0.n, b0.p, &mut out);
        sp2_i.run_entry(&b0.records, b0.n, b0.p, &mut out);
    }
    let mut arm_tot = [0.0f64; 4];
    let mut arm_rc = [[0.0f64; 4]; 4];
    for b in wiki431 {
        let rc = b.route_class as usize;
        let mut s: [[f64; 3]; 4] = [[0.0; 3]; 4];
        for r in 0..3 {
            for off in 0..4 {
                let m = (r + off) % 4;
                let t0 = Instant::now();
                match m {
                    0 => { kh_a1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                    1 => { kh_a2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                    2 => { sp1_i.run_entry(&b.records, b.n, b.p, &mut out); }
                    _ => { sp2_i.run_entry(&b.records, b.n, b.p, &mut out); }
                }
                s[m][r] = t0.elapsed().as_secs_f64() * 1e6;
            }
        }
        for m in 0..4 {
            let med = pct(&mut s[m], 1, 2);
            arm_tot[m] += med;
            arm_rc[m][rc] += med;
        }
    }
    println!("\nINTERLEAVED 1T C_recognise + C_member vs C_Open (ALL 431 wikidata.bz2 blocks, 3 interleaved rds/blk):");
    let inames = ["R5-Head-A1", "R5-Head-A2(AA)", "Spec1-TinyDense", "Spec2-Adaptive"];
    for m in 0..4 {
        println!(
            "  {:16} | norm={:7.2}ms dense={:5.2}ms tiny={:4.2}ms col={:4.2}ms | TOTAL={:7.2}ms ({:.4}x) | < C_Open? {}",
            inames[m],
            arm_rc[m][0] / 1e3,
            arm_rc[m][1] / 1e3,
            arm_rc[m][2] / 1e3,
            arm_rc[m][3] / 1e3,
            arm_tot[m] / 1e3,
            arm_tot[m] / arm_tot[0],
            if m >= 2 { if arm_tot[m] < arm_tot[0] { "YES" } else { "NO" } } else { "-" }
        );
    }
}

fn bench_non_wikidata(non_wiki: &[Block]) {
    println!("\n=== HEADROOM ORACLE v1: NON-WIKIDATA CORPORA (5 JSONL + 1 SYSLOG, 1T 25 rds/blk) ===");
    let mut zoo = ZooState::default();
    let mut spec1 = SpecTinyDense::default();
    let mut spec2 = SpecAdaptive::new(60_000, 24, 16);
    let mut out = Vec::with_capacity(900_064);

    for b in non_wiki {
        for k in 0..NUM_ZOO {
            zoo.run(k, b, &mut out);
        }
        spec1.run_entry(&b.records, b.n, b.p, &mut out);
        spec2.run_entry(&b.records, b.n, b.p, &mut out);
    }

    let mut sum_k = [0.0f64; NUM_ZOO];
    let mut oracle_sum = 0.0f64;
    let mut spec1_sum = 0.0f64;
    let mut spec2_sum = 0.0f64;

    for b in non_wiki {
        let mut meds = [0.0f64; NUM_ZOO];
        for k in 0..16 {
            let mut tv = Vec::with_capacity(25);
            for _ in 0..25 {
                let t0 = Instant::now();
                zoo.run(k, b, &mut out);
                tv.push(t0.elapsed().as_secs_f64() * 1e6);
            }
            meds[k] = pct(&mut tv, 1, 2);
            sum_k[k] += meds[k];
        }
        // Interleave R5-Head, Spec1, Spec2 BEFORE Plain/Seg8 cache flush!
        let mut tv1 = Vec::with_capacity(25);
        let mut tv2 = Vec::with_capacity(25);
        let mut p2 = 0u8;
        for _ in 0..25 {
            let t0 = Instant::now();
            spec1.run_entry(&b.records, b.n, b.p, &mut out);
            tv1.push(t0.elapsed().as_secs_f64() * 1e6);
            let t1 = Instant::now();
            p2 = spec2.run_entry(&b.records, b.n, b.p, &mut out);
            tv2.push(t1.elapsed().as_secs_f64() * 1e6);
        }
        let m1 = pct(&mut tv1, 1, 2);
        let m2 = pct(&mut tv2, 1, 2);
        spec1_sum += m1;
        spec2_sum += m2;

        for &k in &[IDX_PLAIN, IDX_SEG8] {
            let mut tv = Vec::with_capacity(7);
            for _ in 0..7 {
                let t0 = Instant::now();
                zoo.run(k, b, &mut out);
                tv.push(t0.elapsed().as_secs_f64() * 1e6);
            }
            meds[k] = pct(&mut tv, 1, 2);
            sum_k[k] += meds[k];
        }

        let mut best_k = 0usize;
        let mut best_t = f64::INFINITY;
        for k in 0..NUM_ZOO {
            if meds[k] < best_t {
                best_t = meds[k];
                best_k = k;
            }
        }
        oracle_sum += best_t;
        println!(
            "  {:12} (n={:6}, recs={:5}, r/n={:.4}) | Head={:6.1} | Nobi={:6.1} ({:.4}x) | Best={:13} {:6.1} ({:.4}x) | Spec2(p={})={:6.1} ({:.4}x)",
            b.name,
            b.n,
            b.records.len(),
            b.records.len() as f64 / b.n as f64,
            meds[IDX_R5_HEAD],
            meds[IDX_R5_NOBI],
            meds[IDX_R5_NOBI] / meds[IDX_R5_HEAD],
            zoo_name(best_k),
            best_t,
            best_t / meds[IDX_R5_HEAD],
            p2,
            m2,
            m2 / meds[IDX_R5_HEAD]
        );
    }
    let base = sum_k[IDX_R5_HEAD];
    println!(
        "NON_WIKIDATA_1T_SUM: R5-Head={:.1} us | R5-Nobi={:.1} ({:.4}x) | Oracle={:.1} ({:.4}x) | Spec1={:.1} ({:.4}x) | Spec2={:.1} ({:.4}x, <Open? {})",
        base,
        sum_k[IDX_R5_NOBI],
        sum_k[IDX_R5_NOBI] / base,
        oracle_sum,
        oracle_sum / base,
        spec1_sum,
        spec1_sum / base,
        spec2_sum,
        spec2_sum / base,
        if spec2_sum < base { "YES" } else { "NO" }
    );
}

fn bench_mt_suite(label: &str, blocks: &[Block], passes_2t: usize, passes_4w: usize) {
    // Compare 6 arms under 2T and 4W concurrent barrier contention (noting 2-vCPU Xeon host):
    // 0: R5-Head A1, 1: R5-Head A2 (A/A), 2: R5-NobiW16, 3: OpenH(L8), 4: Spec1-TinyDense, 5: Spec2-Adaptive
    let arm_names = [
        "R5-Head-A1",
        "R5-Head-A2(AA)",
        "R5-NobiW16",
        "OpenH(L8)",
        "Spec1-TinyDense",
        "Spec2-Adaptive",
    ];
    for &(threads, passes) in &[(2usize, passes_2t), (4usize, passes_4w)] {
        println!("\n=== {} — {}W CONCURRENT BARRIER ({} passes, 2-vCPU Xeon host) ===", label, threads, passes);
        let mut rmt: [Vec<f64>; 6] = Default::default();
        for pass in 0..passes {
            for off in 0..6 {
                let m = (pass + off) % 6;
                let bar = Barrier::new(threads);
                let t = std::thread::scope(|sc| {
                    let bar_ref = &bar;
                    let mut hs = Vec::with_capacity(threads);
                    for tid in 0..threads {
                        hs.push(sc.spawn(move || {
                            let mut out = Vec::with_capacity(900_064);
                            let mut kh1 = cand_r5_head::Kernel::default();
                            let mut kh2 = cand_r5_head::Kernel::default();
                            let mut kn = cand_r5_nobi::Kernel::default();
                            let mut kl8 = cand_r5_head::Kernel::default();
                            let mut sp1 = SpecTinyDense::default();
                            let mut sp2 = SpecAdaptive::new(60_000, 24, 16);
                            // Warmup on first up to 6 blocks
                            for b in blocks.iter().take(6) {
                                match m {
                                    0 => { kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                                    1 => { kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                                    2 => { kn.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out); }
                                    3 => { kl8.try_run::<true, true>(&b.records, b.n, b.p, 8, cand_r5_head::Cap::NONE, &mut out); }
                                    4 => { sp1.run_entry(&b.records, b.n, b.p, &mut out); }
                                    _ => { sp2.run_entry(&b.records, b.n, b.p, &mut out); }
                                }
                            }
                            bar_ref.wait();
                            let t0 = Instant::now();
                            for idx in 0..blocks.len() {
                                let b = &blocks[(idx + tid) % blocks.len()];
                                match m {
                                    0 => { kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                                    1 => { kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                                    2 => { kn.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out); }
                                    3 => { kl8.try_run::<true, true>(&b.records, b.n, b.p, 8, cand_r5_head::Cap::NONE, &mut out); }
                                    4 => { sp1.run_entry(&b.records, b.n, b.p, &mut out); }
                                    _ => { sp2.run_entry(&b.records, b.n, b.p, &mut out); }
                                }
                            }
                            t0.elapsed().as_secs_f64() * 1e6
                        }));
                    }
                    hs.into_iter()
                        .map(|h| h.join().unwrap())
                        .fold(0.0f64, f64::max)
                });
                rmt[m].push(t);
            }
        }
        let b10 = pct(&mut rmt[0].clone(), 1, 10);
        let b50 = pct(&mut rmt[0].clone(), 1, 2);
        for m in 0..6 {
            let p10 = pct(&mut rmt[m], 1, 10);
            let p50 = pct(&mut rmt[m], 1, 2);
            println!(
                "  {}W {:16} | p10={:9.1} us ({:.4}x) | p50={:9.1} us ({:.4}x)",
                threads,
                arm_names[m],
                p10,
                p10 / b10,
                p50,
                p50 / b50
            );
        }
    }
}

fn main() {
    let six = load_six_admitted();
    let wiki_bin = "/tmp/mibz2_corpora/wikidata_431_blocks.bin";
    let nonwiki_bin = "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin";
    assert!(Path::new(wiki_bin).exists(), "missing {wiki_bin}");
    assert!(Path::new(nonwiki_bin).exists(), "missing {nonwiki_bin}");

    let wiki431 = load_bin_corpus(wiki_bin, "wiki", false);
    let non_wiki = load_bin_corpus(nonwiki_bin, "nonwiki", true);

    println!(
        "Loaded: {} admitted blocks, {} wikidata.bz2 blocks, {} non-Wikidata blocks",
        six.len(),
        wiki431.len(),
        non_wiki.len()
    );

    // 1. Exhaustive exactness + forced fallback audit
    run_exactness_audit(&six, &wiki431, &non_wiki);

    // 2. Analogue D per-region failure breakdown (6 admitted blocks) + 431-block & non-Wikidata census
    run_analogue_d_region_analysis(&six);
    run_431_census(&wiki431, &non_wiki);

    // 3. 1T Headroom Oracle on 6 admitted blocks, all 431 wikidata.bz2 blocks, and 6 non-Wikidata blocks
    bench_six_admitted(&six);
    bench_431_wikidata(&wiki431);
    bench_non_wikidata(&non_wiki);

    // 4. 2T and 4W concurrent barrier timings (6 admitted blocks, non-Wikidata blocks, and all 431 blocks)
    bench_mt_suite("6 ADMITTED BLOCKS", &six, 30, 20);
    bench_mt_suite("6 NON-WIKIDATA BLOCKS", &non_wiki, 20, 15);
    bench_mt_suite("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, 5);
}
