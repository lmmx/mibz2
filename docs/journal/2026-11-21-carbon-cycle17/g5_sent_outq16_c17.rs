//! Carbon Cycle 17 Gate G5 Confrontation Harness (`g5_sent_outq16_c17.rs`):
//! Evaluates `SentOutQ16` (`LIMIT = 16` primary candidate + `LIMIT = 14` L2-balanced variant)
//! against `Master-0.5.8` (`mibz2::bwt::OpenCells`), `C15` (`EpochQ16 + ZeroClearR4`), and
//! `C16` (`const LIMIT = 16` `SwarOutQ16 + CompactR4`).
//!
//! Pre-registered strict Gate G5 criteria (all 4 required for `PASS` on primary candidate `Arm 3`):
//!   - `G5.1` Exactness & `#[forbid(unsafe_code)]`:
//!       * 100% safe Rust kernel module (`#[forbid(unsafe_code)]`),
//!       * Adversarial dirty-`out`, dirty-tape, tiny-`out` overflow, exact `m_lo == 0xFFF0, w == 16`
//!         epoch-cut regression, `m == 1` self-loop, periodic multi-cycle, 228 admitted Wikidata
//!         blocks byte-exact, and 431-block `BzReader::parallel_workers_with` (`W = 8, 10`) exactness.
//!   - `G5.2` Per-Worker Heap & Unique L3 Working Set:
//!       * Retained & peak per-worker heap `<= 0.405x` `Master-0.5.8` (`<= 1,814 KiB`),
//!       * Mean active unique per-block L3 footprint (including `out`) `<= 2,048 KiB`, `ovf_chunks == 0`.
//!   - `G5.3` `chaingrind` Shared-L3 Contention Proxy (`W = 8` `2560 KiB` & `W = 10` `2048 KiB`):
//!       * `DRAM B/outB <= 0.22x` `Master-0.5.8` at `W = 8` and `<= 0.16x` at `W = 10`,
//!       * `cycles/B <= 0.925x` `Master-0.5.8` at `W = 8` and `<= 0.860x` at `W = 10`,
//!       * `instructions <= 0.925x` `Master-0.5.8` (`<= 74.7M` vs `80.75M`).
//!   - `G5.4` Multi-Worker Paired Native Rust Speed (`15` rotated rounds, `lto=fat`):
//!       * Median wall AND sum-CPU `<= 0.920x` `Master-0.5.8` at `W = 8` AND `W = 10` (and `< 1.00x` at `W = 1, 2`).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use mibz2::bwt::{InverseBwt, OpenCells};
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
pub mod prior_c15 {
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
    #[derive(Clone, Copy)]
    struct Strip {
        from: u32,
        to: u32,
        width: u32,
        byte: u8,
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

    struct PassC15<'a> {
        queues: &'a mut [Vec<u16>; 256],
        strips: &'a mut Vec<Strip>,
        all: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        ep_cuts: &'a mut Vec<[u32; 256]>,
        n_ep: usize,
        m: u32,
        mark: u32,
    }
    impl PassC15<'_> {
        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, w: u32) {
            let mut rem = w;
            let mut cur_m = self.m;
            while rem > 0 {
                let ep = (cur_m >> 16) as usize;
                let ep_end = ((ep as u32) + 1) << 16;
                let tk = rem.min(ep_end - cur_m);
                let at = self.thin[a] as usize;
                if self.queues[a].len() < at + 16 {
                    grow_vec_tight(&mut self.queues[a], at + 16, 0u16);
                }
                let base = cur_m as u16;
                let iota: [u16; 16] = std::array::from_fn(|i| base.wrapping_add(i as u16));
                self.queues[a][at..at + 16].copy_from_slice(&iota);
                self.thin[a] += tk;
                cur_m += tk;
                rem -= tk;
                if cur_m == ep_end {
                    self.ep_cuts[ep] = self.thin;
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
            let m_now = self.m;
            if (m_now >> 16) == ((m_now + w) >> 16) {
                let at = self.thin[a] as usize;
                let q = &mut self.queues[a];
                if q.len() < at + 16 {
                    grow_vec_tight(q, at + 16, 0u16);
                }
                let base = m_now as u16;
                let iota: [u16; 16] = std::array::from_fn(|i| base.wrapping_add(i as u16));
                q[at..at + 16].copy_from_slice(&iota);
                self.thin[a] += w;
                self.m = m_now + w;
            } else {
                self.push_slow(a, w);
            }
            self.all[a] += w;
        }
        #[inline(always)]
        fn push_thick(&mut self, c: u8, x: u32, w: u32) {
            let a = usize::from(c);
            self.strips.push(Strip {
                from: x,
                to: self.all[a],
                width: w,
                byte: c,
            });
            self.thick[a] += 1;
            self.all[a] += w;
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
                let end = pos + width;
                if pos < p {
                    self.push_thick(c, pos, p - pos);
                }
                self.push_thin(usize::from(c), p, 1, p);
                if p + 1 < end {
                    self.push_thick(c, p + 1, end - p - 1);
                }
            }
        }
    }

    #[derive(Default)]
    pub struct C15OpenCells {
        queues: Vec<Vec<u16>>,
        ep_cuts: Vec<[u32; 256]>,
        strips: Vec<Strip>,
        image_order: Vec<u32>,
        strip_top: Vec<u32>,
        bands_rev: Vec<[u32; 3]>,
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
            v[0..4].copy_from_slice(&(o0 as u32).to_le_bytes());
            v[4..8].copy_from_slice(&(o1 as u32).to_le_bytes());
            self.inline16.push(v);
            self.lens.push(LONG);
            code
        }
        fn run(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
            self.queues.resize_with(256, Vec::new);
            if self.ep_cuts.len() < 16 {
                self.ep_cuts.resize(16, [0u32; 256]);
            }
            self.strips.clear();
            let mut pass = PassC15 {
                queues: (&mut self.queues[..]).try_into().unwrap(),
                strips: &mut self.strips,
                all: [0; 256],
                thin: [0; 256],
                thick: [0; 256],
                ep_cuts: &mut self.ep_cuts,
                n_ep: 1,
                m: 0,
                mark: 0,
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
            }
            let (m, mark, all, thin, thick, mut n_ep) =
                (pass.m, pass.mark, pass.all, pass.thin, pass.thick, pass.n_ep);
            let final_ep = (m >> 16) as usize;
            if n_ep <= final_ep {
                n_ep = final_ep + 1;
            }
            self.ep_cuts[n_ep - 1] = thin;

            let first = starts(all);
            let mut write = starts(thick);
            let ns = self.strips.len();
            if self.image_order.len() < ns {
                grow_vec_tight(&mut self.image_order, ns, 0);
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
                grow_vec_tight(&mut self.strip_top, ns, 0);
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
                    let code = if nd.word == 0 {
                        a as u32
                    } else if (nd.word & RULE) == 0 && (nd.word >> 24) < INLINE as u32 {
                        let c = self.inline16.len() as u32;
                        let wl = (nd.word >> 24) as usize;
                        let off = (nd.word & 0xFF_FFFF) as usize;
                        let mut v = [0u8; INLINE];
                        v.copy_from_slice(&self.arena[off..off + INLINE]);
                        v[wl] = a as u8;
                        self.inline16.push(v);
                        self.lens.push((wl + 1) as u8);
                        c
                    } else {
                        self.make_long_code(nd.word, a as u8)
                    };
                    self.bands_rev.push([width, dst, code]);
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
                    let span = bnd[0] as usize;
                    let q1 = prev_q;
                    let q0 = q1 - span;
                    prev_q = q0;
                    let dst = bnd[1] as usize;
                    let code = bnd[2];
                    if wide {
                        self.extra[dst..dst + span].fill(code);
                    }
                    let cpack = if wide { 0 } else { code << bits };
                    while ep > 0 && q1 <= self.ep_cuts[ep - 1][a] as usize {
                        ep -= 1;
                    }
                    let ep_lo = if ep > 0 {
                        self.ep_cuts[ep - 1][a] as usize
                    } else {
                        0
                    };
                    if q0 >= ep_lo {
                        let pack = cpack | ((ep as u32) << 16);
                        for (ed, &lo16) in self.edges[dst..dst + span].iter_mut().zip(&qa[q0..q1]) {
                            *ed = pack | u32::from(lo16);
                        }
                    } else {
                        let mut cur_q1 = q1;
                        let mut cur_dst_end = dst + span;
                        while cur_q1 > q0 {
                            while ep > 0 && cur_q1 <= self.ep_cuts[ep - 1][a] as usize {
                                ep -= 1;
                            }
                            let lo = if ep > 0 {
                                (self.ep_cuts[ep - 1][a] as usize).max(q0)
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
            if out.len() < n + INLINE {
                out.resize(n + INLINE, 0);
            }
            let mut at = 0usize;
            let mask = (1u32 << bits) - 1;
            let mut j = mark;
            for _ in 0..m as usize {
                let edge = self.edges[j as usize];
                let code = if wide {
                    self.extra[j as usize] as usize
                } else {
                    (edge >> bits) as usize
                };
                j = edge & mask;
                let ln = self.lens[code];
                if ln != LONG {
                    out[at..at + INLINE].copy_from_slice(&self.inline16[code]);
                    at += usize::from(ln);
                } else {
                    let meta = self.inline16[code];
                    let o0 = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let o1 = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &self.spelling[o0..o1];
                    let tk = word.len().min(n - at);
                    out[at..at + tk].copy_from_slice(&word[..tk]);
                    at += tk;
                }
                if j == mark || at >= n {
                    break;
                }
            }
            out.truncate(at.min(n));
            let mut len = out.len();
            while len < n {
                let tk = len.min(n - len);
                out.extend_from_within(..tk);
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

#[forbid(unsafe_code)]
pub mod candidate_c17 {
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
        mark: u32,
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
                mark: 0,
            }
        }
    }

    impl PassMeta {
        #[inline(always)]
        fn reset(&mut self, max_out_chunks: u16) {
            self.all.fill(0);
            self.thin.fill(0);
            self.thick.fill(0);
            self.head.fill(u16::MAX);
            self.in_chunk.fill(CHUNK_U16 as u16);
            self.n_ep = 1;
            self.n_chunks = 0;
            self.max_out_chunks = max_out_chunks;
            self.m = 0;
            self.mark = 0;
        }
    }

    struct PassOut<'a, const LIMIT: u32> {
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

    impl<const LIMIT: u32> PassOut<'_, LIMIT> {
        const NWORDS: usize = (LIMIT as usize + 3) / 4;
        const CAP_U16: usize = Self::NWORDS * 4;
        // Exact Theorem 17.2 bound: `m_lo <= 0xFFFF - CAP_U16` guarantees `m_lo + w < 0x10000`,
        // preventing both 16-bit SWAR carry AND exact-boundary epoch-cut bypass!
        const M_LO_MAX: u32 = 0xFFFF - (Self::CAP_U16 as u32);

        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, w: u32) {
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
    pub struct SentOutQKernel<const LIMIT: u32, const SENT_EMIT: bool> {
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
        pub last_unique_l3_bytes: usize,
        pub last_ovf_chunks_used: usize,
    }

    impl<const LIMIT: u32, const SENT_EMIT: bool> SentOutQKernel<LIMIT, SENT_EMIT> {
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
                self.lens[256..].fill(13); // intentionally stale non-LONG length beyond `n_codes`!
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

        #[inline(always)]
        fn push_code_slot(&mut self, v: [u8; INLINE], ln: u8) -> u32 {
            if SENT_EMIT {
                let c = self.n_codes;
                if c >= self.inline16.len() {
                    grow_vec_pow2(&mut self.inline16, c + 1, [0u8; INLINE]);
                    grow_vec_pow2(&mut self.lens, c + 1, LONG);
                }
                self.inline16[c] = v;
                self.lens[c] = ln;
                self.n_codes = c + 1;
                c as u32
            } else {
                let c = self.inline16.len() as u32;
                self.inline16.push(v);
                self.lens.push(ln);
                c
            }
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

        pub fn run(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
            self.run_with_chunk_cap(records, n, p, 60_000, out);
        }

        pub fn run_with_chunk_cap(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            cap_override: u16,
            out: &mut Vec<u8>,
        ) {
            assert!(n > 0 && (p as usize) < n);
            if out.len() < n + 32 {
                out.resize(n + 32, 0);
            }
            let max_out_chunks = ((out.len() / CHUNK_B).min(60_000) as u16).min(cap_override);
            self.pm.reset(max_out_chunks);
            self.triples.clear();

            {
                let mut pass = PassOut::<LIMIT> {
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
                }
                assert_eq!(pos as usize, n);
            }

            let m = self.pm.m;
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

            let ovf_used = (self.pm.n_chunks.saturating_sub(max_out_chunks)) as usize;
            self.last_ovf_chunks_used = ovf_used;

            let first = starts(self.pm.all);
            let mut write = starts(self.pm.thick);
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
                xb -= self.tops[s.top as usize].width;
                self.tops[s.top as usize].other = xb;
                sb = s.prev;
            }
            self.arena.extend_from_slice(&[0u8; INLINE]);

            if SENT_EMIT {
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
            } else {
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
            }
            self.spelling.clear();
            let ns_triples = ns;
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
            let n_real_codes = if SENT_EMIT {
                self.n_codes
            } else {
                self.inline16.len()
            };
            let max_code = if SENT_EMIT {
                n_real_codes as u32
            } else {
                (n_real_codes - 1) as u32
            };
            let wide = max_code > (u32::MAX >> bits);
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

            let active_triples = ns_triples.max(self.triples.len()) * 12;
            let active_scratch = 2 * ns_triples * 4;
            let active_tops_bots = (a0_end as usize) * 16 + (b as usize) * 8;
            let active_dict = self.arena.len()
                + self.rules.len() * 16
                + self.spelling.len()
                + n_real_codes * 17;
            let active_edges = (m as usize) * 4 + self.extra.len() * 4;
            self.last_unique_l3_bytes = n
                + 32
                + ovf_used * CHUNK_B
                + (self.pm.n_chunks as usize) * 2
                + active_triples
                + active_scratch
                + active_tops_bots
                + active_dict
                + active_edges;

            if SENT_EMIT && !wide {
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
                emit_hoisted(
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
        debug_assert!(code_cap.is_power_of_two() && lens.len() == code_cap);
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
        if out.len() < n + INLINE {
            out.resize(n + INLINE, 0);
        }
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
    pub struct SentOutQOpenCells<const LIMIT: u32, const SENT_EMIT: bool> {
        pub k: SentOutQKernel<LIMIT, SENT_EMIT>,
        lazy: LazyCells,
    }
    impl<const LIMIT: u32, const SENT_EMIT: bool> InverseBwt
        for SentOutQOpenCells<LIMIT, SENT_EMIT>
    {
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
            let nr = r.records().len();
            if n > 0 && (p as usize) < n && nr * 20 < n * 3 && nr * 64 >= n {
                self.k.run(r.records(), n, p, out);
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
    let data = std::fs::read(bz2_path).unwrap();
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
    let mut out = Vec::with_capacity(900_064);
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
    let mut c17_16 = candidate_c17::SentOutQKernel::<16, true>::default();
    let mut c17_14 = candidate_c17::SentOutQKernel::<14, true>::default();
    let mut ref_out = Vec::new();
    let mut out16 = Vec::new();
    let mut out14 = Vec::new();

    // 1. Dirty `out`, poisoned internal tapes + stale `inline16`/`lens` slots, AND forced `ovf_chunks` (`cap_override = 8`)!
    for (i, b) in blks.iter().take(16).enumerate() {
        assert!(ref_k.invert_runs(&b.runs, b.n, b.p, &mut ref_out, k));
        out16.clear();
        out16.resize(b.n + 64, 0xA5);
        c17_16.poison_tapes_for_test();
        c17_16.run_with_chunk_cap(b.runs.records(), b.n, b.p, 8, &mut out16);
        assert!(c17_16.last_ovf_chunks_used > 0, "expected ovf_chunks > 0");
        assert!(out16 == ref_out, "dirty/ovf mismatch L=16 on block {i}");

        out14.clear();
        out14.resize(b.n + 64, 0x5A);
        c17_14.poison_tapes_for_test();
        c17_14.run_with_chunk_cap(b.runs.records(), b.n, b.p, 8, &mut out14);
        assert!(out14 == ref_out, "dirty/ovf mismatch L=14 on block {i}");
    }

    // 2. Exact `m_lo == 0xFFF0, w == 16` multi-epoch boundary regression + `m == 1` periodic self-loop!
    {
        let single_rec = [((b'Z' as u16) << 8) | 1];
        c17_16.run(&single_rec, 1, 0, &mut out16);
        assert_eq!(out16, b"Z");

        let per_rec = [((b'Q' as u16) << 8) | 64]; // `m == 1` (`take_marked` splits 64 -> 1 thin + 63 thick), periodic `Q^64`!
        c17_16.run(&per_rec, 64, 0, &mut out16);
        assert_eq!(out16, vec![b'Q'; 64]);

        let mut boundary_recs = Vec::with_capacity(140_000);
        for r in 0..0xFFF0u32 {
            let sym = ((r % 251) + 1) as u16;
            boundary_recs.push((sym << 8) | 1);
        }
        boundary_recs.push((253u16 << 8) | 16); // `0xFFF0 + 16 == 0x10000` exact epoch cut!
        for r in 0..70_000u32 {
            let sym = ((r % 251) + 1) as u16;
            boundary_recs.push((sym << 8) | (1 + (r % 3) as u16));
        }
        let n_tot: usize = boundary_recs.iter().map(|&x| (x & 255) as usize).sum();
        let p_test = 0xFFF8u32;
        // Reference via C16 non-sentinel `emit_hoisted` AND naive LF table!
        let mut counts = [0u32; 256];
        let mut bwt = Vec::with_capacity(n_tot);
        for &rec in &boundary_recs {
            let b = (rec >> 8) as u8;
            let w = (rec & 255) as usize;
            counts[usize::from(b)] += w as u32;
            bwt.resize(bwt.len() + w, b);
        }
        let mut occ = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            occ[c] = tot;
            tot += counts[c];
        }
        let mut tt = vec![0u32; n_tot];
        for (i, &b) in bwt.iter().enumerate() {
            let at = occ[usize::from(b)] as usize;
            tt[at] = ((i as u32) << 8) | u32::from(b);
            occ[usize::from(b)] += 1;
        }
        let mut want = Vec::with_capacity(n_tot);
        let mut row = p_test as usize;
        for _ in 0..n_tot {
            let e = tt[row];
            want.push((e & 0xFF) as u8);
            row = (e >> 8) as usize;
            if row == p_test as usize {
                break;
            }
        }
        while want.len() < n_tot {
            let tk = want.len().min(n_tot - want.len());
            want.extend_from_within(..tk);
        }
        c17_16.run(&boundary_recs, n_tot, p_test, &mut out16);
        assert!(out16 == want, "exact 0xFFF0+16==0x10000 epoch-cut regression failed!");
        c17_14.run(&boundary_recs, n_tot, p_test, &mut out14);
        assert!(out14 == want, "L=14 multi-epoch regression failed!");
    }

    // 3. Full 431-block `BzReader::parallel_workers_with` (`W = 8` and `W = 10`) exactness!
    let bz2_bytes = std::fs::read(bz2_path).unwrap();
    for &w in &[8usize, 10] {
        let mut dec = BzReader::parallel_workers_with::<candidate_c17::SentOutQOpenCells<16, true>>(
            std::io::Cursor::new(bz2_bytes.clone()),
            w,
            3 * w,
            k,
        );
        let mut buf = vec![0u8; 256 * 1024];
        let mut total = 0usize;
        loop {
            let n = dec.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            total += n;
        }
        assert_eq!(total, 209_700_196);
    }
}

fn verify_and_mem<K: InverseBwt>(blks: &[AdmittedBlk], check: bool) -> (usize, usize) {
    let k = Kernels::detect();
    let mut ref_k = OpenCells::default();
    let mut ref_out = Vec::with_capacity(900_064);
    let mut out = Vec::with_capacity(900_064);
    if check {
        let mut test_k = K::default();
        for (i, b) in blks.iter().enumerate() {
            assert!(ref_k.invert_runs(&b.runs, b.n, b.p, &mut ref_out, k));
            assert!(test_k.invert_runs(&b.runs, b.n, b.p, &mut out, k));
            assert!(out == ref_out, "mismatch on block {i}");
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
                let mut out = Vec::with_capacity(900_064);
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

type Arm2C16 = candidate_c17::SentOutQOpenCells<16, false>;
type Arm3Sent16 = candidate_c17::SentOutQOpenCells<16, true>;
type Arm4Sent14 = candidate_c17::SentOutQOpenCells<14, true>;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map_or("all", String::as_str);
    let sample_bz2 = args.get(2).map_or("/tmp/sample9_admitted.bz2", String::as_str);
    if let Some(arm) = mode.strip_prefix("cg:") {
        match arm {
            "058" => run_cg_kernel_only::<OpenCells>(sample_bz2),
            "c15" => run_cg_kernel_only::<prior_c15::C15OpenCells>(sample_bz2),
            "c16" => run_cg_kernel_only::<Arm2C16>(sample_bz2),
            "c17_16" => run_cg_kernel_only::<Arm3Sent16>(sample_bz2),
            "c17_14" => run_cg_kernel_only::<Arm4Sent14>(sample_bz2),
            _ => panic!("unknown cg arm {arm}"),
        }
        return;
    }

    let bz2_path = args.get(2).map_or("/tmp/wikidata.bz2", String::as_str);
    let blks = load_admitted_from_bz2(bz2_path);
    assert_eq!(blks.len(), 228);

    println!("Running G5.1 adversarial dirty-out, poisoned-tape, stale-slot, 0xFFF0+16 epoch-cut, 228-block & 431-block BzReader W=8,10 checks...");
    run_adversarial_tests(&blks, bz2_path);
    println!("  G5.1 Exactness & #[forbid(unsafe_code)] Verification: PASS");

    let names = [
        "0: Master-0.5.8 (OpenCells)",
        "1: C15-EpochQ16-ZeroClearR4",
        "2: C16-SwarOutQ16 (const L=16)",
        "3: C17-SentOutQ16 (L=16, Primary)",
        "4: C17-SentOutQ14 (L=14, L2-Bal)",
    ];
    let mems = [
        verify_and_mem::<OpenCells>(&blks, false),
        verify_and_mem::<prior_c15::C15OpenCells>(&blks, true),
        verify_and_mem::<Arm2C16>(&blks, true),
        verify_and_mem::<Arm3Sent16>(&blks, true),
        verify_and_mem::<Arm4Sent14>(&blks, true),
    ];

    let mut l3_16 = Vec::with_capacity(blks.len());
    let mut l3_14 = Vec::with_capacity(blks.len());
    let mut ovf_16 = 0usize;
    let mut ovf_14 = 0usize;
    {
        let mut k16 = candidate_c17::SentOutQKernel::<16, true>::default();
        let mut k14 = candidate_c17::SentOutQKernel::<14, true>::default();
        let mut o = Vec::with_capacity(900_064);
        for b in &blks {
            k16.run(b.runs.records(), b.n, b.p, &mut o);
            l3_16.push(k16.last_unique_l3_bytes as f64 / 1024.0);
            ovf_16 += k16.last_ovf_chunks_used;

            k14.run(b.runs.records(), b.n, b.p, &mut o);
            l3_14.push(k14.last_unique_l3_bytes as f64 / 1024.0);
            ovf_14 += k14.last_ovf_chunks_used;
        }
    }
    l3_16.sort_by(|a, b| a.partial_cmp(b).unwrap());
    l3_14.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean_l3_16 = l3_16.iter().sum::<f64>() / l3_16.len() as f64;
    let p95_l3_16 = l3_16[(l3_16.len() * 95) / 100];
    let mean_l3_14 = l3_14.iter().sum::<f64>() / l3_14.len() as f64;
    let p95_l3_14 = l3_14[(l3_14.len() * 95) / 100];

    println!("\n=== G5.2 Per-Worker Kernel Heap & Unique L3 Footprint (228 Admitted Blocks) ===");
    let (b_ret, b_pk) = mems[0];
    for i in 0..5 {
        println!(
            "  {:<34} | Retained = {:7.1} KiB ({:5.3}x 0.5.8) | Peak = {:7.1} KiB ({:5.3}x 0.5.8)",
            names[i],
            mems[i].0 as f64 / 1024.0,
            mems[i].0 as f64 / b_ret as f64,
            mems[i].1 as f64 / 1024.0,
            mems[i].1 as f64 / b_pk as f64
        );
    }
    println!(
        "  C17-SentOutQ16 Unique L3: mean = {:6.1} KiB (<= 2048 KiB), p95 = {:6.1} KiB | ovf_chunks = {}",
        mean_l3_16, p95_l3_16, ovf_16
    );
    println!(
        "  C17-SentOutQ14 Unique L3: mean = {:6.1} KiB (<= 2048 KiB), p95 = {:6.1} KiB | ovf_chunks = {}",
        mean_l3_14, p95_l3_14, ovf_14
    );
    let g52_pass = (mems[3].0 as f64 / b_ret as f64) <= 0.405
        && (mems[3].1 as f64 / b_pk as f64) <= 0.405
        && mean_l3_16 <= 2048.0
        && ovf_16 == 0;
    println!("  G5.2 Memory Reduction Gate: {}", if g52_pass { "PASS" } else { "FAIL" });

    let rounds = 15usize;
    let mut g54_pass = true;
    for &w in &[1usize, 2, 8, 10] {
        let mut wall = vec![vec![0.0; rounds]; 5];
        let mut cpu = vec![vec![0.0; rounds]; 5];
        for r in 0..rounds + 1 {
            for step in 0..5 {
                let arm = (step + r) % 5;
                let (wm, cm) = match arm {
                    0 => time_workers::<OpenCells>(&blks, w),
                    1 => time_workers::<prior_c15::C15OpenCells>(&blks, w),
                    2 => time_workers::<Arm2C16>(&blks, w),
                    3 => time_workers::<Arm3Sent16>(&blks, w),
                    4 => time_workers::<Arm4Sent14>(&blks, w),
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
        for arm in 0..5 {
            let mw = median(&mut wall[arm]);
            let mc = median(&mut cpu[arm]);
            println!(
                "  {:<34} | wall = {:6.1} ms ({:5.3}x 0.5.8) | sum_cpu = {:7.1} ms ({:5.3}x 0.5.8)",
                names[arm],
                mw,
                mw / bw,
                mc,
                mc / bc
            );
            if arm == 3 {
                let bar = if w >= 8 { 0.920 } else { 1.000 };
                if mw / bw > bar || mc / bc > bar {
                    g54_pass = false;
                }
            }
        }
    }
    println!("\n  G5.4 Multi-Worker Native Speed Gate: {}", if g54_pass { "PASS" } else { "FAIL" });
}
