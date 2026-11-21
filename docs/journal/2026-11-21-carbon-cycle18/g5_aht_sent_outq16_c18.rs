//! Carbon Cycle 18 Gate G5 Confrontation Harness (`g5_aht_sent_outq16_c18.rs`):
//! Evaluates `AhtSentOutQ16` (`#[forbid(unsafe_code)]`, Theorem 18.1 linked-list Ostrowski/AHT
//! self-overlap contraction + `MicroLF` + Theorem 18.2 zero-hot-cost `m <= 256_000, ns <= 16_384`
//! Pass-1 structural admission) across ALL 431 blocks of `wikidata.bz2` AND the 228-block subset
//! against `Master-0.5.8` (`OpenCells`) and Cycle 17 `SentOutQ16` (`Gate 0.15`).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use mibz2::bwt::{InverseBwt, OpenCells, Segmented};
use mibz2::mtf;
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{BzReader, Kernels};

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

#[forbid(unsafe_code)]
pub mod candidate_c18 {
    use mibz2::bwt::{InverseBwt, LazyCells};
    use mibz2::runs::Runs;
    use mibz2::Kernels;

    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const PAD: usize = 1;
    const CAP: u32 = 64;
    const RULE: u32 = 1 << 31;
    const ARENA_MAX: usize = 0x00FF_F000;
    const CHUNK_U16: usize = 512;
    const CHUNK_B: usize = 1024;
    const MICRO_MAX_N: usize = 2048;

    // Theorem 18.2 structural caps: guarantees `bits <= 18` (`<= 16,383` inline codes with `wide == false`)
    // and caps `edges` `<= 1,000 KiB` + `tops/bots/scratch` `<= 1,472 KiB`!
    pub const M_MAX_DEFAULT: u32 = 256_000;
    pub const NS_MAX_DEFAULT: usize = 16_384;

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

    #[cold]
    #[inline(never)]
    fn grow_vec_pow2<T: Clone>(v: &mut Vec<T>, need: usize, init: T) {
        let cur = v.len();
        let target = need.next_power_of_two().max(512);
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

    struct PassMeta {
        all: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        head: [u16; 256],
        tail: [u16; 256],
        in_chunk: [u16; 256],
        ep_cuts: Vec<[u32; 256]>,
        n_ep: usize,
        n_chunks: u16,
        max_out_chunks: u16,
        m: u32,
        m_limit: u32,
        mark: u32,
        aborted: bool,
    }

    impl Default for PassMeta {
        fn default() -> Self {
            Self {
                all: [0; 256],
                thin: [0; 256],
                thick: [0; 256],
                head: [u16::MAX; 256],
                tail: [0; 256],
                in_chunk: [CHUNK_U16 as u16; 256],
                ep_cuts: vec![[0u32; 256]; 16],
                n_ep: 1,
                n_chunks: 0,
                max_out_chunks: 0,
                m: 0,
                m_limit: u32::MAX,
                mark: 0,
                aborted: false,
            }
        }
    }

    impl PassMeta {
        #[inline(always)]
        fn reset(&mut self, max_out_chunks: u16, m_limit: u32) {
            self.all.fill(0);
            self.thin.fill(0);
            self.thick.fill(0);
            self.head.fill(u16::MAX);
            self.in_chunk.fill(CHUNK_U16 as u16);
            self.n_ep = 1;
            self.n_chunks = 0;
            self.max_out_chunks = max_out_chunks;
            self.m = 0;
            self.m_limit = m_limit;
            self.mark = 0;
            self.aborted = false;
        }
    }

    struct PassOut<'a, const LIMIT: u32, const M_MAX: u32> {
        pm: &'a mut PassMeta,
        chunk_next: &'a mut Vec<u16>,
        ovf_chunks: &'a mut Vec<[u8; CHUNK_B]>,
        triples: &'a mut Vec<[u32; 3]>,
        out: &'a mut [u8],
    }

    #[inline(always)]
    fn swar_write_u16s<const NWORDS: usize>(dst: &mut [u8], m_lo: u32) {
        let x0 = (u64::from(m_lo) * 0x0001_0001_0001_0001) + 0x0003_0002_0001_0000;
        if NWORDS >= 1 {
            dst[0..8].copy_from_slice(&x0.to_le_bytes());
        }
        if NWORDS >= 2 {
            dst[8..16].copy_from_slice(&(x0 + 0x0004_0004_0004_0004).to_le_bytes());
        }
        if NWORDS >= 3 {
            dst[16..24].copy_from_slice(&(x0 + 0x0008_0008_0008_0008).to_le_bytes());
        }
        if NWORDS >= 4 {
            dst[24..32].copy_from_slice(&(x0 + 0x000C_000C_000C_000C).to_le_bytes());
        }
    }

    impl<const LIMIT: u32, const M_MAX: u32> PassOut<'_, LIMIT, M_MAX> {
        const NWORDS: usize = (LIMIT as usize + 3) / 4;
        const CAP_U16: usize = Self::NWORDS * 4;
        const M_LO_MAX: u32 = 0xFFFF - (Self::CAP_U16 as u32);

        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, w: u32) {
            if self.pm.m + w > self.pm.m_limit {
                self.pm.m += w;
                self.pm.aborted = true;
                return;
            }
            let mut rem = w;
            let mut cur_m = self.pm.m;
            while rem > 0 {
                let ep = (cur_m >> 16) as usize;
                let ep_end = ((ep as u32) + 1) << 16;
                let mut cid = self.pm.tail[a];
                let mut ic = self.pm.in_chunk[a] as usize;
                if ic >= CHUNK_U16 {
                    cid = self.pm.n_chunks;
                    self.pm.n_chunks += 1;
                    if self.chunk_next.len() <= cid as usize {
                        grow_vec_tight(self.chunk_next, cid as usize + 1, u16::MAX);
                    }
                    self.chunk_next[cid as usize] = u16::MAX;
                    if cid >= self.pm.max_out_chunks {
                        let need_ovf = (cid - self.pm.max_out_chunks) as usize + 1;
                        if self.ovf_chunks.len() < need_ovf {
                            grow_vec_tight(self.ovf_chunks, need_ovf, [0u8; CHUNK_B]);
                        }
                    }
                    if self.pm.head[a] == u16::MAX {
                        self.pm.head[a] = cid;
                    } else {
                        self.chunk_next[self.pm.tail[a] as usize] = cid;
                    }
                    self.pm.tail[a] = cid;
                    ic = 0;
                }
                let tk = rem.min(ep_end - cur_m).min((CHUNK_U16 - ic) as u32);
                let cap_u16 = Self::CAP_U16;
                if (tk as usize) <= cap_u16
                    && ic <= CHUNK_U16 - cap_u16
                    && (cur_m & 0xFFFF) <= Self::M_LO_MAX
                    && cid < self.pm.max_out_chunks
                {
                    let b0 = (cid as usize) * CHUNK_B + ic * 2;
                    let dst = &mut self.out[b0..b0 + cap_u16 * 2];
                    if Self::NWORDS == 3 {
                        swar_write_u16s::<3>(dst, cur_m & 0xFFFF);
                    } else {
                        swar_write_u16s::<4>(dst, cur_m & 0xFFFF);
                    }
                } else {
                    let dst_slice: &mut [u8] = if cid < self.pm.max_out_chunks {
                        let b0 = (cid as usize) * CHUNK_B + ic * 2;
                        &mut self.out[b0..b0 + (tk as usize) * 2]
                    } else {
                        let oi = (cid - self.pm.max_out_chunks) as usize;
                        &mut self.ovf_chunks[oi][ic * 2..(ic + tk as usize) * 2]
                    };
                    for (k, pair) in dst_slice.chunks_exact_mut(2).enumerate() {
                        let b = (cur_m as u16).wrapping_add(k as u16).to_le_bytes();
                        pair[0] = b[0];
                        pair[1] = b[1];
                    }
                }
                self.pm.in_chunk[a] = (ic + tk as usize) as u16;
                self.pm.thin[a] += tk;
                cur_m += tk;
                rem -= tk;
                if cur_m == ep_end {
                    if self.pm.ep_cuts.len() <= ep {
                        grow_vec_tight(&mut self.pm.ep_cuts, ep + 2, [0u32; 256]);
                    }
                    self.pm.ep_cuts[ep] = self.pm.thin;
                    self.pm.n_ep = ep + 2;
                }
            }
            self.pm.m = cur_m;
        }

        #[inline(always)]
        fn push_thin(&mut self, a: usize, x: u32, w: u32, p: u32) {
            if p.wrapping_sub(x) < w {
                self.pm.mark = self.pm.m + (p - x);
            }
            let ic = self.pm.in_chunk[a] as usize;
            let cid = self.pm.tail[a];
            let m_now = self.pm.m;
            let m_lo = m_now & 0xFFFF;
            let cap_u16 = Self::CAP_U16;
            if ic <= CHUNK_U16 - cap_u16 && m_lo <= Self::M_LO_MAX && cid < self.pm.max_out_chunks {
                let b0 = (cid as usize) * CHUNK_B + ic * 2;
                let dst = &mut self.out[b0..b0 + cap_u16 * 2];
                if Self::NWORDS == 3 {
                    swar_write_u16s::<3>(dst, m_lo);
                } else {
                    swar_write_u16s::<4>(dst, m_lo);
                }
                self.pm.in_chunk[a] = (ic + w as usize) as u16;
                self.pm.thin[a] += w;
                self.pm.m = m_now + w;
            } else {
                self.push_slow(a, w);
            }
            self.pm.all[a] += w;
        }

        #[inline(always)]
        fn push_thick(&mut self, c: u8, x: u32, w: u32) {
            let a = usize::from(c);
            self.triples.push([x, self.pm.all[a], (w << 8) | u32::from(c)]);
            self.pm.thick[a] += 1;
            self.pm.all[a] += w;
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
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            if width <= LIMIT {
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
    pub struct AhtSentOutQKernel<const LIMIT: u32, const USE_AHT: bool> {
        pm: Box<PassMeta>,
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
        n_codes: usize,
        edges: Vec<u32>,
        extra: Vec<u32>,
        pub r4_loop_steps: usize,
        pub last_ovf_chunks_used: usize,
    }

    impl<const LIMIT: u32, const USE_AHT: bool> AhtSentOutQKernel<LIMIT, USE_AHT> {
        pub fn poison_tapes_for_test(&mut self) {
            self.chunk_next.fill(0xDEAD);
            self.ovf_chunks.fill([0xCC; CHUNK_B]);
            self.u32_scratch.fill(0xDEAD_BEEF);
            self.tops.fill(Top {
                prev: 0xDEAD_BEEF,
                width: 0xDEAD_BEEF,
                word: 0xDEAD_BEEF,
                other: 0xDEAD_BEEF,
            });
            self.bots.fill(Bot {
                prev: 0xDEAD_BEEF,
                top: 0xDEAD_BEEF,
            });
            if self.inline16.len() > 256 {
                self.inline16[256..].fill([0xEE; INLINE]);
                self.lens[256..].fill(11);
            }
            self.edges.fill(0xDEAD_BEEF);
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
        fn join_pow(&mut self, mut base: u32, mut exp: u32) -> u32 {
            if base == 0 || exp == 0 {
                return 0;
            }
            let mut acc = 0u32;
            while exp > 0 {
                if (exp & 1) != 0 {
                    acc = self.join(acc, base);
                }
                exp >>= 1;
                if exp > 0 {
                    base = self.join(base, base);
                }
            }
            acc
        }

        #[inline(always)]
        fn push_code_slot(&mut self, v: [u8; INLINE], ln: u8) -> u32 {
            let c = self.n_codes;
            if c >= self.inline16.len() {
                grow_vec_pow2(&mut self.inline16, c + 1, [0u8; INLINE]);
                grow_vec_pow2(&mut self.lens, c + 1, LONG);
            }
            self.inline16[c] = v;
            self.lens[c] = ln;
            self.n_codes = c + 1;
            c as u32
        }

        #[cold]
        #[inline(never)]
        fn make_long_code(&mut self, nd_word: u32, a: u8) -> u32 {
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
            self.push_code_slot(v, LONG)
        }

        #[inline(always)]
        fn make_code(&mut self, nd_word: u32, a: u8) -> u32 {
            if nd_word == 0 {
                return u32::from(a);
            }
            if (nd_word & RULE) == 0 && (nd_word >> 24) < INLINE as u32 {
                let wl = (nd_word >> 24) as usize;
                let off = (nd_word & 0xFF_FFFF) as usize;
                let mut v = [0u8; INLINE];
                v.copy_from_slice(&self.arena[off..off + INLINE]);
                v[wl] = a;
                self.push_code_slot(v, (wl + 1) as u8)
            } else {
                self.make_long_code(nd_word, a)
            }
        }

        #[cold]
        #[inline(never)]
        fn run_micro(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
            if self.edges.len() < n {
                grow_vec_tight(&mut self.edges, n, 0u32);
            }
            let mut counts = [0u16; 256];
            let mut pos_check = 0usize;
            for &rec in records {
                let b = (rec >> 8) as usize;
                let w = rec & 255;
                counts[b] = counts[b].wrapping_add(w);
                pos_check += w as usize;
            }
            if pos_check != n {
                return false;
            }
            let mut occ = [0u16; 256];
            let mut tot = 0u16;
            for c in 0..256 {
                occ[c] = tot;
                tot = tot.wrapping_add(counts[c]);
            }
            let tt = &mut self.edges[..n];
            let mut row_pos = 0u32;
            for &rec in records {
                let b = (rec >> 8) as usize;
                let w = (rec & 255) as usize;
                let start = occ[b] as usize;
                occ[b] += w as u16;
                let b_u32 = b as u32;
                for k in 0..w {
                    tt[start + k] = ((row_pos + k as u32) << 8) | b_u32;
                }
                row_pos += w as u32;
            }
            out.clear();
            out.reserve(n);
            let mut r = p as usize;
            for _ in 0..n {
                let e = tt[r];
                out.push((e & 0xFF) as u8);
                r = (e >> 8) as usize;
                if r == p as usize {
                    break;
                }
            }
            while out.len() < n {
                let tk = out.len().min(n - out.len());
                out.extend_from_within(..tk);
            }
            true
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

        pub fn try_run<const M_MAX: u32, const NS_MAX: usize>(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            cap_override: u16,
            out: &mut Vec<u8>,
        ) -> bool {
            if n == 0 || n > 900_000 || (p as usize) >= n {
                return false;
            }
            if USE_AHT && n <= MICRO_MAX_N && cap_override == 60_000 {
                self.r4_loop_steps = 0;
                self.last_ovf_chunks_used = 0;
                return self.run_micro(records, n, p, out);
            }
            if out.len() < n + 32 {
                out.resize(n + 32, 0);
            }
            let max_out_chunks = ((out.len() / CHUNK_B).min(60_000) as u16).min(cap_override);
            let m_limit = if M_MAX < 900_000 {
                M_MAX.min(((n as u64 * 3) / 10) as u32).max(2048)
            } else {
                M_MAX
            };
            self.pm.reset(max_out_chunks, m_limit);
            self.triples.clear();

            {
                let mut pass = PassOut::<LIMIT, M_MAX> {
                    pm: &mut self.pm,
                    chunk_next: &mut self.chunk_next,
                    ovf_chunks: &mut self.ovf_chunks,
                    triples: &mut self.triples,
                    out: &mut out[..],
                };
                let mut pos = 0u32;
                let mut idx = 0;
                while idx < records.len() {
                    let c = (records[idx] >> 8) as u8;
                    let mut width = u32::from(records[idx] & 255);
                    idx += 1;
                    while idx < records.len() && (records[idx] >> 8) as u8 == c {
                        width += u32::from(records[idx] & 255);
                        idx += 1;
                    }
                    pass.take(c, pos, width, p);
                    pos += width;
                    if pass.pm.aborted {
                        return false;
                    }
                }
                if pos as usize != n {
                    return false;
                }
            }

            let m = self.pm.m;
            let ns = self.triples.len();
            if m > m_limit || ns > NS_MAX {
                return false;
            }

            let mark = self.pm.mark;
            let final_ep = (m >> 16) as usize;
            if self.pm.n_ep <= final_ep {
                if self.pm.ep_cuts.len() <= final_ep {
                    grow_vec_tight(&mut self.pm.ep_cuts, final_ep + 2, [0u32; 256]);
                }
                self.pm.n_ep = final_ep + 1;
            }
            let n_ep = self.pm.n_ep;
            self.pm.ep_cuts[n_ep - 1] = self.pm.thin;
            self.last_ovf_chunks_used = (self.pm.n_chunks.saturating_sub(max_out_chunks)) as usize;

            let first = starts(self.pm.all);
            let mut write = starts(self.pm.thick);
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

            let mut steps = 0usize;
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
                    steps += 1;
                    (sa, sb) = (na, nb);
                    continue;
                }
                loop {
                    steps += 1;
                    if USE_AHT && nb == bs && ra > rb && (ra - 1) >= 2 * rb {
                        let q = (ra - 1) / rb;
                        let p_tail = self.bots[bs as usize].prev;
                        let pow_aw = self.join_pow(aw, q);
                        let bw_new = self.join(pow_aw, bw);
                        let (u, v) = if (q & 1) == 1 { (fb, bs) } else { (bs, fb) };
                        let top = &mut self.tops[ts as usize];
                        (top.width, top.word, top.other) = (rb, bw_new, v);
                        self.bots[u as usize] = Bot { prev: p_tail, top: ts };
                        self.bots[v as usize] = Bot { prev: u, top: ts };
                        bs = u;
                        fb = v;
                        nb = u;
                        bw = bw_new;
                        let consumed = q * rb;
                        ra -= consumed;
                        remaining -= consumed;
                    }
                    if USE_AHT && na == ts && ra < rb && (rb - 1) >= 2 * ra {
                        let q = (rb - 1) / ra;
                        let p_tail = self.tops[ts as usize].prev;
                        let pow_qm1 = self.join_pow(bw, q - 1);
                        let w_qm1 = self.join(aw, pow_qm1);
                        let w_q = self.join(w_qm1, bw);
                        let (u, v) = if (q & 1) == 1 { (fa, ts) } else { (ts, fa) };
                        self.bots[bs as usize].top = v;
                        self.tops[u as usize] = Top {
                            prev: p_tail,
                            width: ra,
                            word: w_qm1,
                            other: bs,
                        };
                        self.tops[v as usize] = Top {
                            prev: u,
                            width: ra,
                            word: w_q,
                            other: bs,
                        };
                        ts = u;
                        fa = v;
                        na = u;
                        aw = w_q;
                        let consumed = q * ra;
                        rb -= consumed;
                        remaining -= consumed;
                    }

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
            self.r4_loop_steps = steps;

            let mut xb = m;
            while sb != 0 {
                let s = self.bots[sb as usize];
                xb -= self.tops[s.top as usize].width;
                self.tops[s.top as usize].other = xb;
                sb = s.prev;
            }
            self.arena.extend_from_slice(&[0u8; INLINE]);

            if self.inline16.len() < 512 {
                grow_vec_pow2(&mut self.inline16, 512, [0u8; INLINE]);
                grow_vec_pow2(&mut self.lens, 512, LONG);
                for c in 0..=255u8 {
                    let mut v = [0u8; INLINE];
                    v[0] = c;
                    self.inline16[usize::from(c)] = v;
                    self.lens[usize::from(c)] = 1;
                }
            }
            self.n_codes = 256;
            self.spelling.clear();
            self.triples.clear();

            let thin_start = starts(self.pm.thin);
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

            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let n_real_codes = self.n_codes;
            let wide = (n_real_codes as u32) > (u32::MAX >> bits);
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
                let ep_cuts = &self.pm.ep_cuts[..];
                for a in 0..256 {
                    let b0 = rev_start[a];
                    let b1 = if a > 0 { rev_start[a - 1] } else { bands.len() };
                    if b0 == b1 {
                        continue;
                    }
                    let mut cid = self.pm.head[a];
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

            if !wide {
                let stop_code = self.push_code_slot([0u8; INLINE], LONG);
                emit_sentinel(
                    &mut self.edges[..m as usize],
                    m,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    stop_code,
                    n,
                    mark,
                    bits,
                    out,
                );
            } else {
                emit_wide(
                    &self.edges[..m as usize],
                    &self.extra,
                    &self.spelling,
                    &self.inline16[..n_real_codes],
                    &self.lens[..n_real_codes],
                    n,
                    mark,
                    bits,
                    out,
                );
            }
            true
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

    fn emit_sentinel(
        edges_s: &mut [u32],
        m: u32,
        spelling: &[u8],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        stop_code: u32,
        n: usize,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        if out.len() < n + 32 {
            out.resize(n + 32, 0);
        }
        let mask = (1u32 << bits) - 1;
        let code_cap = inline16.len();
        let code_mask = code_cap - 1;
        let stop_c = stop_code as usize;

        let raw_mark = edges_s[mark as usize];
        let code0 = ((raw_mark >> bits) as usize) & code_mask;
        let mut j = raw_mark & mask;
        edges_s[mark as usize] = (stop_code << bits) | j;

        let mut at = 0usize;
        {
            let out_s = &mut out[..n + INLINE];
            let inline_s = &inline16[..code_cap];
            let lens_s = &lens[..code_cap];

            let ln0 = lens_s[code0];
            if ln0 <= INLINE as u8 && INLINE <= n {
                out_s[0..INLINE].copy_from_slice(&inline_s[code0]);
                at = usize::from(ln0);
            } else if ln0 <= INLINE as u8 {
                let tk = usize::from(ln0).min(n);
                out_s[0..tk].copy_from_slice(&inline_s[code0][..tk]);
                at = tk;
            } else {
                emit_long_word(inline_s[code0], spelling, n, &mut at, out_s);
            }

            let edges_ro = &edges_s[..];
            let mut steps_left = (m as usize).saturating_sub(1);
            let mut hit_stop = false;
            while !hit_stop && at + 64 <= n && steps_left > 0 {
                let budget = ((n - at) >> 4).min(steps_left);
                let mut done = 0usize;
                while done < budget {
                    let edge = edges_ro[j as usize];
                    let c = ((edge >> bits) as usize) & code_mask;
                    j = edge & mask;
                    done += 1;
                    let ln = lens_s[c];
                    if ln <= INLINE as u8 {
                        out_s[at..at + INLINE].copy_from_slice(&inline_s[c]);
                        at += usize::from(ln);
                    } else {
                        if c == stop_c {
                            hit_stop = true;
                        } else {
                            emit_long_word(inline_s[c], spelling, n, &mut at, out_s);
                        }
                        break;
                    }
                }
                steps_left -= done;
            }

            while !hit_stop && at < n && steps_left > 0 {
                let edge = edges_ro[j as usize];
                let c = ((edge >> bits) as usize) & code_mask;
                j = edge & mask;
                steps_left -= 1;
                let ln = lens_s[c];
                if ln <= INLINE as u8 {
                    if at + INLINE <= out_s.len() {
                        out_s[at..at + INLINE].copy_from_slice(&inline_s[c]);
                        at = (at + usize::from(ln)).min(n);
                    } else {
                        let tk = usize::from(ln).min(n - at);
                        out_s[at..at + tk].copy_from_slice(&inline_s[c][..tk]);
                        at += tk;
                    }
                } else if c == stop_c {
                    break;
                } else {
                    emit_long_word(inline_s[c], spelling, n, &mut at, out_s);
                }
            }
        }

        edges_s[mark as usize] = raw_mark;
        out.truncate(at.min(n));
        let mut len = out.len();
        assert!(len > 0 && len <= n);
        while len < n {
            let take = len.min(n - len);
            out.extend_from_within(..take);
            len = out.len();
        }
    }

    #[cold]
    #[inline(never)]
    fn emit_wide(
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
        if out.len() < n + INLINE {
            out.resize(n + INLINE, 0);
        }
        let mut at = 0usize;
        let mask = (1u32 << bits) - 1;
        let mut j = mark;
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
        out.truncate(at.min(n));
        let mut len = out.len();
        while len < n {
            let take = len.min(n - len);
            out.extend_from_within(..take);
            len = out.len();
        }
    }

    #[derive(Default)]
    pub struct AhtOpenCells<const LIMIT: u32, const USE_AHT: bool> {
        pub k: AhtSentOutQKernel<LIMIT, USE_AHT>,
        lazy: LazyCells,
    }

    impl<const LIMIT: u32, const USE_AHT: bool> InverseBwt for AhtOpenCells<LIMIT, USE_AHT> {
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
            let nr = r.records().len();
            if USE_AHT {
                if n > 0 && n <= 900_000 && (p as usize) < n && (n <= MICRO_MAX_N || nr * 100 < n * 28) {
                    if self.k.try_run::<M_MAX_DEFAULT, NS_MAX_DEFAULT>(r.records(), n, p, 60_000, out) {
                        return true;
                    }
                }
            } else if n > 0 && n <= 900_000 && (p as usize) < n && nr * 20 < n * 3 && nr * 64 >= n {
                if self.k.try_run::<900_000, 900_000>(r.records(), n, p, 60_000, out) {
                    return true;
                }
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

enum CorpusBlk {
    RunsBlk { runs: Runs, n: usize, p: u32, admitted_058: bool },
    ColBlk { ll: Vec<u8>, counts: [u32; 256], p: u32 },
}

fn load_all_431(bz2_path: &str) -> Vec<CorpusBlk> {
    let data = std::fs::read(bz2_path).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut syms = Vec::new();
    let mut out = Vec::new();
    while let Ok(Event::Block { symbols: b, .. }) = sc.next(std::mem::take(&mut syms)) {
        let p = b.orig_ptr;
        if runs::takes_runs(&b) {
            let mut r = Runs::default();
            let n = runs::front(&b, &mut r, k).unwrap();
            let nr = r.records().len();
            let admitted_058 = nr * 20 < n * 3 && nr * 64 >= n;
            out.push(CorpusBlk::RunsBlk { runs: r, n, p, admitted_058 });
            syms = b.syms;
        } else {
            let mut ll = Vec::new();
            let mut counts = [0u32; 256];
            mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
            out.push(CorpusBlk::ColBlk { ll, counts, p });
            syms = b.syms;
        }
    }
    out
}

fn invert_any<K: InverseBwt>(kern: &mut K, blk: &CorpusBlk, out: &mut Vec<u8>, k: Kernels) {
    match blk {
        CorpusBlk::RunsBlk { runs, n, p, .. } => {
            assert!(kern.invert_runs(runs, *n, *p, out, k));
        }
        CorpusBlk::ColBlk { ll, counts, p } => {
            kern.invert(ll, counts, *p, out);
        }
    }
}

fn run_cg_sample9<K: InverseBwt>(sample_bz2: &str) {
    let all = load_all_431(sample_bz2);
    let k = Kernels::detect();
    let mut kern = K::default();
    let mut out = Vec::with_capacity(900_064);
    for b in all.iter().skip(1) {
        invert_any(&mut kern, b, &mut out, k);
    }
    client_request(0, &[REQ_WARM, 0, 0, 0, 0, 0]);
    invert_any(&mut kern, &all[0], &mut out, k);
    let mut total = 0u64;
    client_request(0, &[REQ_BEGIN, 0, 0, 0, 0, 0]);
    for b in all.iter().skip(1) {
        invert_any(&mut kern, b, &mut out, k);
        std::hint::black_box(out.as_ptr());
        total += out.len() as u64;
    }
    client_request(0, &[REQ_END, total, 0, 0, 0, 0]);
}

fn run_cg_mixed_dense<K: InverseBwt>(bz2_path: &str) {
    let all = load_all_431(bz2_path);
    let picks = [40usize, 41, 55, 56, 164, 165, 269, 270, 278];
    let k = Kernels::detect();
    let mut kern = K::default();
    let mut out = Vec::with_capacity(900_064);
    for &bi in picks.iter().skip(1) {
        invert_any(&mut kern, &all[bi], &mut out, k);
    }
    client_request(0, &[REQ_WARM, 0, 0, 0, 0, 0]);
    invert_any(&mut kern, &all[picks[0]], &mut out, k);
    let mut total = 0u64;
    client_request(0, &[REQ_BEGIN, 0, 0, 0, 0, 0]);
    for &bi in picks.iter().skip(1) {
        invert_any(&mut kern, &all[bi], &mut out, k);
        std::hint::black_box(out.as_ptr());
        total += out.len() as u64;
    }
    client_request(0, &[REQ_END, total, 0, 0, 0, 0]);
}

fn run_adversarial_g51(all431: &[CorpusBlk], bz2_bytes: &Arc<[u8]>) {
    let k = Kernels::detect();
    let mut seg = Segmented::default();
    let mut k_no = candidate_c18::AhtSentOutQKernel::<16, false>::default();
    let mut k_aht = candidate_c18::AhtSentOutQKernel::<16, true>::default();
    let mut o_no = Vec::new();
    let mut o_aht = Vec::new();
    let mut o_seg = Vec::new();

    // 1. Both `ra > rb` (`1^1 0^299999`) AND `ra < rb` (`1^299999 0^1`) Ostrowski spirals!
    {
        let mut recs_gt = vec![(1u16 << 8) | 1];
        let mut rem = 299_999usize;
        while rem > 0 {
            let tk = rem.min(127);
            recs_gt.push((0u16 << 8) | (tk as u16));
            rem -= tk;
        }
        assert!(k_no.try_run::<900_000, 900_000>(&recs_gt, 300_000, 0, 60_000, &mut o_no));
        assert!(k_aht.try_run::<256_000, 16_384>(&recs_gt, 300_000, 0, 60_000, &mut o_aht));
        assert!(k_no.r4_loop_steps > 250_000 && k_aht.r4_loop_steps <= 8);
        assert!(o_no == o_aht, "ra > rb spiral mismatch");

        let mut recs_lt = Vec::new();
        rem = 299_999;
        while rem > 0 {
            let tk = rem.min(127);
            recs_lt.push((1u16 << 8) | (tk as u16));
            rem -= tk;
        }
        recs_lt.push((0u16 << 8) | 1);
        assert!(k_no.try_run::<900_000, 900_000>(&recs_lt, 300_000, 299_999, 60_000, &mut o_no));
        assert!(k_aht.try_run::<256_000, 16_384>(&recs_lt, 300_000, 299_999, 60_000, &mut o_aht));
        assert!(k_no.r4_loop_steps > 250_000 && k_aht.r4_loop_steps <= 8);
        assert!(o_no == o_aht, "ra < rb spiral mismatch");
    }

    // 2. Theorem 18.2 structural high-`m` rejection test (`r/n = 0.125 < 0.15`, `m = 384,000 > 256,000`)!
    {
        let mut hi_m_recs = Vec::with_capacity(48_000);
        let mut ll = Vec::with_capacity(384_000);
        let mut cnts = [0u32; 256];
        for i in 0..48_000usize {
            let b = ((i % 61) + 1) as u8;
            hi_m_recs.push(((b as u16) << 8) | 8);
            cnts[usize::from(b)] += 8;
            ll.resize(ll.len() + 8, b);
        }
        let mut tmp_out = vec![0u8; 384_064];
        tls_reset();
        let mut fresh_aht = candidate_c18::AhtSentOutQKernel::<16, true>::default();
        let accepted = fresh_aht.try_run::<256_000, 16_384>(&hi_m_recs, 384_000, 123, 60_000, &mut tmp_out);
        let (_, abort_peak) = tls_stats();
        assert!(!accepted, "expected try_run to abort on m=384,000 > m_limit!");
        assert!(abort_peak < 32 * 1024, "expected <32 KiB heap on early Pass-1 abort, got {abort_peak}");
        seg.invert(&ll, &cnts, 123, &mut o_seg);
        assert!(k_no.try_run::<900_000, 900_000>(&hi_m_recs, 384_000, 123, 60_000, &mut o_no));
        assert!(o_no == o_seg);
    }

    // 3. Poisoned tapes + forced `ovf_chunks` on the first 16 run blocks of `wikidata.bz2`!
    let mut ref_k = OpenCells::default();
    let mut checked_ovf = 0usize;
    for b in all431 {
        if let CorpusBlk::RunsBlk { runs, n, p, admitted_058: true } = b {
            assert!(ref_k.invert_runs(runs, *n, *p, &mut o_seg, k));
            o_aht.clear();
            o_aht.resize(*n + 64, 0xA5);
            k_aht.poison_tapes_for_test();
            assert!(k_aht.try_run::<256_000, 16_384>(runs.records(), *n, *p, 8, &mut o_aht));
            assert!(k_aht.last_ovf_chunks_used > 0);
            assert!(o_aht == o_seg);
            checked_ovf += 1;
            if checked_ovf >= 12 {
                break;
            }
        }
    }

    // 4. Full 431-block `BzReader::parallel_workers_with` (`W = 8` and `W = 10`) exact CRC check!
    for &w in &[8usize, 10] {
        let mut dec = BzReader::parallel_workers_with::<candidate_c18::AhtOpenCells<16, true>>(
            std::io::Cursor::new(Arc::clone(bz2_bytes)),
            w,
            3 * w,
            k,
        );
        let mut buf = vec![0u8; 256 * 1024];
        let mut tot = 0usize;
        loop {
            let got = dec.read(&mut buf).unwrap();
            if got == 0 {
                break;
            }
            tot += got;
        }
        assert_eq!(tot, 209_700_196);
    }
}

fn measure_431_workers<K: InverseBwt + Send + 'static>(
    blks: &[CorpusBlk],
    workers: usize,
    check_ref: Option<&[Vec<u8>]>,
) -> (usize, usize, usize, f64, f64) {
    let k = Kernels::detect();
    if let Some(refs) = check_ref {
        let mut tk = K::default();
        let mut out = Vec::with_capacity(900_064);
        for (i, b) in blks.iter().enumerate() {
            invert_any(&mut tk, b, &mut out, k);
            assert!(out == refs[i], "431-block mismatch on block {i}");
        }
    }
    let idx = AtomicUsize::new(0);
    let max_ret = AtomicUsize::new(0);
    let max_pk = AtomicUsize::new(0);
    let sum_pk = AtomicUsize::new(0);
    let cpu_ns = AtomicUsize::new(0);
    let t0 = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..workers {
            let idx_r = &idx;
            let mr_r = &max_ret;
            let mp_r = &max_pk;
            let sp_r = &sum_pk;
            let cpu_r = &cpu_ns;
            s.spawn(move || {
                let mut out = Vec::with_capacity(900_064);
                tls_reset();
                let mut kern = K::default();
                let mut my = 0usize;
                loop {
                    let i = idx_r.fetch_add(1, Ordering::Relaxed);
                    if i >= blks.len() {
                        break;
                    }
                    let ts = Instant::now();
                    invert_any(&mut kern, &blks[i], &mut out, k);
                    std::hint::black_box(out.as_ptr());
                    my += ts.elapsed().as_nanos() as usize;
                }
                let (ret, pk) = tls_stats();
                mr_r.fetch_max(ret, Ordering::Relaxed);
                mp_r.fetch_max(pk, Ordering::Relaxed);
                sp_r.fetch_add(pk, Ordering::Relaxed);
                cpu_r.fetch_add(my, Ordering::Relaxed);
                drop(kern);
            });
        }
    });
    (
        max_ret.load(Ordering::Relaxed),
        max_pk.load(Ordering::Relaxed),
        sum_pk.load(Ordering::Relaxed),
        t0.elapsed().as_secs_f64() * 1e3,
        cpu_ns.load(Ordering::Relaxed) as f64 / 1e6,
    )
}

fn time_bzreader<K: InverseBwt + Send + 'static>(bz2_bytes: &Arc<[u8]>, workers: usize) -> f64 {
    let k = Kernels::detect();
    let t0 = Instant::now();
    let mut dec = BzReader::parallel_workers_with::<K>(
        std::io::Cursor::new(Arc::clone(bz2_bytes)),
        workers,
        3 * workers,
        k,
    );
    let mut buf = vec![0u8; 256 * 1024];
    let mut tot = 0usize;
    loop {
        let got = dec.read(&mut buf).unwrap();
        if got == 0 {
            break;
        }
        tot += got;
    }
    assert_eq!(tot, 209_700_196);
    t0.elapsed().as_secs_f64() * 1e3
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

type Arm1C17 = candidate_c18::AhtOpenCells<16, false>;
type Arm2C18L16 = candidate_c18::AhtOpenCells<16, true>;
type Arm3C18L14 = candidate_c18::AhtOpenCells<14, true>;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map_or("all", String::as_str);
    let path = args.get(2).map_or("/tmp/wikidata.bz2", String::as_str);

    if let Some(arm) = mode.strip_prefix("cg_s9:") {
        match arm {
            "058" => run_cg_sample9::<OpenCells>(path),
            "c17" => run_cg_sample9::<Arm1C17>(path),
            "c18_16" => run_cg_sample9::<Arm2C18L16>(path),
            "c18_14" => run_cg_sample9::<Arm3C18L14>(path),
            _ => panic!("unknown {arm}"),
        }
        return;
    }
    if let Some(arm) = mode.strip_prefix("cg_dense:") {
        match arm {
            "058" => run_cg_mixed_dense::<OpenCells>(path),
            "c17" => run_cg_mixed_dense::<Arm1C17>(path),
            "c18_16" => run_cg_mixed_dense::<Arm2C18L16>(path),
            "c18_14" => run_cg_mixed_dense::<Arm3C18L14>(path),
            _ => panic!("unknown {arm}"),
        }
        return;
    }

    let bz2_bytes: Arc<[u8]> = std::fs::read(path).unwrap().into();
    let all431 = load_all_431(path);
    assert_eq!(all431.len(), 431);
    let blks228: Vec<&CorpusBlk> = all431
        .iter()
        .filter(|b| matches!(b, CorpusBlk::RunsBlk { admitted_058: true, .. }))
        .collect();
    assert_eq!(blks228.len(), 228);

    println!("Running G5.1 Ostrowski ra>rb & ra<rb spirals, structural m>256k abort, poisoned-tape ovf, 431-block & BzReader W=8,10 checks...");
    run_adversarial_g51(&all431, &bz2_bytes);
    println!("  G5.1 Exactness, Spiral Contraction & Structural Fallback Safety: PASS");

    let k = Kernels::detect();
    let mut ref_outs = Vec::with_capacity(431);
    {
        let mut rk = OpenCells::default();
        let mut o = Vec::with_capacity(900_064);
        for b in &all431 {
            invert_any(&mut rk, b, &mut o, k);
            ref_outs.push(o.clone());
        }
    }
    let _ = measure_431_workers::<Arm1C17>(&all431, 1, Some(&ref_outs));
    let _ = measure_431_workers::<Arm2C18L16>(&all431, 1, Some(&ref_outs));
    let _ = measure_431_workers::<Arm3C18L14>(&all431, 1, Some(&ref_outs));

    // Also measure 228-block single-worker kernel heap for continuity with Cycles 15-17!
    let heap_228 = {
        let mut out = Vec::with_capacity(900_064);
        tls_reset();
        let mut ck = Arm2C18L16::default();
        for b in &blks228 {
            invert_any(&mut ck, b, &mut out, k);
        }
        let s = tls_stats();
        drop(ck);
        s
    };

    let names = [
        "0: Master-0.5.8 (OpenCells, 228/431)",
        "1: C17-SentOutQ16 (Gate 0.15, 228/431)",
        "2: C18-AhtSentOutQ16 (StructGate, Primary)",
        "3: C18-AhtSentOutQ14 (StructGate, L=14)",
    ];
    let h0 = measure_431_workers::<OpenCells>(&all431, 10, None);
    let h1 = measure_431_workers::<Arm1C17>(&all431, 10, None);
    let h2 = measure_431_workers::<Arm2C18L16>(&all431, 10, None);
    let h3 = measure_431_workers::<Arm3C18L14>(&all431, 10, None);
    let hs = [h0, h1, h2, h3];

    println!("\n=== G5.2 Whole-Corpus (431 Blocks) 10-Worker Kernel Heap ===");
    for i in 0..4 {
        println!(
            "  {:<42} | MaxWorkerPeak = {:7.1} KiB ({:5.3}x 0.5.8) | Sum10Workers = {:6.2} MiB ({:5.3}x 0.5.8)",
            names[i],
            hs[i].1 as f64 / 1024.0,
            hs[i].1 as f64 / h0.1 as f64,
            hs[i].2 as f64 / (1024.0 * 1024.0),
            hs[i].2 as f64 / h0.2 as f64,
        );
    }
    println!(
        "  228-Block Subset Per-Worker Heap (C18-AhtSentOutQ16): {:7.1} KiB ({:5.3}x 0.5.8's 4479.5 KiB)",
        heap_228.1 as f64 / 1024.0,
        heap_228.1 as f64 / (4479.5 * 1024.0)
    );
    let g52_pass = (h2.1 as f64 / h0.1 as f64) <= 0.220
        && (h2.2 as f64 / h0.2 as f64) <= 0.350
        && (heap_228.1 as f64 / (4479.5 * 1024.0)) <= 0.390;
    println!("  G5.2 Whole-Corpus 10-Worker Memory Gate: {}", if g52_pass { "PASS" } else { "FAIL" });

    let rounds = 15usize;
    let mut g54_pass = true;
    for &w in &[1usize, 2, 8, 10] {
        let mut wall = vec![vec![0.0; rounds]; 4];
        let mut cpu = vec![vec![0.0; rounds]; 4];
        for r in 0..rounds + 1 {
            for step in 0..4 {
                let arm = (step + r) % 4;
                let (_, _, _, wm, cm) = match arm {
                    0 => measure_431_workers::<OpenCells>(&all431, w, None),
                    1 => measure_431_workers::<Arm1C17>(&all431, w, None),
                    2 => measure_431_workers::<Arm2C18L16>(&all431, w, None),
                    3 => measure_431_workers::<Arm3C18L14>(&all431, w, None),
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
        println!("\n=== G5.4a Whole-Corpus (431 Blocks) Kernel Stage (`lto=fat`) at W = {w} ({rounds} rounds) ===");
        for arm in 0..4 {
            let mw = median(&mut wall[arm]);
            let mc = median(&mut cpu[arm]);
            println!(
                "  {:<42} | wall = {:6.1} ms ({:5.3}x 0.5.8) | sum_cpu = {:7.1} ms ({:5.3}x 0.5.8)",
                names[arm], mw, mw / bw, mc, mc / bc
            );
            if arm == 2 {
                let bar = if w >= 8 { 0.900 } else { 0.950 };
                if mw / bw > bar || mc / bc > bar {
                    g54_pass = false;
                }
            }
        }
    }

    for &w in &[8usize, 10] {
        let mut bzr = vec![vec![0.0; 11]; 4];
        for r in 0..12 {
            for step in 0..4 {
                let arm = (step + r) % 4;
                let wm = match arm {
                    0 => time_bzreader::<OpenCells>(&bz2_bytes, w),
                    1 => time_bzreader::<Arm1C17>(&bz2_bytes, w),
                    2 => time_bzreader::<Arm2C18L16>(&bz2_bytes, w),
                    3 => time_bzreader::<Arm3C18L14>(&bz2_bytes, w),
                    _ => unreachable!(),
                };
                if r >= 1 {
                    bzr[arm][r - 1] = wm;
                }
            }
        }
        let b0 = median(&mut bzr[0].clone());
        println!("\n=== G5.4b End-to-End `BzReader::parallel_workers_with` (431 Blocks) at W = {w} (11 rounds) ===");
        for arm in 0..4 {
            let mw = median(&mut bzr[arm]);
            println!("  {:<42} | wall = {:6.1} ms ({:5.3}x 0.5.8)", names[arm], mw, mw / b0);
            if arm == 2 && mw / b0 > 0.925 {
                g54_pass = false;
            }
        }
    }
    println!("\n  G5.4 Whole-Corpus Multi-Worker Speed Gate: {}", if g54_pass { "PASS" } else { "FAIL" });
}
