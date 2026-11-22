//! Selective High-Yield Ephemeral Copy-Ribbon Kernel directly over `cand_r5_head`'s
//! verbatim `endpoint.rs` and `outq.rs` modules (`#![forbid(unsafe_code)]`).
//!
//! Zero O(r) boundary tables:
//! - Pass 1 (`S1`): compile-time `const COLLECT_SEEDS` harvests compact `(m << 5) | w`
//!   ribbon seeds ONLY for thin runs with `w >= cfg.w_min` passing Poincare-Kac
//!   selective section `(m & cfg.stride_mask) == 0`.
//! - Elimination (`S2b`/`S3a`): 100% identical to `cand_r5_head` (zero strip inflation).
//! - Wiring (`S4`): fast sequential directed `OutQ16::wire` (zero `wire_xor` random RMWs).
//! - Ribbon Fusion (`S4.5`): supports both MLP=1 (`contract_ribbons_serial`) and
//!   8-way lockstep MLP-interleaved (`contract_ribbons_mlp8`) with 64-bit Bloom
//!   disjointness fast-filter, Wald SPRT early-abort, zero-memory in-place `u32::MAX`
//!   tombstoning of permanently dead interior nodes, and strict L1D dictionary cap.
//! - Emit (`S5`): unidirectional `emit` (`Ribbon-Uni`, +0 B extra edge array) or
//!   live-only XOR conversion + `emit_bidirectional` (`Ribbon-Bi`).

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/endpoint.rs"]
mod endpoint;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/outq.rs"]
mod outq;

use std::time::Instant;

pub const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const MAX_RIBBON_STEPS: usize = 64;
const MAX_COMP_BYTES: usize = 512;
const MLP_LANES: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct RibbonConfig {
    pub name: &'static str,
    pub enabled: bool,
    pub use_mlp8: bool,
    pub w_min: u8,
    pub stride_mask: u32,
    pub min_steps: u32,
    pub min_yield: u32,
    /// Cap on hot L1D table `17 * lens.len()` (inline16 + lens)
    pub l1_dict_cap_bytes: usize,
    /// Cap on additional bytes appended to `spelling` by ribbons
    pub added_spell_cap_bytes: usize,
    pub use_bi_xor: bool,
    /// If true, run Wald SPRT early-abort after probing `wald_window` seeds.
    pub wald_sprt: bool,
    pub wald_window: usize,
    pub wald_min_hits: usize,
    /// Force `use_bi_xor` even on small blocks (for exhaustive exactness testing).
    pub force_bi_small: bool,
}

impl RibbonConfig {
    pub const NOBI_CTRL: Self = Self {
        name: "Ctrl-Nobi",
        enabled: false,
        use_mlp8: false,
        w_min: 17,
        stride_mask: 0,
        min_steps: 2,
        min_yield: 32,
        l1_dict_cap_bytes: 32_768,
        added_spell_cap_bytes: 32_768,
        use_bi_xor: false,
        wald_sprt: false,
        wald_window: 64,
        wald_min_hits: 4,
        force_bi_small: false,
    };
}

#[derive(Default, Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Counts {
    pub open_nodes: usize,
    pub steps: usize,
    pub rules: usize,
    pub n: usize,
    pub runs: usize,
    pub retained: usize,
    pub thick: usize,
    pub bands: usize,
    pub words: usize,
    pub copies: usize,
    pub traced: usize,
    pub thick_area: usize,
    pub unit_work: usize,
    pub longest: usize,
    pub table_bytes: usize,
    pub spirals: bool,
    // Analogue E ribbon telemetry
    pub seeds_collected: usize,
    pub seeds_probed: usize,
    pub wald_aborted: bool,
    pub ribbons_committed: usize,
    pub hops_saved: usize,
    pub trace_steps_total: usize,
    pub open_dict_bytes: usize,
    pub final_dict_bytes: usize,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct StageTimesUs {
    pub s1_pass: f64,
    pub s23_elim: f64,
    pub s4_wire: f64,
    pub s45_ribbon: f64,
    pub s46_xor: f64,
    pub s5_emit: f64,
}

impl StageTimesUs {
    pub fn total(&self) -> f64 {
        self.s1_pass + self.s23_elim + self.s4_wire + self.s45_ribbon + self.s46_xor + self.s5_emit
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

impl Strip {
    #[inline(always)]
    fn unpack(w: &[u32]) -> Self {
        Strip {
            from: w[0],
            to: w[1],
            width: w[2] >> 8,
            byte: (w[2] & 0xFF) as u8,
        }
    }
}

#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        debug_assert!(end <= 900_000 && dst <= 900_000 && code < (1 << 24));
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 {
        (self.0 & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn dst(self) -> u32 {
        ((self.0 >> 20) & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn code(self) -> u32 {
        (self.0 >> 40) as u32
    }
}

#[derive(Default)]
pub struct Kernel {
    endpoint: endpoint::Endpoint,
    spill: outq::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    xor_edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    seeds: Vec<u32>,
}

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            runs: 0,
        }
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

#[inline(always)]
fn split(
    c: u8,
    pos: u32,
    width: u32,
    p: u32,
    limit: u32,
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
    let end = pos + width;
    if width > limit && (pos..end).contains(&p) {
        if pos < p {
            visit(c, pos, p - pos, false);
        }
        visit(c, p, 1, true);
        if p + 1 < end {
            visit(c, p + 1, end - p - 1, false);
        }
    } else {
        visit(c, pos, width, width <= limit);
    }
}

struct Pass<'a> {
    queues: outq::Queues<'a>,
    edges: &'a mut Vec<u32>,
    seeds: &'a mut Vec<u32>,
    ns: usize,
    f: First,
    w_min: u32,
    stride_mask: u32,
}

impl Pass<'_> {
    #[inline(always)]
    fn take<const COLLECT_SEEDS: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        self.f.runs += 1;
        if width <= limit {
            self.piece::<COLLECT_SEEDS>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<COLLECT_SEEDS>(c, pos, width, false, p);
        } else {
            self.take_marked::<COLLECT_SEEDS>(c, pos, width, p, limit);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked<const COLLECT_SEEDS: bool>(
        &mut self,
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
    ) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<COLLECT_SEEDS>(c, x, w, keep, p)
        });
    }

    #[cold]
    #[inline(never)]
    fn push_seed_cold(&mut self, packed: u32) {
        self.seeds.push(packed);
    }

    #[inline(always)]
    fn piece<const COLLECT_SEEDS: bool>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            let cur_m = f.m;
            self.queues.push_thin(a, cur_m, w);
            f.m = cur_m + w;
            if COLLECT_SEEDS && w >= self.w_min && (cur_m & self.stride_mask) == 0 {
                self.push_seed_cold((cur_m << 5) | w);
            }
        } else {
            let at = 3 * self.ns;
            if self.edges.len() < at + 3 {
                let need = (at + 3).max(2 * self.edges.len()).max(1024);
                self.edges.reserve_exact(need - self.edges.len());
                self.edges.resize(need, 0);
            }
            let thin_a = self.queues.thin_of(a);
            self.edges[at..at + 3].copy_from_slice(&[
                x,
                thin_a + f.thick_rows[a],
                (w << 8) | u32::from(c),
            ]);
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
}

fn begin_inline(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
}

#[inline(always)]
fn word_len_of(code: usize, inline16: &[[u8; INLINE]], lens: &[u8]) -> usize {
    let l = lens[code];
    if l != LONG {
        usize::from(l)
    } else {
        let slot = &inline16[code];
        let s = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
        let e = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
        e - s
    }
}

#[inline(always)]
fn append_word_bytes(
    code: usize,
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &mut Vec<u8>,
) {
    let l = lens[code];
    if l != LONG {
        spelling.extend_from_slice(&inline16[code][..usize::from(l)]);
    } else {
        let slot = &inline16[code];
        let s = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
        let e = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
        spelling.extend_from_within(s..e);
    }
}

#[derive(Clone, Copy)]
struct MlpLane {
    active: bool,
    u0: u32,
    w: u8,
    cur: u32,
    len: u8,
    bloom: u64,
    path: [u32; MAX_RIBBON_STEPS],
    codes: [u32; MAX_RIBBON_STEPS],
    e0s: [u32; MAX_RIBBON_STEPS],
}

impl Default for MlpLane {
    fn default() -> Self {
        Self {
            active: false,
            u0: 0,
            w: 0,
            cur: 0,
            len: 0,
            bloom: 0,
            path: [0; MAX_RIBBON_STEPS],
            codes: [0; MAX_RIBBON_STEPS],
            e0s: [0; MAX_RIBBON_STEPS],
        }
    }
}

#[inline(always)]
fn bloom_bits(pos: u32, w: u32) -> u64 {
    let b1 = 1u64 << ((pos >> 4) & 63);
    let b2 = 1u64 << (((pos + w - 1) >> 4) & 63);
    b1 | b2
}

impl Kernel {
    #[inline(never)]
    pub fn run(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cfg: &RibbonConfig,
        out: &mut Vec<u8>,
    ) -> Counts {
        if cfg.enabled {
            self.run_inner::<false, true>(records, n, p, cfg, out).0
        } else {
            self.run_inner::<false, false>(records, n, p, cfg, out).0
        }
    }

    #[inline(never)]
    pub fn run_timed(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cfg: &RibbonConfig,
        out: &mut Vec<u8>,
    ) -> (Counts, StageTimesUs) {
        if cfg.enabled {
            self.run_inner::<true, true>(records, n, p, cfg, out)
        } else {
            self.run_inner::<true, false>(records, n, p, cfg, out)
        }
    }

    fn run_inner<const TIMED: bool, const COLLECT_SEEDS: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cfg: &RibbonConfig,
        out: &mut Vec<u8>,
    ) -> (Counts, StageTimesUs) {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let mut tm = StageTimesUs::default();
        let t0 = if TIMED { Some(Instant::now()) } else { None };

        out.resize(n + INLINE, 0);
        if COLLECT_SEEDS {
            self.seeds.clear();
            if n >= 65_536 && self.seeds.capacity() < 4096 {
                self.seeds.reserve(4096);
            }
        }
        let (ns, f, heads) = {
            let mut pass = Pass {
                queues: outq::Queues::new(&mut out[..], &mut self.spill, u32::MAX),
                edges: &mut self.edges,
                seeds: &mut self.seeds,
                ns: 0,
                f: First::default(),
                w_min: u32::from(cfg.w_min),
                stride_mask: cfg.stride_mask,
            };
            let mut pos = 0u32;
            let mut i = 0usize;
            let nrec = records.len();
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= LIMIT {
                    pass.piece::<COLLECT_SEEDS>(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take::<COLLECT_SEEDS>(c, pos, width, p, LIMIT);
                    pos += width;
                }
            }
            let mut f = pass.f;
            f.thin = pass.queues.extract_thin();
            let heads = pass.queues.heads();
            (pass.ns, f, heads)
        };
        if let Some(ts) = t0 {
            tm.s1_pass = ts.elapsed().as_secs_f64() * 1e6;
        }
        let t1 = if TIMED { Some(Instant::now()) } else { None };

        let First {
            thick_rows,
            thin,
            thick,
            m,
            mark,
            runs,
        } = f;
        let mut stats = Counts {
            n,
            runs,
            retained: m as usize,
            thick: ns,
            seeds_collected: if COLLECT_SEEDS { self.seeds.len() } else { 0 },
            ..Counts::default()
        };
        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        assert!(m > 0 && all.iter().sum::<u32>() == n as u32);
        let first = starts(all);
        let mut write = starts(thick);
        let need = self.arena_words(ns, m as usize);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }
        self.bands.clear();
        if n >= 65_536 && self.bands.capacity() < 8192 {
            self.bands.reserve_exact(8192);
        }
        self.spelling.clear();
        if n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }
        let cursors = self.induced(ns, n as u32, &thin, &first, &mut write, &mut stats);
        stats.bands = self.bands.len();
        stats.open_dict_bytes = self.lens.len() * (INLINE + 1);
        if let Some(ts) = t1 {
            tm.s23_elim = ts.elapsed().as_secs_f64() * 1e6;
        }
        let t2 = if TIMED { Some(Instant::now()) } else { None };

        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (m as usize + self.extra.len());

        // S4: Fast sequential directed wire
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
            for &b in &self.bands[cursors[a]..end] {
                let span = b.end() as usize;
                let dst = b.dst() as usize;
                let bcode = b.code();
                let pack = if wide { 0 } else { bcode << bits };
                if wide {
                    self.extra[dst..dst + span].fill(bcode);
                }
                queue.wire(&mut self.edges[dst..dst + span], pack);
            }
        }
        if let Some(ts) = t2 {
            tm.s4_wire = ts.elapsed().as_secs_f64() * 1e6;
        }
        let t3 = if TIMED { Some(Instant::now()) } else { None };

        // S4.5: Selective high-yield ephemeral copy-ribbon contraction directly on edges[0..m]
        if COLLECT_SEEDS && !wide && !self.seeds.is_empty() {
            if cfg.use_mlp8 {
                self.contract_ribbons_mlp8(m as usize, mark, bits, cfg, &mut stats);
            } else {
                self.contract_ribbons_serial(m as usize, mark, bits, cfg, &mut stats);
            }
        }
        stats.final_dict_bytes = self.lens.len() * (INLINE + 1);
        if let Some(ts) = t3 {
            tm.s45_ribbon = ts.elapsed().as_secs_f64() * 1e6;
        }
        let t4 = if TIMED { Some(Instant::now()) } else { None };

        // Optional S4.6: Live-only XOR conversion for bidirectional emit (`Ribbon-Bi`)
        let do_bi = cfg.use_bi_xor && !wide && (m >= 65_536 || cfg.force_bi_small);
        let mut predecessor = mark;
        if do_bi {
            let mu = m as usize;
            if self.xor_edges.len() < mu {
                self.xor_edges.resize(mu, 0);
            }
            let xedges = &mut self.xor_edges[..mu];
            xedges.fill(0);
            let mask = (1u32 << bits) - 1;
            let edges = &self.edges[..mu];
            for u in 0..mu {
                let e = edges[u];
                if e != u32::MAX {
                    let v = (e & mask) as usize;
                    if v == mark as usize {
                        predecessor = u as u32;
                    }
                    xedges[u] ^= e;
                    xedges[v] ^= u as u32;
                }
            }
        }
        if let Some(ts) = t4 {
            tm.s46_xor = ts.elapsed().as_secs_f64() * 1e6;
        }
        let t5 = if TIMED { Some(Instant::now()) } else { None };

        // S5: Emit
        if do_bi {
            emit_bidirectional(
                &self.xor_edges[..m as usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                predecessor,
                bits,
                out,
            );
        } else {
            emit(
                &self.edges[..m as usize],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        }
        if let Some(ts) = t5 {
            tm.s5_emit = ts.elapsed().as_secs_f64() * 1e6;
        }
        (stats, tm)
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn try_commit_ribbon(
        edges: &mut [u32],
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        spelling: &mut Vec<u8>,
        l1_dict_bytes: &mut usize,
        added_spell_bytes: &mut usize,
        max_codes: usize,
        bits: u32,
        mask: u32,
        cfg: &RibbonConfig,
        u0: usize,
        w: usize,
        len: usize,
        path: &[u32],
        codes: &[u32],
        e0s: Option<&[u32]>,
    ) -> bool {
        if lens.len() >= max_codes || *l1_dict_bytes + (INLINE + 1) > cfg.l1_dict_cap_bytes {
            return false;
        }
        // In MLP8 mode, re-verify that no concurrent lane modified or tombstoned any interval
        // in path[0..len] since this lane traced it (all cache lines are hot in L1D!).
        if let Some(exp) = e0s {
            for t in 0..len - 1 {
                let p = path[t] as usize;
                let e0 = exp[t];
                let sl = &edges[p..p + w];
                for k in 0..w {
                    if sl[k] != e0 + k as u32 {
                        return false;
                    }
                }
            }
            let last = path[len - 1] as usize;
            let clast = codes[len - 1];
            let sl = &edges[last..last + w];
            for &ek in sl {
                if ek == u32::MAX || (ek >> bits) != clast {
                    return false;
                }
            }
        }

        let mut total_bytes = 0usize;
        for &cid in &codes[..len] {
            total_bytes += word_len_of(cid as usize, inline16, lens);
        }
        if total_bytes == 0 || total_bytes > MAX_COMP_BYTES {
            return false;
        }
        let spell_add = if total_bytes > INLINE { total_bytes } else { 0 };
        if *added_spell_bytes + spell_add > cfg.added_spell_cap_bytes {
            return false;
        }

        let new_code = lens.len() as u32;
        if total_bytes <= INLINE {
            let mut slot = [0u8; INLINE];
            let mut off = 0usize;
            for &cid in &codes[..len] {
                let cu = cid as usize;
                let l = lens[cu];
                if l != LONG {
                    let lu = usize::from(l);
                    slot[off..off + lu].copy_from_slice(&inline16[cu][..lu]);
                    off += lu;
                } else {
                    let wbytes = long_word(&inline16[cu], spelling);
                    slot[off..off + wbytes.len()].copy_from_slice(wbytes);
                    off += wbytes.len();
                }
            }
            inline16.push(slot);
            lens.push(total_bytes as u8);
        } else {
            let start = spelling.len() as u32;
            for &cid in &codes[..len] {
                append_word_bytes(cid as usize, inline16, lens, spelling);
            }
            let end = spelling.len() as u32;
            inline16.push(long_slot(start, end));
            lens.push(LONG);
        }
        *l1_dict_bytes += INLINE + 1;
        *added_spell_bytes += spell_add;

        let last = path[len - 1] as usize;
        let pack_new = new_code << bits;
        for k in 0..w {
            edges[u0 + k] = pack_new | (edges[last + k] & mask);
        }
        for &dead32 in &path[1..len] {
            let dead = dead32 as usize;
            for e in &mut edges[dead..dead + w] {
                *e = u32::MAX;
            }
        }
        true
    }

    /// MLP=1 serial ephemeral ribbon contraction with 64-bit Bloom disjointness fast-filter.
    #[inline(never)]
    fn contract_ribbons_serial(
        &mut self,
        m: usize,
        mark: u32,
        bits: u32,
        cfg: &RibbonConfig,
        stats: &mut Counts,
    ) {
        let mask = (1u32 << bits) - 1;
        let max_codes = (u32::MAX >> bits) as usize;
        let edges = &mut self.edges[..m];
        let mut l1_dict_bytes = self.lens.len() * (INLINE + 1);
        let mut added_spell_bytes = 0usize;

        let mut path = [0u32; MAX_RIBBON_STEPS];
        let mut codes = [0u32; MAX_RIBBON_STEPS];

        let mut probed = 0usize;
        let mut wald_hits = 0usize;
        let mut ribbons_ok = 0usize;
        let mut hops_saved = 0usize;
        let mut trace_steps = 0usize;

        for &seed in &self.seeds {
            if self.lens.len() >= max_codes
                || l1_dict_bytes + (INLINE + 1) > cfg.l1_dict_cap_bytes
                || added_spell_bytes >= cfg.added_spell_cap_bytes
            {
                break;
            }
            if cfg.wald_sprt && probed == cfg.wald_window && wald_hits < cfg.wald_min_hits {
                stats.wald_aborted = true;
                break;
            }
            let u0 = (seed >> 5) as usize;
            let w = (seed & 31) as usize;
            if u0 + w > m {
                continue;
            }
            let e0 = edges[u0];
            if e0 == u32::MAX {
                continue;
            }
            probed += 1;

            let v0 = (e0 & mask) as usize;
            if v0 + w > m {
                continue;
            }
            let mut rigid0 = true;
            {
                let sl = &edges[u0..u0 + w];
                for k in 1..w {
                    if sl[k] != e0 + k as u32 {
                        rigid0 = false;
                        break;
                    }
                }
            }
            if !rigid0 {
                continue;
            }

            let c0 = e0 >> bits;
            path[0] = u0 as u32;
            codes[0] = c0;
            let mut bloom = bloom_bits(u0 as u32, w as u32);
            let mut len = 1usize;
            let mut cur = v0;
            let mut cur_steps = 1usize;

            while len < MAX_RIBBON_STEPS {
                if cur + w > m || mark.wrapping_sub(cur as u32) < w as u32 {
                    break;
                }
                let ec = edges[cur];
                if ec == u32::MAX {
                    break;
                }
                let cur32 = cur as u32;
                let w32 = w as u32;
                let bm = bloom_bits(cur32, w32);
                if (bloom & bm) != 0 {
                    let mut overlap = false;
                    for &prev in &path[..len] {
                        let diff = if cur32 >= prev { cur32 - prev } else { prev - cur32 };
                        if diff < w32 {
                            overlap = true;
                            break;
                        }
                    }
                    if overlap {
                        break;
                    }
                }
                bloom |= bm;

                cur_steps += 1;
                let cc = ec >> bits;
                let vc = (ec & mask) as usize;
                let mut is_rigid = vc + w <= m;
                let mut same_code = true;
                {
                    let sl = &edges[cur..cur + w];
                    if is_rigid {
                        for k in 1..w {
                            let ek = sl[k];
                            if ek != ec + k as u32 {
                                is_rigid = false;
                                if ek == u32::MAX || (ek >> bits) != cc {
                                    same_code = false;
                                }
                                for &er in &sl[k + 1..w] {
                                    if er == u32::MAX || (er >> bits) != cc {
                                        same_code = false;
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    } else {
                        for &er in &sl[1..w] {
                            if er == u32::MAX || (er >> bits) != cc {
                                same_code = false;
                                break;
                            }
                        }
                    }
                }
                if !same_code {
                    break;
                }
                path[len] = cur32;
                codes[len] = cc;
                len += 1;
                if is_rigid {
                    cur = vc;
                } else {
                    break;
                }
            }
            trace_steps += cur_steps;

            if len >= 2 {
                let saved = (w as u32) * (len as u32 - 1);
                if (len as u32) >= cfg.min_steps && saved >= cfg.min_yield {
                    wald_hits += 1;
                    if Self::try_commit_ribbon(
                        edges,
                        &mut self.inline16,
                        &mut self.lens,
                        &mut self.spelling,
                        &mut l1_dict_bytes,
                        &mut added_spell_bytes,
                        max_codes,
                        bits,
                        mask,
                        cfg,
                        u0,
                        w,
                        len,
                        &path,
                        &codes,
                        None,
                    ) {
                        ribbons_ok += 1;
                        hops_saved += saved as usize;
                    }
                }
            }
        }

        stats.seeds_probed = probed;
        stats.ribbons_committed = ribbons_ok;
        stats.hops_saved = hops_saved;
        stats.trace_steps_total = trace_steps;
    }

    /// 8-way lockstep MLP-interleaved ephemeral ribbon contraction: overlaps up to 8
    /// independent L2/L3 `edges` pointer-chases simultaneously in out-of-order execution.
    #[inline(never)]
    fn contract_ribbons_mlp8(
        &mut self,
        m: usize,
        mark: u32,
        bits: u32,
        cfg: &RibbonConfig,
        stats: &mut Counts,
    ) {
        let mask = (1u32 << bits) - 1;
        let max_codes = (u32::MAX >> bits) as usize;
        let edges = &mut self.edges[..m];
        let mut l1_dict_bytes = self.lens.len() * (INLINE + 1);
        let mut added_spell_bytes = 0usize;

        let mut lanes = [MlpLane::default(); MLP_LANES];
        let mut active_cnt = 0usize;
        let mut seed_idx = 0usize;
        let n_seeds = self.seeds.len();

        let mut probed = 0usize;
        let mut wald_hits = 0usize;
        let mut ribbons_ok = 0usize;
        let mut hops_saved = 0usize;
        let mut trace_steps = 0usize;
        let mut stop_refill = false;

        loop {
            // 1. Refill inactive lanes from seed stream
            if !stop_refill && active_cnt < MLP_LANES {
                for lane in &mut lanes {
                    if lane.active {
                        continue;
                    }
                    while seed_idx < n_seeds {
                        if self.lens.len() >= max_codes
                            || l1_dict_bytes + (INLINE + 1) > cfg.l1_dict_cap_bytes
                            || added_spell_bytes >= cfg.added_spell_cap_bytes
                        {
                            stop_refill = true;
                            break;
                        }
                        if cfg.wald_sprt && probed >= cfg.wald_window && wald_hits < cfg.wald_min_hits
                        {
                            stats.wald_aborted = true;
                            stop_refill = true;
                            break;
                        }
                        let seed = self.seeds[seed_idx];
                        seed_idx += 1;
                        let u0 = (seed >> 5) as usize;
                        let w = (seed & 31) as usize;
                        if u0 + w > m {
                            continue;
                        }
                        let e0 = edges[u0];
                        if e0 == u32::MAX {
                            continue;
                        }
                        probed += 1;
                        let v0 = (e0 & mask) as usize;
                        if v0 + w > m {
                            continue;
                        }
                        let sl = &edges[u0..u0 + w];
                        let mut rigid0 = true;
                        for k in 1..w {
                            if sl[k] != e0 + k as u32 {
                                rigid0 = false;
                                break;
                            }
                        }
                        if !rigid0 {
                            continue;
                        }
                        lane.active = true;
                        lane.u0 = u0 as u32;
                        lane.w = w as u8;
                        lane.cur = v0 as u32;
                        lane.len = 1;
                        lane.bloom = bloom_bits(u0 as u32, w as u32);
                        lane.path[0] = u0 as u32;
                        lane.codes[0] = e0 >> bits;
                        lane.e0s[0] = e0;
                        trace_steps += 1;
                        active_cnt += 1;
                        break;
                    }
                    if stop_refill || seed_idx >= n_seeds {
                        break;
                    }
                }
            }

            if active_cnt == 0 {
                break;
            }

            // 2. Advance ALL active lanes by 1 step in lockstep (MLP=8!)
            let mut finished_mask = 0u8;
            for (li, lane) in lanes.iter_mut().enumerate() {
                if !lane.active {
                    continue;
                }
                let cur = lane.cur as usize;
                let w = lane.w as usize;
                let len = lane.len as usize;
                if len >= MAX_RIBBON_STEPS
                    || cur + w > m
                    || mark.wrapping_sub(lane.cur) < u32::from(lane.w)
                {
                    finished_mask |= 1 << li;
                    continue;
                }
                let ec = edges[cur];
                if ec == u32::MAX {
                    finished_mask |= 1 << li;
                    continue;
                }
                let bm = bloom_bits(lane.cur, u32::from(lane.w));
                if (lane.bloom & bm) != 0 {
                    let mut overlap = false;
                    let w32 = u32::from(lane.w);
                    for &prev in &lane.path[..len] {
                        let diff = if lane.cur >= prev {
                            lane.cur - prev
                        } else {
                            prev - lane.cur
                        };
                        if diff < w32 {
                            overlap = true;
                            break;
                        }
                    }
                    if overlap {
                        finished_mask |= 1 << li;
                        continue;
                    }
                }
                lane.bloom |= bm;
                trace_steps += 1;

                let cc = ec >> bits;
                let vc = (ec & mask) as usize;
                let mut is_rigid = vc + w <= m;
                let mut same_code = true;
                let sl = &edges[cur..cur + w];
                if is_rigid {
                    for k in 1..w {
                        let ek = sl[k];
                        if ek != ec + k as u32 {
                            is_rigid = false;
                            if ek == u32::MAX || (ek >> bits) != cc {
                                same_code = false;
                            }
                            for &er in &sl[k + 1..w] {
                                if er == u32::MAX || (er >> bits) != cc {
                                    same_code = false;
                                    break;
                                }
                            }
                            break;
                        }
                    }
                } else {
                    for &er in &sl[1..w] {
                        if er == u32::MAX || (er >> bits) != cc {
                            same_code = false;
                            break;
                        }
                    }
                }
                if !same_code {
                    finished_mask |= 1 << li;
                    continue;
                }
                lane.path[len] = lane.cur;
                lane.codes[len] = cc;
                lane.e0s[len] = ec;
                lane.len = (len + 1) as u8;
                if is_rigid {
                    lane.cur = vc as u32;
                } else {
                    finished_mask |= 1 << li;
                }
            }

            // 3. Retire & commit any finished lanes
            while finished_mask != 0 {
                let li = finished_mask.trailing_zeros() as usize;
                finished_mask &= finished_mask - 1;
                let lane = &mut lanes[li];
                lane.active = false;
                active_cnt -= 1;
                let len = lane.len as usize;
                let w = lane.w as usize;
                if len >= 2 {
                    let saved = (w as u32) * (len as u32 - 1);
                    if (len as u32) >= cfg.min_steps && saved >= cfg.min_yield {
                        wald_hits += 1;
                        if Self::try_commit_ribbon(
                            edges,
                            &mut self.inline16,
                            &mut self.lens,
                            &mut self.spelling,
                            &mut l1_dict_bytes,
                            &mut added_spell_bytes,
                            max_codes,
                            bits,
                            mask,
                            cfg,
                            lane.u0 as usize,
                            w,
                            len,
                            &lane.path,
                            &lane.codes,
                            Some(&lane.e0s),
                        ) {
                            ribbons_ok += 1;
                            hops_saved += saved as usize;
                        }
                    }
                }
            }
        }

        stats.seeds_probed = probed;
        stats.ribbons_committed = ribbons_ok;
        stats.hops_saved = hops_saved;
        stats.trace_steps_total = trace_steps;
    }

    pub fn scratch_bytes(&self) -> usize {
        use std::mem::size_of;
        self.spill.scratch_bytes()
            + size_of::<Band>() * self.bands.capacity()
            + self.spelling.capacity()
            + 4 * self.edges.capacity()
            + 4 * self.xor_edges.capacity()
            + 4 * self.extra.capacity()
            + INLINE * self.inline16.capacity()
            + self.lens.capacity()
            + 4 * self.seeds.capacity()
            + self.endpoint.scratch_bytes()
    }
}

#[allow(clippy::too_many_arguments)]
fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bidirectional(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single {
            break;
        }
        let eb = edges[back as usize];
        let cb = (eb >> bits) as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}
