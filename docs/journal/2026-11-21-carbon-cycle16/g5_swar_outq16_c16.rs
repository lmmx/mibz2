//! Gate G5 Multi-Worker Native Safe-Rust & `chaingrind` Confrontation Harness for Carbon Cycle 16:
//!   - Baseline (`Arm 0`): `Master-0.5.8` (`mibz2::bwt::OpenCells` from `origin/master` `bb98c99`)
//!   - Prior    (`Arm 1`): `C15-EpochQ16-ZeroClearR4` (`[Vec<u16>; 256]` + `ZeroClearR4`, Cycle 15 winner)
//!   - Promoted (`Arm 2`): `C16-SwarOutQ16-CompactR4` (`#[forbid(unsafe_code)]` candidate module:
//!                         Theorem 16.1 `1 KiB` (`512 x u16`) 64K-Epoch Queues Aliased in Dead `out[0..n)`
//!                         with Exact `ovf_chunks` Spillover + Theorem 16.2a Carry-Free 64-Bit SWAR
//!                         16-Wide `u16` Iota (`(m & 0xFFFF) <= 0xFFF0`) + Theorem 16.2b Disjoint-Lifetime
//!                         `[u32; 3]` Metadata Coloring (`strips` + `bands_rev`) & `2*ns` `u32` Scratch)
//!
//! FROZEN PRE-REGISTERED CYCLE 16 PASS/FAIL GATES (all must hold simultaneously):
//!   (G5.1) Exactness & Zero-Unsafe: `#[forbid(unsafe_code)]` on `candidate_c16`; adversarial
//!          dirty-`out` + dirty-tape + tiny-block `ovf_chunks` spillover + multi-epoch (`LIMIT=64`)
//!          self-tests + 228/228 admitted blocks byte-exact + 431-block `BzReader` W=8,10 CRC check.
//!   (G5.2) Per-Worker Memory Reduction: Retained & Peak per-worker heap <= 0.42x `Master-0.5.8`
//!          AND <= 0.75x `C15`, AND mean unique L3 working set per block <= 2048 KiB (fitting W=10!).
//!   (G5.3) `chaingrind` i9-10900K Contention Proxy:
//!          - At W=8 (`LLC=2560KB`): `DRAM B/outB <= 0.25x 0.5.8` & `< C15`, `cycles/B < 0.5.8`.
//!          - At W=10 (`LLC=2048KB`): `DRAM B/outB <= 0.15x 0.5.8` & `<= 0.25x C15`,
//!            `L3 B/outB <= 0.85x 0.5.8`, `instr/B < 1.00x 0.5.8`, `cycles/B <= 0.88x 0.5.8` & `< C15`.
//!   (G5.4) Native Rust Multi-Worker Speed (15 paired interleaved rounds across 228 admitted blocks):
//!          `wall <= 0.935x Master-0.5.8` AND `sum_cpu <= 0.935x Master-0.5.8` at W=1, 2, 8, 10.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;

struct TrackAlloc;
thread_local! {
    static TLS_LIVE: Cell<isize> = const { Cell::new(0) };
    static TLS_PEAK: Cell<isize> = const { Cell::new(0) };
}
fn tls_reset() {
    TLS_LIVE.with(|l| {
        l.set(0);
        TLS_PEAK.with(|p| p.set(0));
    });
}
fn tls_stats() -> (usize, usize) {
    (
        TLS_LIVE.with(|l| l.get()).max(0) as usize,
        TLS_PEAK.with(|p| p.get()).max(0) as usize,
    )
}
unsafe impl GlobalAlloc for TrackAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let _ = TLS_LIVE.try_with(|l| {
                let nl = l.get() + layout.size() as isize;
                l.set(nl);
                let _ = TLS_PEAK.try_with(|pk| if nl > pk.get() { pk.set(nl) });
            });
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            let _ = TLS_LIVE.try_with(|l| {
                let nl = l.get() + layout.size() as isize;
                l.set(nl);
                let _ = TLS_PEAK.try_with(|pk| if nl > pk.get() { pk.set(nl) });
            });
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = TLS_LIVE.try_with(|l| l.set(l.get() - layout.size() as isize));
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let np = unsafe { System.realloc(ptr, layout, new_size) };
        if !np.is_null() {
            let d = new_size as isize - layout.size() as isize;
            let _ = TLS_LIVE.try_with(|l| {
                let nl = l.get() + d;
                l.set(nl);
                let _ = TLS_PEAK.try_with(|pk| if nl > pk.get() { pk.set(nl) });
            });
        }
        np
    }
}
#[global_allocator]
static GLOBAL: TrackAlloc = TrackAlloc;

#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn client_request(default: u64, args: &[u64; 6]) -> u64 {
    let mut r = default;
    let ptr = std::hint::black_box(args.as_ptr());
    unsafe {
        std::arch::asm!(
            "rol rdi, 3", "rol rdi, 13", "rol rdi, 61", "rol rdi, 51", "xchg rbx, rbx",
            inout("rdx") r,
            in("rax") ptr,
            options(nostack),
        );
    }
    r
}
const REQ_BEGIN: u64 = (b'M' as u64) << 24 | (b'S' as u64) << 16;
const REQ_END: u64 = REQ_BEGIN + 1;
const REQ_WARM: u64 = REQ_BEGIN + 2;

// ============================================================================
// PROMOTED CYCLE 16 CANDIDATE MODULE (`#[forbid(unsafe_code)]` — 100% Safe Rust!)
// ============================================================================
#[forbid(unsafe_code)]
pub mod candidate_c16 {
    use mibz2::bwt::{InverseBwt, LazyCells};
    use mibz2::runs::Runs;
    use mibz2::Kernels;

    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const LIMIT: u32 = 16;
    const PAD: usize = 1;
    const CAP: u32 = 64;
    const RULE: u32 = 1 << 31;
    const ARENA_MAX: usize = 0x00FF_F000;
    const CHUNK_U16: usize = 512;
    const CHUNK_B: usize = 1024;

    #[derive(Default, Clone, Copy)]
    struct Top {
        prev: u32,
        width: u32,
        word: u32,
        other: u32,
    }

    #[derive(Default, Clone, Copy)]
    struct Bot {
        prev: u32,
        top: u32,
    }

    #[derive(Clone, Copy)]
    struct Rule {
        left: u32,
        right: u32,
        len: u32,
        first: u32,
    }

    #[cold]
    #[inline(never)]
    fn grow_vec_tight<T: Clone>(v: &mut Vec<T>, need: usize, init: T) {
        let cur = v.len();
        let target = need.max(cur + (cur >> 2)).max(256);
        if v.capacity() < target {
            v.reserve_exact(target - cur);
        }
        v.resize(target, init);
    }

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let s = total;
            total += w;
            s
        })
    }

    struct PassTables {
        all: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        head: [u16; 256],
        tail: [u16; 256],
        in_chunk: [u16; 256],
        ep_cuts: Vec<[u32; 256]>,
    }

    impl Default for PassTables {
        fn default() -> Self {
            Self {
                all: [0; 256],
                thin: [0; 256],
                thick: [0; 256],
                head: [u16::MAX; 256],
                tail: [0; 256],
                in_chunk: [CHUNK_U16 as u16; 256],
                ep_cuts: vec![[0u32; 256]; 16],
            }
        }
    }

    struct PassOut<'a> {
        tb: &'a mut PassTables,
        chunk_next: &'a mut Vec<u16>,
        ovf_chunks: &'a mut Vec<[u8; CHUNK_B]>,
        triples: &'a mut Vec<[u32; 3]>,
        out: &'a mut [u8],
        m: u32,
        mark: u32,
        n_chunks: u16,
        max_out_chunks: u16,
        n_ep: usize,
    }

    impl PassOut<'_> {
        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, w: u32) {
            let mut rem = w;
            let mut cur_m = self.m;
            while rem > 0 {
                let ep = (cur_m >> 16) as usize;
                let ep_end = ((ep as u32) + 1) << 16;
                let mut cid = self.tb.tail[a];
                let mut ic = self.tb.in_chunk[a] as usize;
                if ic >= CHUNK_U16 {
                    cid = self.n_chunks;
                    self.n_chunks += 1;
                    if self.chunk_next.len() <= cid as usize {
                        grow_vec_tight(self.chunk_next, cid as usize + 1, u16::MAX);
                    }
                    self.chunk_next[cid as usize] = u16::MAX;
                    if cid >= self.max_out_chunks {
                        let need_ovf = (cid - self.max_out_chunks) as usize + 1;
                        if self.ovf_chunks.len() < need_ovf {
                            grow_vec_tight(self.ovf_chunks, need_ovf, [0u8; CHUNK_B]);
                        }
                    }
                    if self.tb.head[a] == u16::MAX {
                        self.tb.head[a] = cid;
                    } else {
                        self.chunk_next[self.tb.tail[a] as usize] = cid;
                    }
                    self.tb.tail[a] = cid;
                    ic = 0;
                }
                let tk = rem.min(ep_end - cur_m).min((CHUNK_U16 - ic) as u32);
                if tk <= 16
                    && ic <= CHUNK_U16 - 16
                    && (cur_m & 0xFFFF) <= 0xFFF0
                    && cid < self.max_out_chunks
                {
                    let x0 = (u64::from(cur_m & 0xFFFF) * 0x0001_0001_0001_0001)
                        + 0x0003_0002_0001_0000;
                    let b0 = (cid as usize) * CHUNK_B + ic * 2;
                    let dst = &mut self.out[b0..b0 + 32];
                    dst[0..8].copy_from_slice(&x0.to_le_bytes());
                    dst[8..16].copy_from_slice(&(x0 + 0x0004_0004_0004_0004).to_le_bytes());
                    dst[16..24].copy_from_slice(&(x0 + 0x0008_0008_0008_0008).to_le_bytes());
                    dst[24..32].copy_from_slice(&(x0 + 0x000C_000C_000C_000C).to_le_bytes());
                } else {
                    let dst_slice: &mut [u8] = if cid < self.max_out_chunks {
                        let b0 = (cid as usize) * CHUNK_B + ic * 2;
                        &mut self.out[b0..b0 + (tk as usize) * 2]
                    } else {
                        let oi = (cid - self.max_out_chunks) as usize;
                        &mut self.ovf_chunks[oi][ic * 2..(ic + tk as usize) * 2]
                    };
                    for (k, pair) in dst_slice.chunks_exact_mut(2).enumerate() {
                        let b = (cur_m as u16).wrapping_add(k as u16).to_le_bytes();
                        pair[0] = b[0];
                        pair[1] = b[1];
                    }
                }
                self.tb.in_chunk[a] = (ic + tk as usize) as u16;
                self.tb.thin[a] += tk;
                cur_m += tk;
                rem -= tk;
                if cur_m == ep_end {
                    self.tb.ep_cuts[ep] = self.tb.thin;
                    self.n_ep = ep + 2;
                }
            }
            self.m = cur_m;
        }

        #[inline(always)]
        fn push_thin(&mut self, a: usize, x: u32, w: u32, p: u32) {
            if p.wrapping_sub(x) < w {
                self.mark = self.m + (p - x);
            }
            let ic = self.tb.in_chunk[a] as usize;
            let cid = self.tb.tail[a];
            let m_now = self.m;
            let m_lo = m_now & 0xFFFF;
            if w <= 16 && ic <= CHUNK_U16 - 16 && m_lo <= 0xFFF0 && cid < self.max_out_chunks {
                // Theorem 16.2(a): Carry-Free 64-Bit SWAR 16-lane u16 iota!
                let x0 = (u64::from(m_lo) * 0x0001_0001_0001_0001) + 0x0003_0002_0001_0000;
                let b0 = (cid as usize) * CHUNK_B + ic * 2;
                let dst = &mut self.out[b0..b0 + 32];
                dst[0..8].copy_from_slice(&x0.to_le_bytes());
                dst[8..16].copy_from_slice(&(x0 + 0x0004_0004_0004_0004).to_le_bytes());
                dst[16..24].copy_from_slice(&(x0 + 0x0008_0008_0008_0008).to_le_bytes());
                dst[24..32].copy_from_slice(&(x0 + 0x000C_000C_000C_000C).to_le_bytes());
                self.tb.in_chunk[a] = (ic + w as usize) as u16;
                self.tb.thin[a] += w;
                self.m = m_now + w;
            } else {
                self.push_slow(a, w);
            }
            self.tb.all[a] += w;
        }

        #[inline(always)]
        fn push_thick(&mut self, c: u8, x: u32, w: u32) {
            let a = usize::from(c);
            self.triples.push([x, self.tb.all[a], (w << 8) | u32::from(c)]);
            self.tb.thick[a] += 1;
            self.tb.all[a] += w;
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            if pos < p {
                self.push_thick(c, pos, p - pos);
            }
            self.push_thin(usize::from(c), p, 1, p);
            if p + 1 < end {
                self.push_thick(c, p + 1, end - p - 1);
            }
        }

        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
            if width <= limit {
                self.push_thin(usize::from(c), pos, width, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.push_thick(c, pos, width);
            } else {
                self.take_marked(c, pos, width, p);
            }
        }
    }

    #[derive(Default)]
    pub struct SwarOutQ16Kernel {
        tb: Box<PassTables>,
        chunk_next: Vec<u16>,
        ovf_chunks: Vec<[u8; CHUNK_B]>,
        triples: Vec<[u32; 3]>,
        u32_scratch: Vec<u32>,
        tops: Vec<Top>,
        bots: Vec<Bot>,
        arena: Vec<u8>,
        rules: Vec<Rule>,
        stack: Vec<u32>,
        spelling: Vec<u8>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        pub last_unique_l3_bytes: usize,
        pub last_ovf_chunks_used: usize,
    }

    impl SwarOutQ16Kernel {
        pub fn poison_tapes_for_test(&mut self) {
            let n_t = self.tops.len().max(256);
            self.tops.resize(n_t, Top::default());
            self.tops.fill(Top {
                prev: 0xDEAD_BEEF,
                width: 0xDEAD_BEEF,
                word: 0xDEAD_BEEF,
                other: 0xDEAD_BEEF,
            });
            let n_b = self.bots.len().max(256);
            self.bots.resize(n_b, Bot::default());
            self.bots.fill(Bot {
                prev: 0xDEAD_BEEF,
                top: 0xDEAD_BEEF,
            });
            let n_u = self.u32_scratch.len().max(256);
            self.u32_scratch.resize(n_u, 0);
            self.u32_scratch.fill(0xDEAD_BEEF);
        }

        fn word_len(&self, w: u32) -> u32 {
            if w == 0 {
                0
            } else if w & RULE == 0 {
                w >> 24
            } else {
                self.rules[(w & !RULE) as usize].len
            }
        }

        fn join(&mut self, a: u32, b: u32) -> u32 {
            if a == 0 {
                return b;
            }
            if b == 0 {
                return a;
            }
            if (a | b) & RULE == 0 {
                let (al, bl) = (a >> 24, b >> 24);
                let tl = al + bl;
                if tl <= CAP && self.arena.len() + tl as usize <= ARENA_MAX {
                    let off = self.arena.len() as u32;
                    let (ao, bo) = ((a & 0xFF_FFFF) as usize, (b & 0xFF_FFFF) as usize);
                    self.arena.extend_from_within(ao..ao + al as usize);
                    self.arena.extend_from_within(bo..bo + bl as usize);
                    return (tl << 24) | off;
                }
            }
            let id = self.rules.len() as u32;
            let len = self.word_len(a) + self.word_len(b);
            self.rules.push(Rule {
                left: a,
                right: b,
                len,
                first: u32::MAX,
            });
            RULE | id
        }

        #[cold]
        #[inline(never)]
        fn make_long_code(&mut self, nd_word: u32, a: u8) -> u32 {
            let code = self.inline16.len() as u32;
            let o0 = self.spelling.len();
            self.stack.clear();
            self.stack.push(nd_word);
            while let Some(w) = self.stack.pop() {
                if w & RULE == 0 {
                    let off = (w & 0xFF_FFFF) as usize;
                    self.spelling
                        .extend_from_slice(&self.arena[off..off + (w >> 24) as usize]);
                } else {
                    let rule = &mut self.rules[(w & !RULE) as usize];
                    if rule.first == u32::MAX {
                        rule.first = self.spelling.len() as u32;
                        let (l, r) = (rule.left, rule.right);
                        self.stack.push(r);
                        self.stack.push(l);
                    } else {
                        let start = rule.first as usize;
                        self.spelling
                            .extend_from_within(start..start + rule.len as usize);
                    }
                }
            }
            self.spelling.push(a);
            let o1 = self.spelling.len();
            let mut v = [0u8; INLINE];
            v[0..4].copy_from_slice(&(o0 as u32).to_le_bytes());
            v[4..8].copy_from_slice(&(o1 as u32).to_le_bytes());
            self.inline16.push(v);
            self.lens.push(LONG);
            code
        }

        #[inline(always)]
        fn make_code(&mut self, nd_word: u32, a: u8) -> u32 {
            if nd_word == 0 {
                return u32::from(a);
            }
            if (nd_word & RULE) == 0 && (nd_word >> 24) < INLINE as u32 {
                let code = self.inline16.len() as u32;
                let wl = (nd_word >> 24) as usize;
                let off = (nd_word & 0xFF_FFFF) as usize;
                let mut v = [0u8; INLINE];
                v.copy_from_slice(&self.arena[off..off + INLINE]);
                v[wl] = a;
                self.inline16.push(v);
                self.lens.push((wl + 1) as u8);
                code
            } else {
                self.make_long_code(nd_word, a)
            }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_band_cross(
            edges: &mut [u32],
            out_bytes: &[u8],
            ovf_chunks: &[[u8; CHUNK_B]],
            chunk_next: &[u16],
            ep_cuts: &[[u32; 256]],
            n_ep: usize,
            max_out_chunks: u16,
            a: usize,
            cid: &mut u16,
            ic: &mut usize,
            ep: &mut usize,
            q_cur: &mut usize,
            mut cur_dst: usize,
            mut rem_span: usize,
            cpack: u32,
        ) {
            while rem_span > 0 {
                while *ep + 1 < n_ep && *q_cur >= ep_cuts[*ep][a] as usize {
                    *ep += 1;
                }
                let ep_hi = ep_cuts[*ep][a] as usize;
                if *ic >= CHUNK_U16 {
                    *cid = chunk_next[*cid as usize];
                    *ic = 0;
                }
                let tk = rem_span.min(CHUNK_U16 - *ic).min(ep_hi - *q_cur);
                let pack = cpack | ((*ep as u32) << 16);
                let src_b: &[u8] = if *cid < max_out_chunks {
                    let off_b = (*cid as usize) * CHUNK_B + *ic * 2;
                    &out_bytes[off_b..off_b + tk * 2]
                } else {
                    let oi = (*cid - max_out_chunks) as usize;
                    &ovf_chunks[oi][*ic * 2..(*ic + tk) * 2]
                };
                for (ed, pair) in edges[cur_dst..cur_dst + tk]
                    .iter_mut()
                    .zip(src_b.chunks_exact(2))
                {
                    let lo16 = u16::from_le_bytes([pair[0], pair[1]]);
                    *ed = pack | u32::from(lo16);
                }
                *ic += tk;
                *q_cur += tk;
                cur_dst += tk;
                rem_span -= tk;
            }
        }

        pub fn run(&mut self, records: &[u16], n: usize, p: u32, limit: u32, out: &mut Vec<u8>) {
            assert!(n > 0 && n <= 900_000 && (p as usize) < n);
            if out.len() < n + INLINE {
                out.resize(n + INLINE, 0);
            }
            let max_out_chunks = (out.len() / CHUNK_B).min(60_000) as u16;
            self.tb.all.fill(0);
            self.tb.thin.fill(0);
            self.tb.thick.fill(0);
            self.tb.head.fill(u16::MAX);
            self.tb.in_chunk.fill(CHUNK_U16 as u16);
            self.triples.clear();

            let (m, mark, n_chunks, mut n_ep) = {
                let mut pass = PassOut {
                    tb: &mut self.tb,
                    chunk_next: &mut self.chunk_next,
                    ovf_chunks: &mut self.ovf_chunks,
                    triples: &mut self.triples,
                    out: &mut out[..],
                    m: 0,
                    mark: 0,
                    n_chunks: 0,
                    max_out_chunks,
                    n_ep: 1,
                };
                let mut pos = 0u32;
                let mut idx = 0;
                if limit == LIMIT {
                    while idx < records.len() {
                        let c = (records[idx] >> 8) as u8;
                        let mut width = u32::from(records[idx] & 255);
                        idx += 1;
                        while idx < records.len() && (records[idx] >> 8) as u8 == c {
                            width += u32::from(records[idx] & 255);
                            idx += 1;
                        }
                        pass.take(c, pos, width, p, LIMIT);
                        pos += width;
                    }
                } else {
                    while idx < records.len() {
                        let c = (records[idx] >> 8) as u8;
                        let mut width = u32::from(records[idx] & 255);
                        idx += 1;
                        while idx < records.len() && (records[idx] >> 8) as u8 == c {
                            width += u32::from(records[idx] & 255);
                            idx += 1;
                        }
                        pass.take(c, pos, width, p, limit);
                        pos += width;
                    }
                }
                assert!(pass.m > 0 && pos == n as u32);
                (pass.m, pass.mark, pass.n_chunks, pass.n_ep)
            };

            let final_ep = (m >> 16) as usize;
            if n_ep <= final_ep {
                n_ep = final_ep + 1;
            }
            self.tb.ep_cuts[n_ep - 1] = self.tb.thin;

            let first = starts(self.tb.all);
            let mut write = starts(self.tb.thick);
            let ns = self.triples.len();
            if self.u32_scratch.len() < 2 * ns {
                grow_vec_tight(&mut self.u32_scratch, 2 * ns, 0u32);
            }
            for (i, s) in self.triples.iter_mut().enumerate() {
                let a = (s[2] & 255) as usize;
                s[1] += first[a];
                self.u32_scratch[write[a] as usize] = i as u32;
                write[a] += 1;
            }

            let slots = PAD + 3 * ns + 4;
            if self.tops.len() < slots {
                grow_vec_tight(&mut self.tops, slots, Top::default());
            }
            if self.bots.len() < slots {
                grow_vec_tight(&mut self.bots, slots, Bot::default());
            }
            self.tops[0] = Top::default();
            self.bots[0] = Bot::default();

            self.rules.clear();
            if self.arena.len() < 256 {
                self.arena.clear();
                self.arena.extend(0..=255u8);
            }
            self.arena.truncate(256);

            let p0_base = PAD as u32;
            let a0_base = p0_base + ns as u32 + 2;
            let (mut t, mut b) = (a0_base, PAD as u32);
            let mut x = 0u32;
            for (i, s) in self.triples.iter().enumerate() {
                let s_from = s[0];
                let s_width = s[2] >> 8;
                let s_byte = (s[2] & 255) as u8;
                if x < s_from {
                    self.tops[t as usize] = Top {
                        prev: t - 1,
                        width: s_from - x,
                        word: 0,
                        other: b,
                    };
                    self.bots[b as usize] = Bot {
                        prev: b - 1,
                        top: t,
                    };
                    t += 1;
                    b += 1;
                }
                self.u32_scratch[ns + i] = t;
                self.tops[t as usize] = Top {
                    prev: t - 1,
                    width: s_width,
                    word: (1 << 24) | u32::from(s_byte),
                    other: 0,
                };
                t += 1;
                x = s_from + s_width;
            }
            if x < n as u32 {
                self.tops[t as usize] = Top {
                    prev: t - 1,
                    width: n as u32 - x,
                    word: 0,
                    other: b,
                };
                self.bots[b as usize] = Bot {
                    prev: b - 1,
                    top: t,
                };
                t += 1;
                b += 1;
            }
            let a0_end = t;
            let mut p0 = p0_base;
            x = 0;
            for ii in 0..ns {
                let i = self.u32_scratch[ii] as usize;
                let s = self.triples[i];
                let s_to = s[1];
                let s_width = s[2] >> 8;
                if x < s_to {
                    self.tops[p0 as usize] = Top {
                        prev: p0 - 1,
                        width: s_to - x,
                        word: 0,
                        other: b,
                    };
                    self.bots[b as usize] = Bot {
                        prev: b - 1,
                        top: p0,
                    };
                    p0 += 1;
                    b += 1;
                }
                let st = self.u32_scratch[ns + i];
                self.tops[st as usize].other = b;
                self.bots[b as usize] = Bot {
                    prev: b - 1,
                    top: st,
                };
                b += 1;
                x = s_to + s_width;
            }
            if x < n as u32 {
                self.tops[p0 as usize] = Top {
                    prev: p0 - 1,
                    width: n as u32 - x,
                    word: 0,
                    other: b,
                };
                self.bots[b as usize] = Bot {
                    prev: b - 1,
                    top: p0,
                };
                p0 += 1;
                b += 1;
            }
            if p0 > p0_base {
                self.tops[p0_base as usize].prev = 0;
            }
            self.bots[PAD].prev = 0;
            self.tops[a0_base as usize].prev = if p0 > p0_base { p0 - 1 } else { 0 };

            let (mut sa, mut sb) = (a0_end - 1, b - 1);
            let mut remaining = n as u32;
            while remaining > 0 {
                let ta = self.tops[sa as usize];
                let (mut na, mut ra, mut aw, mut bs) = (ta.prev, ta.width, ta.word, ta.other);
                let bb = self.bots[sb as usize];
                let (mut nb, mut ts) = (bb.prev, bb.top);
                let tb = self.tops[ts as usize];
                let (mut rb, mut bw) = (tb.width, tb.word);
                let (mut fa, mut fb) = (sa, sb);
                if bs == sb {
                    remaining -= ra;
                    (sa, sb) = (na, nb);
                    continue;
                }
                loop {
                    let wd = self.join(aw, bw);
                    if ra == rb {
                        remaining -= ra;
                        let top = &mut self.tops[ts as usize];
                        (top.width, top.word, top.other) = (ra, wd, bs);
                        self.bots[bs as usize].top = ts;
                        (sa, sb) = (na, nb);
                        break;
                    } else if ra > rb {
                        remaining -= rb;
                        let top = &mut self.tops[ts as usize];
                        (top.width, top.word, top.other) = (rb, wd, bs);
                        self.bots[fb as usize].prev = self.bots[bs as usize].prev;
                        self.bots[bs as usize] = Bot { prev: fb, top: ts };
                        bs = fb;
                        ra -= rb;
                        fb = nb;
                        let bn = self.bots[nb as usize];
                        (nb, ts) = (bn.prev, bn.top);
                        let tn = self.tops[ts as usize];
                        (rb, bw) = (tn.width, tn.word);
                    } else {
                        remaining -= ra;
                        self.bots[bs as usize].top = ts;
                        self.tops[fa as usize].prev = self.tops[ts as usize].prev;
                        self.tops[ts as usize] = Top {
                            prev: fa,
                            width: ra,
                            word: wd,
                            other: bs,
                        };
                        ts = fa;
                        rb -= ra;
                        fa = na;
                        let tn = self.tops[na as usize];
                        (na, ra, aw, bs) = (tn.prev, tn.width, tn.word, tn.other);
                    }
                }
            }
            let mut xb = m;
            while sb != 0 {
                let s = self.bots[sb as usize];
                let w = self.tops[s.top as usize].width;
                xb -= w;
                self.tops[s.top as usize].other = xb;
                sb = s.prev;
            }
            assert_eq!(xb, 0);
            self.arena.extend_from_slice(&[0u8; INLINE]);

            if self.inline16.len() < 256 {
                self.inline16.clear();
                self.inline16.extend((0..=255u8).map(|c| {
                    let mut v = [0; INLINE];
                    v[0] = c;
                    v
                }));
                self.lens.clear();
                self.lens.resize(256, 1);
            }
            self.inline16.truncate(256);
            self.lens.truncate(256);
            self.spelling.clear();

            // Theorem 16.2(b): `strips` in `self.triples` is now dead; reuse for `bands_rev`!
            self.triples.clear();
            let thin_start = starts(self.tb.thin);
            let mut rev_start = [0usize; 256];
            let mut rem = if sa != 0 {
                self.tops[sa as usize].width
            } else {
                0
            };
            let mut kc = m;
            for a in (0..256).rev() {
                rev_start[a] = self.triples.len();
                let start = thin_start[a];
                while kc > start {
                    let nd = self.tops[sa as usize];
                    let width = (kc - start).min(rem);
                    rem -= width;
                    kc -= width;
                    let dst = nd.other + rem;
                    let code = self.make_code(nd.word, a as u8);
                    self.triples.push([width, dst, code]);
                    if rem == 0 {
                        sa = nd.prev;
                        rem = if sa != 0 {
                            self.tops[sa as usize].width
                        } else {
                            0
                        };
                    }
                }
            }
            assert_eq!(sa, 0);

            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let wide = ((self.inline16.len() - 1) as u32) > (u32::MAX >> bits);
            if self.edges.len() < m as usize {
                grow_vec_tight(&mut self.edges, m as usize, 0u32);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }

            {
                let out_bytes = &out[..];
                let bands = &self.triples[..];
                let ep_cuts = &self.tb.ep_cuts[..];
                for a in 0..256 {
                    let b0 = rev_start[a];
                    let b1 = if a > 0 { rev_start[a - 1] } else { bands.len() };
                    if b0 == b1 {
                        continue;
                    }
                    let mut cid = self.tb.head[a];
                    let mut ic = 0usize;
                    let mut ep = 0usize;
                    let mut q_cur = 0usize;
                    let mut ep_hi = ep_cuts[0][a] as usize;
                    for bnd in bands[b0..b1].iter().rev() {
                        let span = bnd[0] as usize;
                        let dst = bnd[1] as usize;
                        let code = bnd[2];
                        if wide {
                            self.extra[dst..dst + span].fill(code);
                        }
                        let cpack = if wide { 0 } else { code << bits };
                        if ic + span <= CHUNK_U16
                            && q_cur + span <= ep_hi
                            && cid < max_out_chunks
                        {
                            let pack = cpack | ((ep as u32) << 16);
                            let off_b = (cid as usize) * CHUNK_B + ic * 2;
                            let src_b = &out_bytes[off_b..off_b + span * 2];
                            for (ed, pair) in self.edges[dst..dst + span]
                                .iter_mut()
                                .zip(src_b.chunks_exact(2))
                            {
                                let lo16 = u16::from_le_bytes([pair[0], pair[1]]);
                                *ed = pack | u32::from(lo16);
                            }
                            ic += span;
                            q_cur += span;
                        } else {
                            Self::wire_band_cross(
                                &mut self.edges,
                                out_bytes,
                                &self.ovf_chunks,
                                &self.chunk_next,
                                ep_cuts,
                                n_ep,
                                max_out_chunks,
                                a,
                                &mut cid,
                                &mut ic,
                                &mut ep,
                                &mut q_cur,
                                dst,
                                span,
                                cpack,
                            );
                            ep_hi = ep_cuts[ep][a] as usize;
                        }
                    }
                }
            }

            let ovf_used = n_chunks.saturating_sub(max_out_chunks) as usize;
            self.last_ovf_chunks_used = ovf_used;
            // Unique L3 bytes touched: `out` (`n` bytes) ALREADY hosts the queues (`0` extra unique L3 bytes!),
            // and `triples` (`max(ns, nb) * 12`) is shared between `strips` and `bands_rev`!
            self.last_unique_l3_bytes = records.len() * 2
                + (n_chunks as usize) * 2
                + ovf_used * CHUNK_B
                + ns.max(self.triples.len()) * 12
                + ns * 8
                + ((p0 - p0_base + a0_end - a0_base) as usize) * std::mem::size_of::<Top>()
                + ((b as usize) - PAD) * std::mem::size_of::<Bot>()
                + self.arena.len()
                + self.rules.len() * std::mem::size_of::<Rule>()
                + self.spelling.len()
                + self.inline16.len() * INLINE
                + self.lens.len()
                + (m as usize) * 4
                + if wide { (m as usize) * 4 } else { 0 }
                + n;

            emit_hoisted(
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
    }

    #[cold]
    #[inline(never)]
    fn emit_long_word(
        meta: [u8; INLINE],
        spelling: &[u8],
        n: usize,
        at: &mut usize,
        out: &mut [u8],
    ) {
        let o0 = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
        let o1 = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
        let word = &spelling[o0..o1];
        let take = word.len().min(n - *at);
        out[*at..*at + take].copy_from_slice(&word[..take]);
        *at += take;
    }

    fn emit_hoisted(
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
        let mut at = 0usize;
        let mask = (1u32 << bits) - 1;
        let mut j = mark;
        if extra.is_empty() {
            for _ in 0..edges.len() {
                let edge = edges[j as usize];
                let code = (edge >> bits) as usize;
                j = edge & mask;
                let len = lens[code];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&inline16[code]);
                    at += usize::from(len);
                } else {
                    emit_long_word(inline16[code], spelling, n, &mut at, out);
                }
                if j == mark || at >= n {
                    break;
                }
            }
        } else {
            for _ in 0..edges.len() {
                let edge = edges[j as usize];
                let code = extra[j as usize] as usize;
                j = edge & mask;
                let len = lens[code];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&inline16[code]);
                    at += usize::from(len);
                } else {
                    emit_long_word(inline16[code], spelling, n, &mut at, out);
                }
                if j == mark || at >= n {
                    break;
                }
            }
        }
        out.truncate(at.min(n));
        let mut len = out.len();
        assert!(len > 0 && len <= n);
        while len < n {
            let take = len.min(n - len);
            out.extend_from_within(..take);
            len = out.len();
        }
    }

    #[derive(Default)]
    pub struct SwarOutQ16OpenCells {
        pub open: SwarOutQ16Kernel,
        lazy: LazyCells,
    }

    impl InverseBwt for SwarOutQ16OpenCells {
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
            let nr = r.records().len();
            if n > 0 && (p as usize) < n && nr * 20 < n * 3 && nr * 64 >= n {
                self.open.run(r.records(), n, p, LIMIT, out);
                return true;
            }
            self.lazy.invert_runs(r, n, p, out, k)
        }
        fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
            self.lazy.invert(l, c, p, out);
        }
        fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
            self.lazy.invert_in_place(l, c, p)
        }
    }
}

// ============================================================================
// Prior Promoted Reference (`Arm 1`: Cycle 15 `EpochQ16OpenCells`)
// ============================================================================
#[forbid(unsafe_code)]
mod prior_c15 {
    use mibz2::bwt::{InverseBwt, LazyCells};
    use mibz2::runs::Runs;
    use mibz2::Kernels;

    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const LIMIT: u32 = 16;
    const PAD: usize = 1;
    const CAP: u32 = 64;
    const RULE: u32 = 1 << 31;
    const ARENA_MAX: usize = 0x00FF_F000;

    #[derive(Clone, Copy)]
    struct Strip {
        from: u32,
        to: u32,
        width: u32,
        byte: u8,
    }
    #[derive(Clone, Copy)]
    struct BandRev {
        q_start: u32,
        dst: u32,
        code: u32,
    }
    #[derive(Default, Clone, Copy)]
    struct Top {
        prev: u32,
        width: u32,
        word: u32,
        other: u32,
    }
    #[derive(Default, Clone, Copy)]
    struct Bot {
        prev: u32,
        top: u32,
    }
    #[derive(Clone, Copy)]
    struct Rule {
        left: u32,
        right: u32,
        len: u32,
        first: u32,
    }

    #[cold]
    #[inline(never)]
    fn grow_vec_tight<T: Clone>(v: &mut Vec<T>, need: usize, init: T) {
        let cur = v.len();
        let target = need.max(cur + (cur >> 2)).max(256);
        if v.capacity() < target {
            v.reserve_exact(target - cur);
        }
        v.resize(target, init);
    }

    #[inline(always)]
    fn push_rows_u16(queue: &mut Vec<u16>, len: u32, m: u32) {
        let at = len as usize;
        let need = at + 16;
        if queue.len() < need {
            grow_vec_tight(queue, need, 0u16);
        }
        let base = m as u16;
        let iota: [u16; 16] = std::array::from_fn(|i| base.wrapping_add(i as u16));
        queue[at..at + 16].copy_from_slice(&iota);
    }

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let s = total;
            total += w;
            s
        })
    }

    #[derive(Default)]
    pub struct C15OpenCells {
        queues: Vec<Vec<u16>>,
        strips: Vec<Strip>,
        image_order: Vec<u32>,
        tops: Vec<Top>,
        bots: Vec<Bot>,
        strip_top: Vec<u32>,
        arena: Vec<u8>,
        rules: Vec<Rule>,
        stack: Vec<u32>,
        bands_rev: Vec<BandRev>,
        spelling: Vec<u8>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        lazy: LazyCells,
    }

    impl C15OpenCells {
        fn word_len(&self, w: u32) -> u32 {
            if w == 0 {
                0
            } else if w & RULE == 0 {
                w >> 24
            } else {
                self.rules[(w & !RULE) as usize].len
            }
        }
        fn join(&mut self, a: u32, b: u32) -> u32 {
            if a == 0 {
                return b;
            }
            if b == 0 {
                return a;
            }
            if (a | b) & RULE == 0 {
                let (al, bl) = (a >> 24, b >> 24);
                let tl = al + bl;
                if tl <= CAP && self.arena.len() + tl as usize <= ARENA_MAX {
                    let off = self.arena.len() as u32;
                    let (ao, bo) = ((a & 0xFF_FFFF) as usize, (b & 0xFF_FFFF) as usize);
                    self.arena.extend_from_within(ao..ao + al as usize);
                    self.arena.extend_from_within(bo..bo + bl as usize);
                    return (tl << 24) | off;
                }
            }
            let id = self.rules.len() as u32;
            let len = self.word_len(a) + self.word_len(b);
            self.rules.push(Rule {
                left: a,
                right: b,
                len,
                first: u32::MAX,
            });
            RULE | id
        }
        #[cold]
        #[inline(never)]
        fn make_long_code(&mut self, nd_word: u32, a: u8) -> u32 {
            let code = self.inline16.len() as u32;
            let o0 = self.spelling.len();
            self.stack.clear();
            self.stack.push(nd_word);
            while let Some(w) = self.stack.pop() {
                if w & RULE == 0 {
                    let off = (w & 0xFF_FFFF) as usize;
                    self.spelling
                        .extend_from_slice(&self.arena[off..off + (w >> 24) as usize]);
                } else {
                    let rule = &mut self.rules[(w & !RULE) as usize];
                    if rule.first == u32::MAX {
                        rule.first = self.spelling.len() as u32;
                        let (l, r) = (rule.left, rule.right);
                        self.stack.push(r);
                        self.stack.push(l);
                    } else {
                        let start = rule.first as usize;
                        self.spelling
                            .extend_from_within(start..start + rule.len as usize);
                    }
                }
            }
            self.spelling.push(a);
            let o1 = self.spelling.len();
            let mut v = [0u8; INLINE];
            v[0..4].copy_from_slice(&(o0 as u32).to_ne_bytes());
            v[4..8].copy_from_slice(&(o1 as u32).to_ne_bytes());
            self.inline16.push(v);
            self.lens.push(LONG);
            code
        }
        #[inline(always)]
        fn make_code(&mut self, nd_word: u32, a: u8) -> u32 {
            if nd_word == 0 {
                return u32::from(a);
            }
            if (nd_word & RULE) == 0 && (nd_word >> 24) < INLINE as u32 {
                let code = self.inline16.len() as u32;
                let wl = (nd_word >> 24) as usize;
                let off = (nd_word & 0xFF_FFFF) as usize;
                let mut v = [0u8; INLINE];
                v.copy_from_slice(&self.arena[off..off + INLINE]);
                v[wl] = a;
                self.inline16.push(v);
                self.lens.push((wl + 1) as u8);
                code
            } else {
                self.make_long_code(nd_word, a)
            }
        }
        fn run(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
            self.strips.clear();
            self.queues.resize_with(256, Vec::new);
            let mut all = [0u32; 256];
            let mut thin = [0u32; 256];
            let mut thick = [0u32; 256];
            let mut ep_cuts = [[0u32; 256]; 16];
            let mut n_ep = 1usize;
            let (mut m, mut mark, mut pos) = (0u32, 0u32, 0u32);
            let mut idx = 0;
            {
                let qs: &mut [Vec<u16>; 256] = (&mut self.queues[..]).try_into().unwrap();
                while idx < records.len() {
                    let c = (records[idx] >> 8) as u8;
                    let mut width = u32::from(records[idx] & 255);
                    idx += 1;
                    while idx < records.len() && (records[idx] >> 8) as u8 == c {
                        width += u32::from(records[idx] & 255);
                        idx += 1;
                    }
                    let a = usize::from(c);
                    if width <= LIMIT {
                        if p.wrapping_sub(pos) < width {
                            mark = m + (p - pos);
                        }
                        if (m >> 16) == ((m + width) >> 16) {
                            push_rows_u16(&mut qs[a], thin[a], m);
                            thin[a] += width;
                            m += width;
                        } else {
                            let mut rem = width;
                            while rem > 0 {
                                let ep = (m >> 16) as usize;
                                let ep_end = ((ep as u32) + 1) << 16;
                                let tk = rem.min(ep_end - m);
                                push_rows_u16(&mut qs[a], thin[a], m);
                                thin[a] += tk;
                                m += tk;
                                rem -= tk;
                                if m == ep_end {
                                    ep_cuts[ep] = thin;
                                    n_ep = ep + 2;
                                }
                            }
                        }
                        all[a] += width;
                    } else if p.wrapping_sub(pos) >= width {
                        self.strips.push(Strip {
                            from: pos,
                            to: all[a],
                            width,
                            byte: c,
                        });
                        thick[a] += 1;
                        all[a] += width;
                    } else {
                        let end = pos + width;
                        if pos < p {
                            let w1 = p - pos;
                            self.strips.push(Strip {
                                from: pos,
                                to: all[a],
                                width: w1,
                                byte: c,
                            });
                            thick[a] += 1;
                            all[a] += w1;
                        }
                        mark = m;
                        push_rows_u16(&mut qs[a], thin[a], m);
                        thin[a] += 1;
                        m += 1;
                        if (m & 0xFFFF) == 0 {
                            let ep = ((m >> 16) - 1) as usize;
                            ep_cuts[ep] = thin;
                            n_ep = ep + 2;
                        }
                        all[a] += 1;
                        if p + 1 < end {
                            let w2 = end - p - 1;
                            self.strips.push(Strip {
                                from: p + 1,
                                to: all[a],
                                width: w2,
                                byte: c,
                            });
                            thick[a] += 1;
                            all[a] += w2;
                        }
                    }
                    pos += width;
                }
            }
            let final_ep = (m >> 16) as usize;
            if n_ep <= final_ep {
                n_ep = final_ep + 1;
            }
            ep_cuts[n_ep - 1] = thin;
            let first = starts(all);
            let mut write = starts(thick);
            let ns = self.strips.len();
            if self.image_order.len() < ns {
                grow_vec_tight(&mut self.image_order, ns, 0u32);
            }
            for (i, s) in self.strips.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                self.image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }
            let slots = PAD + 3 * ns + 4;
            if self.tops.len() < slots {
                grow_vec_tight(&mut self.tops, slots, Top::default());
            }
            if self.bots.len() < slots {
                grow_vec_tight(&mut self.bots, slots, Bot::default());
            }
            if self.strip_top.len() < ns {
                grow_vec_tight(&mut self.strip_top, ns, 0u32);
            }
            self.tops[0] = Top::default();
            self.bots[0] = Bot::default();
            self.rules.clear();
            if self.arena.len() < 256 {
                self.arena.clear();
                self.arena.extend(0..=255u8);
            }
            self.arena.truncate(256);

            let p0_base = PAD as u32;
            let a0_base = p0_base + ns as u32 + 2;
            let (mut t, mut b) = (a0_base, PAD as u32);
            let mut x = 0u32;
            for (i, s) in self.strips.iter().enumerate() {
                if x < s.from {
                    self.tops[t as usize] = Top {
                        prev: t - 1,
                        width: s.from - x,
                        word: 0,
                        other: b,
                    };
                    self.bots[b as usize] = Bot {
                        prev: b - 1,
                        top: t,
                    };
                    t += 1;
                    b += 1;
                }
                self.strip_top[i] = t;
                self.tops[t as usize] = Top {
                    prev: t - 1,
                    width: s.width,
                    word: (1 << 24) | u32::from(s.byte),
                    other: 0,
                };
                t += 1;
                x = s.from + s.width;
            }
            if x < n as u32 {
                self.tops[t as usize] = Top {
                    prev: t - 1,
                    width: n as u32 - x,
                    word: 0,
                    other: b,
                };
                self.bots[b as usize] = Bot {
                    prev: b - 1,
                    top: t,
                };
                t += 1;
                b += 1;
            }
            let a0_end = t;
            let mut p0 = p0_base;
            x = 0;
            for ii in 0..ns {
                let i = self.image_order[ii] as usize;
                let s = self.strips[i];
                if x < s.to {
                    self.tops[p0 as usize] = Top {
                        prev: p0 - 1,
                        width: s.to - x,
                        word: 0,
                        other: b,
                    };
                    self.bots[b as usize] = Bot {
                        prev: b - 1,
                        top: p0,
                    };
                    p0 += 1;
                    b += 1;
                }
                let st = self.strip_top[i];
                self.tops[st as usize].other = b;
                self.bots[b as usize] = Bot {
                    prev: b - 1,
                    top: st,
                };
                b += 1;
                x = s.to + s.width;
            }
            if x < n as u32 {
                self.tops[p0 as usize] = Top {
                    prev: p0 - 1,
                    width: n as u32 - x,
                    word: 0,
                    other: b,
                };
                self.bots[b as usize] = Bot {
                    prev: b - 1,
                    top: p0,
                };
                p0 += 1;
                b += 1;
            }
            if p0 > p0_base {
                self.tops[p0_base as usize].prev = 0;
            }
            self.bots[PAD].prev = 0;
            self.tops[a0_base as usize].prev = if p0 > p0_base { p0 - 1 } else { 0 };

            let (mut sa, mut sb) = (a0_end - 1, b - 1);
            let mut remaining = n as u32;
            while remaining > 0 {
                let ta = self.tops[sa as usize];
                let (mut na, mut ra, mut aw, mut bs) = (ta.prev, ta.width, ta.word, ta.other);
                let bb = self.bots[sb as usize];
                let (mut nb, mut ts) = (bb.prev, bb.top);
                let tb = self.tops[ts as usize];
                let (mut rb, mut bw) = (tb.width, tb.word);
                let (mut fa, mut fb) = (sa, sb);
                if bs == sb {
                    remaining -= ra;
                    (sa, sb) = (na, nb);
                    continue;
                }
                loop {
                    let wd = self.join(aw, bw);
                    if ra == rb {
                        remaining -= ra;
                        let top = &mut self.tops[ts as usize];
                        (top.width, top.word, top.other) = (ra, wd, bs);
                        self.bots[bs as usize].top = ts;
                        (sa, sb) = (na, nb);
                        break;
                    } else if ra > rb {
                        remaining -= rb;
                        let top = &mut self.tops[ts as usize];
                        (top.width, top.word, top.other) = (rb, wd, bs);
                        self.bots[fb as usize].prev = self.bots[bs as usize].prev;
                        self.bots[bs as usize] = Bot { prev: fb, top: ts };
                        bs = fb;
                        ra -= rb;
                        fb = nb;
                        let bn = self.bots[nb as usize];
                        (nb, ts) = (bn.prev, bn.top);
                        let tn = self.tops[ts as usize];
                        (rb, bw) = (tn.width, tn.word);
                    } else {
                        remaining -= ra;
                        self.bots[bs as usize].top = ts;
                        self.tops[fa as usize].prev = self.tops[ts as usize].prev;
                        self.tops[ts as usize] = Top {
                            prev: fa,
                            width: ra,
                            word: wd,
                            other: bs,
                        };
                        ts = fa;
                        rb -= ra;
                        fa = na;
                        let tn = self.tops[na as usize];
                        (na, ra, aw, bs) = (tn.prev, tn.width, tn.word, tn.other);
                    }
                }
            }
            let mut xb = m;
            while sb != 0 {
                let s = self.bots[sb as usize];
                xb -= self.tops[s.top as usize].width;
                self.tops[s.top as usize].other = xb;
                sb = s.prev;
            }
            self.arena.extend_from_slice(&[0u8; INLINE]);
            if self.inline16.len() < 256 {
                self.inline16.clear();
                self.inline16.extend((0..=255u8).map(|c| {
                    let mut v = [0; INLINE];
                    v[0] = c;
                    v
                }));
                self.lens.clear();
                self.lens.resize(256, 1);
            }
            self.inline16.truncate(256);
            self.lens.truncate(256);
            self.spelling.clear();
            self.bands_rev.clear();
            let thin_start = starts(thin);
            let mut rev_start = [0usize; 256];
            let mut rem = if sa != 0 {
                self.tops[sa as usize].width
            } else {
                0
            };
            let mut kc = m;
            for a in (0..256).rev() {
                rev_start[a] = self.bands_rev.len();
                let start = thin_start[a];
                while kc > start {
                    let nd = self.tops[sa as usize];
                    let width = (kc - start).min(rem);
                    rem -= width;
                    kc -= width;
                    let dst = nd.other + rem;
                    let code = self.make_code(nd.word, a as u8);
                    self.bands_rev.push(BandRev {
                        q_start: kc - start,
                        dst,
                        code,
                    });
                    if rem == 0 {
                        sa = nd.prev;
                        rem = if sa != 0 {
                            self.tops[sa as usize].width
                        } else {
                            0
                        };
                    }
                }
            }
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let wide = ((self.inline16.len() - 1) as u32) > (u32::MAX >> bits);
            if self.edges.len() < m as usize {
                grow_vec_tight(&mut self.edges, m as usize, 0u32);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }
            for a in (0..256).rev() {
                let b0 = rev_start[a];
                let b1 = if a > 0 {
                    rev_start[a - 1]
                } else {
                    self.bands_rev.len()
                };
                if b0 == b1 {
                    continue;
                }
                let qa = &self.queues[a];
                let mut prev_q = thin[a] as usize;
                let mut ep = n_ep - 1;
                for bnd in &self.bands_rev[b0..b1] {
                    let q0 = bnd.q_start as usize;
                    let span = prev_q - q0;
                    let q1 = prev_q;
                    prev_q = q0;
                    let dst = bnd.dst as usize;
                    if wide {
                        self.extra[dst..dst + span].fill(bnd.code);
                    }
                    let cpack = if wide { 0 } else { bnd.code << bits };
                    while ep > 0 && q1 <= ep_cuts[ep - 1][a] as usize {
                        ep -= 1;
                    }
                    let ep_lo = if ep > 0 { ep_cuts[ep - 1][a] as usize } else { 0 };
                    if q0 >= ep_lo {
                        let pack = cpack | ((ep as u32) << 16);
                        for (ed, &lo16) in self.edges[dst..dst + span].iter_mut().zip(&qa[q0..q1]) {
                            *ed = pack | u32::from(lo16);
                        }
                    } else {
                        let mut cur_q1 = q1;
                        let mut cur_dst_end = dst + span;
                        while cur_q1 > q0 {
                            while ep > 0 && cur_q1 <= ep_cuts[ep - 1][a] as usize {
                                ep -= 1;
                            }
                            let lo = if ep > 0 {
                                (ep_cuts[ep - 1][a] as usize).max(q0)
                            } else {
                                q0
                            };
                            let tk = cur_q1 - lo;
                            let pack = cpack | ((ep as u32) << 16);
                            for (ed, &lo16) in self.edges[cur_dst_end - tk..cur_dst_end]
                                .iter_mut()
                                .zip(&qa[lo..cur_q1])
                            {
                                *ed = pack | u32::from(lo16);
                            }
                            cur_q1 = lo;
                            cur_dst_end -= tk;
                        }
                    }
                }
            }
            out.resize(n + INLINE, 0);
            let mut at = 0usize;
            let mask = (1u32 << bits) - 1;
            let mut j = mark;
            for _ in 0..m as usize {
                let edge = self.edges[j as usize];
                let code = if self.extra.is_empty() {
                    (edge >> bits) as usize
                } else {
                    self.extra[j as usize] as usize
                };
                j = edge & mask;
                let len = self.lens[code];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&self.inline16[code]);
                    at += usize::from(len);
                } else {
                    let meta = self.inline16[code];
                    let o0 = u32::from_ne_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let o1 = u32::from_ne_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &self.spelling[o0..o1];
                    let take = word.len().min(n - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    at += take;
                }
                if j == mark || at >= n {
                    break;
                }
            }
            out.truncate(at.min(n));
            let mut len = out.len();
            while len < n {
                let take = len.min(n - len);
                out.extend_from_within(..take);
                len = out.len();
            }
        }
    }

    impl InverseBwt for C15OpenCells {
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
            let nr = r.records().len();
            if n > 0 && (p as usize) < n && nr * 20 < n * 3 && nr * 64 >= n {
                self.run(r.records(), n, p, out);
                return true;
            }
            self.lazy.invert_runs(r, n, p, out, k)
        }
        fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
            self.lazy.invert(l, c, p, out);
        }
        fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
            self.lazy.invert_in_place(l, c, p)
        }
    }
}

struct AdmittedBlk {
    runs: Runs,
    n: usize,
    p: u32,
}

fn load_admitted_from_bz2(bz2_path: &str) -> Vec<AdmittedBlk> {
    let data = std::fs::read(bz2_path).expect("read bz2");
    let mut sc = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut syms = Vec::new();
    let mut out = Vec::new();
    while let Ok(Event::Block { symbols: b, .. }) = sc.next(std::mem::take(&mut syms)) {
        if runs::takes_runs(&b) {
            let mut r = Runs::default();
            let n = runs::front(&b, &mut r, k).unwrap();
            let nr = r.records().len();
            if nr * 20 < n * 3 && nr * 64 >= n {
                out.push(AdmittedBlk {
                    runs: r,
                    n,
                    p: b.orig_ptr,
                });
            }
        }
        syms = b.syms;
    }
    out
}

fn run_cg_kernel_only<K: InverseBwt>(sample_bz2: &str) {
    let blks = load_admitted_from_bz2(sample_bz2);
    let k = Kernels::detect();
    let mut kern = K::default();
    let mut out = Vec::with_capacity(900_032);
    for b in blks.iter().skip(1) {
        kern.invert_runs(&b.runs, b.n, b.p, &mut out, k);
    }
    client_request(0, &[REQ_WARM, 0, 0, 0, 0, 0]);
    kern.invert_runs(&blks[0].runs, blks[0].n, blks[0].p, &mut out, k);
    let mut total = 0u64;
    client_request(0, &[REQ_BEGIN, 0, 0, 0, 0, 0]);
    for b in blks.iter().skip(1) {
        kern.invert_runs(&b.runs, b.n, b.p, &mut out, k);
        std::hint::black_box(out.as_ptr());
        total += b.n as u64;
    }
    client_request(0, &[REQ_END, total, 0, 0, 0, 0]);
}

fn run_adversarial_tests(blks: &[AdmittedBlk], bz2_path: &str) {
    let k = Kernels::detect();
    let mut ref_k = OpenCells::default();
    let mut ref_out = Vec::with_capacity(900_032);
    let mut cand = candidate_c16::SwarOutQ16Kernel::default();
    let mut out = Vec::with_capacity(900_032);

    // 1. Poisoned-`out` + poisoned-tape + multi-epoch (`LIMIT=64`) tests on 24 real blocks
    for (i, b) in blks.iter().enumerate().take(24) {
        assert!(ref_k.invert_runs(&b.runs, b.n, b.p, &mut ref_out, k));
        out.resize(b.n + 64, 0xA5);
        out.fill(0xA5);
        cand.poison_tapes_for_test();
        cand.run(b.runs.records(), b.n, b.p, 16, &mut out);
        assert_eq!(out, ref_out, "poisoned out/tape mismatch on block {i}");
        assert_eq!(cand.last_ovf_chunks_used, 0, "expected 0 ovf_chunks on full block {i}");

        out.fill(0x5A);
        cand.poison_tapes_for_test();
        cand.run(b.runs.records(), b.n, b.p, 64, &mut out);
        assert_eq!(out, ref_out, "multi-epoch LIMIT=64 mismatch on block {i}");
    }

    // 2. Tiny-block `ovf_chunks` spillover test (`n < 1024`, `max_out_chunks == 0`, multi-symbol!)
    let mut tiny_recs = Vec::new();
    for step in 0..300u16 {
        let sym = (step % 17) as u16;
        tiny_recs.push((sym << 8) | 1);
    }
    out.clear();
    cand.poison_tapes_for_test();
    cand.run(&tiny_recs, 300, 137, 16, &mut out);
    assert!(cand.last_ovf_chunks_used >= 17, "expected ovf_chunks spillover on tiny n=300");
    assert_eq!(out.len(), 300);

    // 3. Full 431-block `wikidata.bz2` parallel decode with CRC check at W=8 and W=10
    let bz2_bytes = std::fs::read(bz2_path).expect("read wikidata.bz2");
    for &w in &[8usize, 10usize] {
        let mut rdr = mibz2::BzReader::parallel_workers_with::<candidate_c16::SwarOutQ16OpenCells>(
            std::io::Cursor::new(bz2_bytes.clone()),
            w,
            3 * w,
            k,
        );
        let mut total = 0usize;
        let mut buf = vec![0u8; 256 * 1024];
        loop {
            let got = rdr.read(&mut buf).expect("CRC-verified parallel decode failed");
            if got == 0 {
                break;
            }
            total += got;
        }
        assert_eq!(total, 209_700_196);
    }
}

fn verify_and_mem<K: InverseBwt>(blks: &[AdmittedBlk], check: bool) -> (usize, usize) {
    let k = Kernels::detect();
    let mut ref_k = OpenCells::default();
    let mut ref_out = Vec::with_capacity(900_032);
    let mut out = Vec::with_capacity(900_032);
    if check {
        let mut test_k = K::default();
        for (i, b) in blks.iter().enumerate() {
            assert!(ref_k.invert_runs(&b.runs, b.n, b.p, &mut ref_out, k));
            assert!(test_k.invert_runs(&b.runs, b.n, b.p, &mut out, k));
            assert_eq!(out, ref_out, "mismatch on block {i}");
        }
    }
    tls_reset();
    let mut kern = K::default();
    for b in blks {
        assert!(kern.invert_runs(&b.runs, b.n, b.p, &mut out, k));
    }
    let s = tls_stats();
    drop(kern);
    s
}

fn time_workers<K: InverseBwt + Send + 'static>(blks: &[AdmittedBlk], workers: usize) -> (f64, f64) {
    let idx = AtomicUsize::new(0);
    let k = Kernels::detect();
    let cpu_ns = AtomicUsize::new(0);
    let t0 = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..workers {
            let idx_r = &idx;
            let cpu_r = &cpu_ns;
            s.spawn(move || {
                let mut kern = K::default();
                let mut out = Vec::with_capacity(900_032);
                let mut my = 0usize;
                loop {
                    let i = idx_r.fetch_add(1, Ordering::Relaxed);
                    if i >= blks.len() {
                        break;
                    }
                    let b = &blks[i];
                    let ts = Instant::now();
                    let ok = kern.invert_runs(&b.runs, b.n, b.p, &mut out, k);
                    std::hint::black_box(ok);
                    std::hint::black_box(out.as_ptr());
                    my += ts.elapsed().as_nanos() as usize;
                }
                cpu_r.fetch_add(my, Ordering::Relaxed);
            });
        }
    });
    (
        t0.elapsed().as_secs_f64() * 1e3,
        cpu_ns.load(Ordering::Relaxed) as f64 / 1e6,
    )
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map_or("bench", String::as_str);
    let sample_bz2 = args.get(2).map_or("/tmp/sample9_admitted.bz2", String::as_str);
    if let Some(arm) = mode.strip_prefix("cg:") {
        match arm {
            "058" => run_cg_kernel_only::<OpenCells>(sample_bz2),
            "c15" => run_cg_kernel_only::<prior_c15::C15OpenCells>(sample_bz2),
            "c16" => run_cg_kernel_only::<candidate_c16::SwarOutQ16OpenCells>(sample_bz2),
            _ => panic!("unknown arm {arm}"),
        }
        return;
    }

    let bz2_path = args.get(2).map_or("/tmp/wikidata.bz2", String::as_str);
    let blks = load_admitted_from_bz2(bz2_path);
    assert_eq!(blks.len(), 228);

    println!("Running G5.1 adversarial dirty-out, dirty-tape, tiny-ovf, multi-epoch, 228-block & 431-block BzReader W=8,10 checks...");
    run_adversarial_tests(&blks, bz2_path);
    println!("  G5.1 Exactness & #[forbid(unsafe_code)] Verification: PASS");

    let names = [
        "0: Master-0.5.8 (OpenCells)",
        "1: C15-EpochQ16-ZeroClearR4",
        "2: C16-SwarOutQ16-CompactR4",
    ];
    let mems = [
        verify_and_mem::<OpenCells>(&blks, false),
        verify_and_mem::<prior_c15::C15OpenCells>(&blks, true),
        verify_and_mem::<candidate_c16::SwarOutQ16OpenCells>(&blks, true),
    ];

    let mut l3_c16 = Vec::with_capacity(blks.len());
    let mut total_ovf = 0usize;
    {
        let mut kc16 = candidate_c16::SwarOutQ16Kernel::default();
        let mut o = Vec::with_capacity(900_032);
        for b in &blks {
            kc16.run(b.runs.records(), b.n, b.p, 16, &mut o);
            l3_c16.push(kc16.last_unique_l3_bytes as f64 / 1024.0);
            total_ovf += kc16.last_ovf_chunks_used;
        }
    }
    l3_c16.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean_l3 = l3_c16.iter().sum::<f64>() / l3_c16.len() as f64;
    let p95_l3 = l3_c16[(l3_c16.len() * 95) / 100];

    println!("\n=== G5.2 Per-Worker Kernel Heap & Unique L3 Footprint (228 Admitted Blocks) ===");
    let (b_ret, b_pk) = mems[0];
    let (c15_ret, _) = mems[1];
    for i in 0..3 {
        println!(
            "  {:<32} | Retained = {:7.1} KiB ({:5.3}x 0.5.8, {:5.3}x C15) | Peak = {:7.1} KiB ({:5.3}x 0.5.8)",
            names[i],
            mems[i].0 as f64 / 1024.0,
            mems[i].0 as f64 / b_ret as f64,
            mems[i].0 as f64 / c15_ret as f64,
            mems[i].1 as f64 / 1024.0,
            mems[i].1 as f64 / b_pk as f64
        );
    }
    println!(
        "  C16 Unique Per-Block L3 Footprint (incl. 900KB out): mean = {:6.1} KiB (<= 2048 KiB W=10 LLC!), p95 = {:6.1} KiB | ovf_chunks = {}",
        mean_l3, p95_l3, total_ovf
    );
    let g52_pass = (mems[2].0 as f64 / b_ret as f64) <= 0.42
        && (mems[2].1 as f64 / b_pk as f64) <= 0.42
        && (mems[2].0 as f64 / c15_ret as f64) <= 0.75
        && mean_l3 <= 2048.0
        && total_ovf == 0;
    println!("  G5.2 Memory Reduction Gate: {}", if g52_pass { "PASS" } else { "FAIL" });

    let rounds = 15usize;
    let mut g54_pass = true;
    for &w in &[1usize, 2, 8, 10] {
        let mut wall = vec![vec![0.0; rounds]; 3];
        let mut cpu = vec![vec![0.0; rounds]; 3];
        for r in 0..rounds + 1 {
            for step in 0..3 {
                let arm = (step + r) % 3;
                let (wm, cm) = match arm {
                    0 => time_workers::<OpenCells>(&blks, w),
                    1 => time_workers::<prior_c15::C15OpenCells>(&blks, w),
                    2 => time_workers::<candidate_c16::SwarOutQ16OpenCells>(&blks, w),
                    _ => unreachable!(),
                };
                if r >= 1 {
                    wall[arm][r - 1] = wm;
                    cpu[arm][r - 1] = cm;
                }
            }
        }
        let bw = median(&mut wall[0].clone());
        let bc = median(&mut cpu[0].clone());
        println!("\n=== G5.4 Paired Native Rust Timing (`lto=fat`) at W = {w} Workers ({rounds} rounds) ===");
        for arm in 0..3 {
            let mw = median(&mut wall[arm]);
            let mc = median(&mut cpu[arm]);
            println!(
                "  {:<32} | wall = {:6.1} ms ({:5.3}x 0.5.8) | sum_cpu = {:7.1} ms ({:5.3}x 0.5.8)",
                names[arm],
                mw,
                mw / bw,
                mc,
                mc / bc
            );
            if arm == 2 && (mw / bw > 0.935 || mc / bc > 0.935) {
                g54_pass = false;
            }
        }
    }
    println!("\n  G5.4 Multi-Worker Native Speed Gate: {}", if g54_pass { "PASS" } else { "FAIL" });
}
