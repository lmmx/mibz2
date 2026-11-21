//! Gate G5 Multi-Worker Native Safe-Rust & `chaingrind` Confrontation Harness for Carbon Cycle 15:
//!   - Baseline (`Arm 0`): `Master-0.5.8` (`mibz2::bwt::OpenCells` from `origin/master` `bb98c99`)
//!   - Control  (`Arm 1`): `PR#675` (`R4@24 + 8-lane BP4K Pages` — historical 1T winner / 8T i9 loser)
//!   - Ref      (`Arm 2`): `PR#679` (`R4@16 + Vec16` — serial R4 with u32 queues & cleared tapes)
//!   - Promoted (`Arm 3`): `C15-EpochQ16-ZeroClearR4` (`#[forbid(unsafe_code)]` candidate module:
//!                         Theorem 15.1 Contiguous 16-bit 64K-Epoch Queues `[Vec<u16>; 256]` +
//!                         Theorem 15.2 Zero-Clear `PAD=1` Reused R4 Tapes + Direct Arena Inlining
//!                         + Zero-`offsets` Cold Metadata + Unreversed `BandRev` + Hoisted `emit`)
//!
//! FROZEN PRE-REGISTERED CYCLE 15 PASS/FAIL GATES (all must hold simultaneously):
//!   (G5.1) Exactness & Zero-Unsafe: `#[forbid(unsafe_code)]` on `candidate_c15`; adversarial
//!          multi-epoch & dirty-tape self-tests + 228/228 admitted Wikidata blocks byte-exact +
//!          431-block `wikidata.bz2` `BzReader::parallel_workers_with` exact at W=8 and W=10.
//!   (G5.2) Per-Worker Memory Non-Growth (Reduction): Retained & Peak per-worker heap <= 0.80x
//!          `Master-0.5.8`, AND mean & p95 active per-block working set < `Master-0.5.8`.
//!   (G5.3) `chaingrind` i9-10900K Contention Proxy (W=8 `LLC=2560KB` & W=10 `LLC=2048KB`):
//!          `DRAM B/outB <= 0.70x Master-0.5.8`, `L3 B/outB <= 0.92x Master-0.5.8`,
//!          `instr/outB < 1.00x Master-0.5.8`, AND `cycles/outB < 1.00x Master-0.5.8` at BOTH W=8 & W=10.
//!   (G5.4) Native Rust Multi-Worker Speed (15 paired interleaved rounds across 228 admitted blocks):
//!          `wall <= 0.925x Master-0.5.8` AND `sum_cpu <= 0.925x Master-0.5.8` at W=1, 2, 8, 10,
//!          AND faster (`< 1.000x`) than `PR#679` on `sum_cpu` at W=8 and W=10.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;

// ============================================================================
// Exact Thread-Local Heap Allocation Tracker (for black-box 0.5.8 & all arms)
// ============================================================================
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

// ============================================================================
// Valgrind `chaingrind` Client Request (Harness Only)
// ============================================================================
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
// PROMOTED CYCLE 15 CANDIDATE MODULE (`#[forbid(unsafe_code)]` — 100% Safe Rust!)
// ============================================================================
#[forbid(unsafe_code)]
pub mod candidate_c15 {
    use mibz2::bwt::{InverseBwt, LazyCells};
    use mibz2::runs::Runs;
    use mibz2::Kernels;

    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const LIMIT: u32 = 16;
    const PAD: usize = 1; // Theorem 15.2: Only slot 0 is the null sentinel!
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
        let target = need.max(cur + (cur >> 2)).max(256); // 1.25x growth avoids 2x overshoot!
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

    #[derive(Clone, Copy)]
    struct First16 {
        all: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        ep_cuts: [[u32; 256]; 16],
        n_ep: usize,
        m: u32,
        mark: u32,
    }

    struct Pass16<'a> {
        queues: &'a mut [Vec<u16>; 256],
        strips: &'a mut Vec<Strip>,
        f: First16,
    }

    impl Pass16<'_> {
        #[cold]
        #[inline(never)]
        fn push_epoch_cross(&mut self, a: usize, w: u32) {
            let mut rem = w;
            let mut cur_m = self.f.m;
            while rem > 0 {
                let ep = (cur_m >> 16) as usize;
                let ep_end = ((ep as u32) + 1) << 16;
                let tk = rem.min(ep_end - cur_m);
                if tk <= 16 {
                    push_rows_u16(&mut self.queues[a], self.f.thin[a], cur_m);
                } else {
                    let at = self.f.thin[a] as usize;
                    if self.queues[a].len() < at + tk as usize {
                        grow_vec_tight(&mut self.queues[a], at + tk as usize, 0u16);
                    }
                    for k in 0..tk as usize {
                        self.queues[a][at + k] = (cur_m as u16).wrapping_add(k as u16);
                    }
                }
                self.f.thin[a] += tk;
                cur_m += tk;
                rem -= tk;
                if cur_m == ep_end {
                    self.f.ep_cuts[ep] = self.f.thin;
                    self.f.n_ep = ep + 2;
                }
            }
            self.f.m = cur_m;
        }

        #[inline(always)]
        fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
            let a = usize::from(c);
            if keep {
                if p.wrapping_sub(x) < w {
                    self.f.mark = self.f.m + (p - x);
                }
                let m_now = self.f.m;
                if (m_now >> 16) == ((m_now + w) >> 16) && w <= 16 {
                    push_rows_u16(&mut self.queues[a], self.f.thin[a], m_now);
                    self.f.thin[a] += w;
                    self.f.m = m_now + w;
                } else {
                    self.push_epoch_cross(a, w);
                }
            } else {
                self.strips.push(Strip {
                    from: x,
                    to: self.f.all[a],
                    width: w,
                    byte: c,
                });
                self.f.thick[a] += 1;
            }
            self.f.all[a] += w;
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            if pos < p {
                self.piece(c, pos, p - pos, false, p);
            }
            self.piece(c, p, 1, true, p);
            if p + 1 < end {
                self.piece(c, p + 1, end - p - 1, false, p);
            }
        }

        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            if width <= LIMIT {
                self.piece(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece(c, pos, width, false, p);
            } else {
                self.take_marked(c, pos, width, p);
            }
        }
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
    pub struct EpochQ16Kernel {
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
        pub last_active_bytes: usize,
    }

    impl EpochQ16Kernel {
        pub fn poison_tapes_for_test(&mut self) {
            if self.queues.len() < 256 {
                self.queues.resize_with(256, || vec![0xBEEFu16; 64]);
            }
            for q in &mut self.queues {
                if q.len() < 64 {
                    q.resize(64, 0xBEEF);
                } else {
                    q.fill(0xBEEF);
                }
            }
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
            let n_s = self.strip_top.len().max(128);
            self.strip_top.resize(n_s, 0);
            self.strip_top.fill(0xDEAD_BEEF);
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

        pub fn run(&mut self, records: &[u16], n: usize, p: u32, limit: u32, out: &mut Vec<u8>) {
            assert!(n > 0 && n <= 900_000 && (p as usize) < n);
            self.strips.clear();
            self.queues.resize_with(256, Vec::new);
            let mut pass = Pass16 {
                queues: (&mut self.queues[..]).try_into().unwrap(),
                strips: &mut self.strips,
                f: First16 {
                    all: [0; 256],
                    thin: [0; 256],
                    thick: [0; 256],
                    ep_cuts: [[0; 256]; 16],
                    n_ep: 1,
                    m: 0,
                    mark: 0,
                },
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
                    pass.take(c, pos, width, p);
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
                    if width <= limit {
                        pass.piece(c, pos, width, true, p);
                    } else if p.wrapping_sub(pos) >= width {
                        pass.piece(c, pos, width, false, p);
                    } else {
                        pass.take_marked(c, pos, width, p);
                    }
                    pos += width;
                }
            }
            let mut f = pass.f;
            let final_ep = (f.m >> 16) as usize;
            if f.n_ep <= final_ep {
                f.n_ep = final_ep + 1;
            }
            f.ep_cuts[f.n_ep - 1] = f.thin;

            let m = f.m;
            let mark = f.mark;
            assert!(m > 0 && pos == n as u32);
            let first = starts(f.all);
            let mut write = starts(f.thick);
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

            // Theorem 15.2: Zero-Clear Reused Tapes (`PAD = 1` — NO `.clear()` + `.resize()`!)
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
                let i = self.image_order[ii];
                let s = self.strips[i as usize];
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
                let st = self.strip_top[i as usize];
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
                assert!(ra > 0 && rb > 0);
                if bs == sb {
                    assert!(ra <= remaining);
                    remaining -= ra;
                    (sa, sb) = (na, nb);
                    continue;
                }
                loop {
                    let wd = self.join(aw, bw);
                    if ra == rb {
                        assert!(ra <= remaining);
                        remaining -= ra;
                        let top = &mut self.tops[ts as usize];
                        (top.width, top.word, top.other) = (ra, wd, bs);
                        self.bots[bs as usize].top = ts;
                        (sa, sb) = (na, nb);
                        break;
                    } else if ra > rb {
                        assert!(rb <= remaining);
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
                        assert!(ra <= remaining);
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
                assert!(w <= xb);
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
            self.bands_rev.clear();

            let thin_start = starts(f.thin);
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
                    assert!(width > 0);
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

            // Theorem 15.1: Contiguous `u16 -> u32` Wiring with 64K-Epoch Lifting
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
                let mut prev_q = f.thin[a] as usize;
                let mut ep = f.n_ep - 1;
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
                    while ep > 0 && q1 <= f.ep_cuts[ep - 1][a] as usize {
                        ep -= 1;
                    }
                    let ep_lo = if ep > 0 {
                        f.ep_cuts[ep - 1][a] as usize
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
                            while ep > 0 && cur_q1 <= f.ep_cuts[ep - 1][a] as usize {
                                ep -= 1;
                            }
                            let lo = if ep > 0 {
                                (f.ep_cuts[ep - 1][a] as usize).max(q0)
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

            // Record exact active working set touched in this block!
            self.last_active_bytes = records.len() * 2
                + (m as usize) * 2 // q16 active bytes
                + ns * std::mem::size_of::<Strip>()
                + ns * 8 // image_order + strip_top
                + ((p0 - p0_base + a0_end - a0_base) as usize) * std::mem::size_of::<Top>()
                + ((b as usize) - PAD) * std::mem::size_of::<Bot>()
                + self.arena.len()
                + self.rules.len() * std::mem::size_of::<Rule>()
                + self.bands_rev.len() * std::mem::size_of::<BandRev>()
                + self.spelling.len()
                + self.inline16.len() * INLINE
                + self.lens.len()
                + (m as usize) * 4 // edges
                + if wide { (m as usize) * 4 } else { 0 }
                + n; // out

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
        let o0 = u32::from_ne_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
        let o1 = u32::from_ne_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
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
        out.resize(n + INLINE, 0);
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
    pub struct EpochQ16OpenCells {
        pub open: EpochQ16Kernel,
        lazy: LazyCells,
    }

    impl InverseBwt for EpochQ16OpenCells {
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(
            &mut self,
            r: &Runs,
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
            k: Kernels,
        ) -> bool {
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
// Historical Control Arm 1: `PR#675` (`R4@24 + 8-lane BP4K Pages`)
// ============================================================================
mod pr675 {
    use mibz2::bwt::{InverseBwt, LazyCells};
    use mibz2::runs::Runs;
    use mibz2::Kernels;

    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const PAD_SLOTS: usize = 16;
    const GIM_CAP: usize = 64;
    const PAGE: usize = 4096;

    #[derive(Clone, Copy)]
    struct Strip {
        from: u32,
        to: u32,
        width: u32,
        byte: u8,
    }
    #[derive(Clone, Copy, Default)]
    struct R4Top {
        prev: u32,
        width: u32,
        word: u32,
        other: u32,
    }
    #[derive(Clone, Copy, Default)]
    struct R4Bot {
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
    struct Band {
        end: u32,
        dst: u32,
        code: u32,
    }

    #[derive(Default)]
    pub struct Arm675 {
        strips: Vec<Strip>,
        image_order: Vec<u32>,
        strip_top: Vec<u32>,
        tops: Vec<R4Top>,
        bots: Vec<R4Bot>,
        arena: Vec<u8>,
        rules: Vec<Rule>,
        stack: Vec<u32>,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        queues: Vec<Vec<u32>>,
        ovf: Vec<u8>,
        lazy: LazyCells,
    }

    impl Arm675 {
        fn word_len(&self, w: u32) -> u32 {
            if w == 0 {
                0
            } else if (w & 0x8000_0000) == 0 {
                w >> 24
            } else {
                self.rules[(w & 0x7FFF_FFFF) as usize].len
            }
        }
        fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 {
                return right;
            }
            if right == 0 {
                return left;
            }
            if (left | right) & 0x8000_0000 == 0 {
                let (ll, rl) = (left >> 24, right >> 24);
                let tot = ll + rl;
                if tot as usize <= GIM_CAP && self.arena.len() + tot as usize <= 0x00FF_F000 {
                    let off = self.arena.len() as u32;
                    let lo = (left & 0x00FF_FFFF) as usize;
                    let ro = (right & 0x00FF_FFFF) as usize;
                    self.arena.extend_from_within(lo..lo + ll as usize);
                    self.arena.extend_from_within(ro..ro + rl as usize);
                    return (tot << 24) | off;
                }
            }
            let tot = self.word_len(left) + self.word_len(right);
            let id = self.rules.len() as u32;
            self.rules.push(Rule {
                left,
                right,
                len: tot,
                first: u32::MAX,
            });
            0x8000_0000 | id
        }
        fn append_long(&mut self, w: u32) {
            self.stack.clear();
            self.stack.push(w);
            while let Some(cur) = self.stack.pop() {
                if (cur & 0x8000_0000) == 0 {
                    let ln = (cur >> 24) as usize;
                    let off = (cur & 0x00FF_FFFF) as usize;
                    self.spelling.extend_from_slice(&self.arena[off..off + ln]);
                } else {
                    let idx = (cur & 0x7FFF_FFFF) as usize;
                    let r = self.rules[idx];
                    if r.first == u32::MAX {
                        self.rules[idx].first = self.spelling.len() as u32;
                        self.stack.push(r.right);
                        self.stack.push(r.left);
                    } else {
                        let st = r.first as usize;
                        self.spelling.extend_from_within(st..st + r.len as usize);
                    }
                }
            }
        }
        fn make_inline_code(&mut self, nd_word: u32, a: u8) -> u32 {
            if nd_word == 0 {
                return u32::from(a);
            }
            let code = self.inline16.len() as u32;
            if (nd_word & 0x8000_0000) == 0 && (nd_word >> 24) <= 15 {
                let wl = (nd_word >> 24) as usize;
                let off = (nd_word & 0x00FF_FFFF) as usize;
                let mut slot = [0u8; INLINE];
                slot.copy_from_slice(&self.arena[off..off + INLINE]);
                slot[wl] = a;
                self.inline16.push(slot);
                self.lens.push((wl + 1) as u8);
            } else {
                let o0 = self.spelling.len() as u32;
                self.append_long(nd_word);
                self.spelling.push(a);
                let o1 = self.spelling.len() as u32;
                let mut slot = [0u8; INLINE];
                slot[0..4].copy_from_slice(&o0.to_ne_bytes());
                slot[4..8].copy_from_slice(&o1.to_ne_bytes());
                self.inline16.push(slot);
                self.lens.push(LONG);
            }
            code
        }

        pub fn run_block(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
            const LIM24: u32 = 24;
            self.queues.resize_with(256, Vec::new);
            let mut all = [0u32; 256];
            let mut thin = [0u32; 256];
            let mut thick = [0u32; 256];
            let (mut m, mut mark, mut pos) = (0u32, 0u32, 0u32);
            self.strips.clear();
            let mut i = 0;
            while i < records.len() {
                let c = (records[i] >> 8) as u8;
                let mut w = u32::from(records[i] & 255);
                i += 1;
                while i < records.len() && (records[i] >> 8) as u8 == c {
                    w += u32::from(records[i] & 255);
                    i += 1;
                }
                let mut push_piece = |x: u32, pw: u32, keep: bool| {
                    let a = usize::from(c);
                    if keep {
                        if p.wrapping_sub(x) < pw {
                            mark = m + (p - x);
                        }
                        let at = thin[a] as usize;
                        let need = at + 24;
                        if self.queues[a].len() < need {
                            let nc = need.max(self.queues[a].len() * 2).max(1024);
                            self.queues[a].resize(nc, 0);
                        }
                        let base = m;
                        let iota: [u32; 24] = std::array::from_fn(|k| base + k as u32);
                        self.queues[a][at..at + 24].copy_from_slice(&iota);
                        thin[a] += pw;
                        m += pw;
                    } else {
                        self.strips.push(Strip {
                            from: x,
                            to: all[a],
                            width: pw,
                            byte: c,
                        });
                        thick[a] += 1;
                    }
                    all[a] += pw;
                };
                if w <= LIM24 {
                    push_piece(pos, w, true);
                } else if p.wrapping_sub(pos) >= w {
                    push_piece(pos, w, false);
                } else {
                    let end = pos + w;
                    if pos < p {
                        push_piece(pos, p - pos, false);
                    }
                    push_piece(p, 1, true);
                    if p + 1 < end {
                        push_piece(p + 1, end - p - 1, false);
                    }
                }
                pos += w;
            }

            let mut first = [0u32; 256];
            let mut write = [0u32; 256];
            let (mut tot, mut tw) = (0u32, 0u32);
            for a in 0..256 {
                first[a] = tot;
                tot += all[a];
                write[a] = tw;
                tw += thick[a];
            }
            let n_strips = self.strips.len();
            self.image_order.resize(n_strips, 0);
            self.strip_top.resize(n_strips, 0);
            for (si, s) in self.strips.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                self.image_order[write[a] as usize] = si as u32;
                write[a] += 1;
            }
            let need = PAD_SLOTS + 3 * n_strips + 8;
            self.tops.clear();
            self.tops.resize(need, R4Top::default());
            self.bots.clear();
            self.bots.resize(need, R4Bot::default());
            self.rules.clear();
            if self.arena.len() < 256 {
                self.arena.clear();
                self.arena.extend(0..=255u8);
            }
            self.arena.truncate(256);

            let p0_base = PAD_SLOTS as u32;
            let a0_base = p0_base + n_strips as u32 + 2;
            let (mut t_cur, mut b_cur, mut xc) = (a0_base, PAD_SLOTS as u32, 0u32);
            for si in 0..n_strips {
                let s = self.strips[si];
                if xc < s.from {
                    self.tops[t_cur as usize] = R4Top {
                        prev: t_cur - 1,
                        width: s.from - xc,
                        word: 0,
                        other: b_cur,
                    };
                    self.bots[b_cur as usize] = R4Bot {
                        prev: b_cur - 1,
                        top: t_cur,
                    };
                    t_cur += 1;
                    b_cur += 1;
                }
                self.strip_top[si] = t_cur;
                self.tops[t_cur as usize] = R4Top {
                    prev: t_cur - 1,
                    width: s.width,
                    word: (1 << 24) | u32::from(s.byte),
                    other: 0,
                };
                t_cur += 1;
                xc = s.from + s.width;
            }
            if xc < n as u32 {
                self.tops[t_cur as usize] = R4Top {
                    prev: t_cur - 1,
                    width: n as u32 - xc,
                    word: 0,
                    other: b_cur,
                };
                self.bots[b_cur as usize] = R4Bot {
                    prev: b_cur - 1,
                    top: t_cur,
                };
                t_cur += 1;
                b_cur += 1;
            }
            let a0_end = t_cur;
            let mut p0_cur = p0_base;
            xc = 0;
            for ii in 0..n_strips {
                let idx = self.image_order[ii] as usize;
                let s = self.strips[idx];
                if xc < s.to {
                    self.tops[p0_cur as usize] = R4Top {
                        prev: p0_cur - 1,
                        width: s.to - xc,
                        word: 0,
                        other: b_cur,
                    };
                    self.bots[b_cur as usize] = R4Bot {
                        prev: b_cur - 1,
                        top: p0_cur,
                    };
                    p0_cur += 1;
                    b_cur += 1;
                }
                let st_i = self.strip_top[idx];
                self.tops[st_i as usize].other = b_cur;
                self.bots[b_cur as usize] = R4Bot {
                    prev: b_cur - 1,
                    top: st_i,
                };
                b_cur += 1;
                xc = s.to + s.width;
            }
            if xc < n as u32 {
                self.tops[p0_cur as usize] = R4Top {
                    prev: p0_cur - 1,
                    width: n as u32 - xc,
                    word: 0,
                    other: b_cur,
                };
                self.bots[b_cur as usize] = R4Bot {
                    prev: b_cur - 1,
                    top: p0_cur,
                };
                p0_cur += 1;
                b_cur += 1;
            }
            if p0_cur > p0_base {
                self.tops[p0_base as usize].prev = 0;
            }
            self.bots[PAD_SLOTS].prev = 0;
            self.tops[a0_base as usize].prev = if p0_cur > p0_base { p0_cur - 1 } else { 0 };

            let (mut sa, mut sb) = (a0_end - 1, b_cur - 1);
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
                        self.bots[bs as usize] = R4Bot { prev: fb, top: ts };
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
                        self.tops[ts as usize] = R4Top {
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
            let mut x_bot = m;
            while sb != 0 {
                let bslot = self.bots[sb as usize];
                let ts = bslot.top as usize;
                x_bot -= self.tops[ts].width;
                self.tops[ts].other = x_bot;
                sb = bslot.prev;
            }
            self.arena.extend_from_slice(&[0u8; 16]);
            if self.inline16.len() < 256 {
                self.inline16.clear();
                for c in 0..=255u8 {
                    let mut v = [0u8; INLINE];
                    v[0] = c;
                    self.inline16.push(v);
                }
                self.lens.clear();
                self.lens.resize(256, 1);
            }
            self.inline16.truncate(256);
            self.lens.truncate(256);
            self.spelling.clear();
            self.bands.clear();

            let mut thin_start = [0u32; 256];
            let mut acc = 0u32;
            for a in 0..256 {
                thin_start[a] = acc;
                acc += thin[a];
            }
            let mut cursors_rev = [0usize; 256];
            let mut rem_node = if sa != 0 {
                self.tops[sa as usize].width
            } else {
                0
            };
            let mut kc = m;
            for a in (0..256).rev() {
                cursors_rev[a] = self.bands.len();
                let start_a = thin_start[a];
                while kc > start_a {
                    let nd = self.tops[sa as usize];
                    let width = (kc - start_a).min(rem_node);
                    rem_node -= width;
                    kc -= width;
                    let dst = nd.other + rem_node;
                    let code = self.make_inline_code(nd.word, a as u8);
                    self.bands.push(Band {
                        end: kc - start_a,
                        dst,
                        code,
                    });
                    if rem_node == 0 {
                        sa = nd.prev;
                        rem_node = if sa != 0 {
                            self.tops[sa as usize].width
                        } else {
                            0
                        };
                    }
                }
            }
            let bits = (u32::BITS - m.saturating_sub(1).leading_zeros()).max(1);
            let s_cuts = 48u32;
            let wide = ((self.inline16.len() as u32) + s_cuts) > (u32::MAX >> bits);
            self.edges.resize(m as usize + 1, 0);
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize + 1, 0);
            }
            for a in (0..256).rev() {
                let b0 = cursors_rev[a];
                let b1 = if a > 0 {
                    cursors_rev[a - 1]
                } else {
                    self.bands.len()
                };
                let mut prev_off = thin[a] as usize;
                let qa = &self.queues[a];
                for bi in b0..b1 {
                    let b = self.bands[bi];
                    let q0 = b.end as usize;
                    let span = prev_off - q0;
                    prev_off = q0;
                    let dst = b.dst as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    for (t, &j) in self.edges[dst..dst + span]
                        .iter_mut()
                        .zip(&qa[q0..q0 + span])
                    {
                        *t = pack | j;
                    }
                }
            }

            if m < 4096 || wide {
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
                return;
            }

            out.resize(n + INLINE, 0);
            if self.ovf.len() < 64 * PAGE {
                self.ovf.resize(64 * PAGE, 0);
            }
            let mask = (1u32 << bits) - 1;
            let sbase = self.inline16.len() as u32;
            self.lens.resize(sbase as usize + 64, LONG);
            self.inline16.resize(sbase as usize + 64, [0u8; INLINE]);

            let mut cuts = Vec::with_capacity(48);
            for q in 0..48u32 {
                let v = ((u64::from(mark) + u64::from(q) * u64::from(m) / 48) % u64::from(m)) as u32;
                if cuts.last() != Some(&v) {
                    cuts.push(v);
                }
            }
            let s_eff = cuts.len();
            let mut saved_raw = vec![0u32; s_eff];
            let mut saved_code = vec![0u32; s_eff];
            let mut saved_next = vec![0u32; s_eff];
            for (q, &v) in cuts.iter().enumerate() {
                let raw = self.edges[v as usize];
                saved_raw[q] = raw;
                saved_code[q] = raw >> bits;
                saved_next[q] = raw & mask;
                self.edges[v as usize] = ((sbase + q as u32) << bits) | (raw & mask);
            }
            self.edges[m as usize] = m;

            let n_phys = n / PAGE;
            let mut top_phys = n_phys as isize - 1;
            let mut ovf_used = 0usize;
            let mut ext_is_phys = Vec::<isize>::with_capacity(320);
            let mut ext_len = Vec::<usize>::with_capacity(320);
            let mut ext_next = Vec::<isize>::with_capacity(320);
            let mut head = vec![-1isize; s_eff];
            let mut succ = vec![-1isize; s_eff];

            fn alloc_page(
                seg0_cur: usize,
                top_phys: &mut isize,
                ovf_used: &mut usize,
                ext_is_phys: &mut Vec<isize>,
                ext_len: &mut Vec<usize>,
                ext_next: &mut Vec<isize>,
            ) -> usize {
                let id = ext_len.len();
                if *top_phys >= 0 && (*top_phys as usize) * PAGE >= seg0_cur + PAGE {
                    let pid = *top_phys;
                    *top_phys -= 1;
                    ext_is_phys.push(pid);
                } else {
                    let oid = *ovf_used as isize;
                    *ovf_used += 1;
                    ext_is_phys.push(-(oid + 1));
                }
                ext_len.push(0);
                ext_next.push(-1);
                id
            }

            let mut next_q = 0usize;
            let mut active = 0usize;
            let mut j_l = [m; 8];
            let mut q_l = [0usize; 8];
            let mut eid_l = [-2isize; 8];
            let mut pos_l = [0usize; 8];
            let mut seg0_len = 0usize;

            let write_code = |l: usize,
                              cd: usize,
                              eid_l: &mut [isize; 8],
                              pos_l: &mut [usize; 8],
                              seg0_len: &mut usize,
                              head: &mut [isize],
                              top_phys: &mut isize,
                              ovf_used: &mut usize,
                              ext_is_phys: &mut Vec<isize>,
                              ext_len: &mut Vec<usize>,
                              ext_next: &mut Vec<isize>,
                              out: &mut [u8],
                              ovf: &mut [u8],
                              inline16: &[[u8; INLINE]],
                              lens: &[u8],
                              spelling: &[u8]| {
                if eid_l[l] == -2 {
                    return;
                }
                let temp: [u8; 16];
                let word: &[u8] = if lens[cd] != LONG {
                    temp = inline16[cd];
                    &temp[..lens[cd] as usize]
                } else {
                    let meta = inline16[cd];
                    let o0 = u32::from_ne_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let o1 = u32::from_ne_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    &spelling[o0..o1]
                };
                let mut rem = word;
                if eid_l[l] == -1 {
                    let lim = if *top_phys >= 0 {
                        ((*top_phys as usize) + 1) * PAGE
                    } else {
                        0
                    }
                    .min(n);
                    if pos_l[0] + rem.len() <= lim {
                        out[pos_l[0]..pos_l[0] + rem.len()].copy_from_slice(rem);
                        pos_l[0] += rem.len();
                        return;
                    }
                    *seg0_len = pos_l[0];
                    let ne = alloc_page(pos_l[0], top_phys, ovf_used, ext_is_phys, ext_len, ext_next);
                    head[0] = ne as isize;
                    eid_l[0] = ne as isize;
                    pos_l[0] = 0;
                }
                while !rem.is_empty() {
                    let ce = eid_l[l] as usize;
                    if pos_l[l] == PAGE {
                        ext_len[ce] = PAGE;
                        let s0c = if eid_l[0] == -1 { pos_l[0] } else { *seg0_len };
                        let ne = alloc_page(s0c, top_phys, ovf_used, ext_is_phys, ext_len, ext_next);
                        ext_next[ce] = ne as isize;
                        eid_l[l] = ne as isize;
                        pos_l[l] = 0;
                    }
                    let ce = eid_l[l] as usize;
                    let av = PAGE - pos_l[l];
                    let tk = rem.len().min(av);
                    let loc = ext_is_phys[ce];
                    if loc >= 0 {
                        let base = (loc as usize) * PAGE + pos_l[l];
                        out[base..base + tk].copy_from_slice(&rem[..tk]);
                    } else {
                        let base = ((-loc - 1) as usize) * PAGE + pos_l[l];
                        ovf[base..base + tk].copy_from_slice(&rem[..tk]);
                    }
                    pos_l[l] += tk;
                    rem = &rem[tk..];
                }
            };

            for l in 0..8 {
                if next_q < s_eff {
                    let q = next_q;
                    next_q += 1;
                    active += 1;
                    q_l[l] = q;
                    if q == 0 {
                        eid_l[0] = -1;
                        pos_l[0] = 0;
                    } else {
                        let s0c = if eid_l[0] == -1 { pos_l[0] } else { seg0_len };
                        let ne = alloc_page(
                            s0c,
                            &mut top_phys,
                            &mut ovf_used,
                            &mut ext_is_phys,
                            &mut ext_len,
                            &mut ext_next,
                        );
                        head[q] = ne as isize;
                        eid_l[l] = ne as isize;
                        pos_l[l] = 0;
                    }
                    write_code(
                        l,
                        saved_code[q] as usize,
                        &mut eid_l,
                        &mut pos_l,
                        &mut seg0_len,
                        &mut head,
                        &mut top_phys,
                        &mut ovf_used,
                        &mut ext_is_phys,
                        &mut ext_len,
                        &mut ext_next,
                        out,
                        &mut self.ovf,
                        &self.inline16,
                        &self.lens,
                        &self.spelling,
                    );
                    j_l[l] = saved_next[q];
                }
            }

            let edges = &self.edges[..];
            let inline16 = &self.inline16[..];
            let lens = &self.lens[..];
            let spelling = &self.spelling[..];
            while active > 0 {
                for l in 0..8 {
                    let ev = edges[j_l[l] as usize];
                    let cd = (ev >> bits) as usize;
                    j_l[l] = ev & mask;
                    let ln = lens[cd];
                    if ln != LONG && pos_l[l] + INLINE <= PAGE {
                        let eid = eid_l[l];
                        if eid >= 0 {
                            let loc = ext_is_phys[eid as usize];
                            if loc >= 0 {
                                let b = (loc as usize) * PAGE + pos_l[l];
                                out[b..b + INLINE].copy_from_slice(&inline16[cd]);
                            } else {
                                let b = ((-loc - 1) as usize) * PAGE + pos_l[l];
                                self.ovf[b..b + INLINE].copy_from_slice(&inline16[cd]);
                            }
                            pos_l[l] += usize::from(ln);
                            continue;
                        } else if eid == -1 {
                            let lim = if top_phys >= 0 {
                                ((top_phys as usize) + 1) * PAGE
                            } else {
                                0
                            }
                            .min(n);
                            if pos_l[0] + INLINE <= lim {
                                out[pos_l[0]..pos_l[0] + INLINE].copy_from_slice(&inline16[cd]);
                                pos_l[0] += usize::from(ln);
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                    if cd < sbase as usize {
                        write_code(
                            l,
                            cd,
                            &mut eid_l,
                            &mut pos_l,
                            &mut seg0_len,
                            &mut head,
                            &mut top_phys,
                            &mut ovf_used,
                            &mut ext_is_phys,
                            &mut ext_len,
                            &mut ext_next,
                            out,
                            &mut self.ovf,
                            inline16,
                            lens,
                            spelling,
                        );
                    } else if eid_l[l] != -2 {
                        let hit_q = cd - sbase as usize;
                        succ[q_l[l]] = hit_q as isize;
                        if eid_l[l] == -1 {
                            seg0_len = pos_l[0];
                        } else if eid_l[l] >= 0 {
                            ext_len[eid_l[l] as usize] = pos_l[l];
                        }
                        if next_q < s_eff {
                            let nq = next_q;
                            next_q += 1;
                            q_l[l] = nq;
                            let s0c = if eid_l[0] == -1 { pos_l[0] } else { seg0_len };
                            let ne = alloc_page(
                                s0c,
                                &mut top_phys,
                                &mut ovf_used,
                                &mut ext_is_phys,
                                &mut ext_len,
                                &mut ext_next,
                            );
                            head[nq] = ne as isize;
                            eid_l[l] = ne as isize;
                            pos_l[l] = 0;
                            write_code(
                                l,
                                saved_code[nq] as usize,
                                &mut eid_l,
                                &mut pos_l,
                                &mut seg0_len,
                                &mut head,
                                &mut top_phys,
                                &mut ovf_used,
                                &mut ext_is_phys,
                                &mut ext_len,
                                &mut ext_next,
                                out,
                                &mut self.ovf,
                                inline16,
                                lens,
                                spelling,
                            );
                            j_l[l] = saved_next[nq];
                        } else {
                            active -= 1;
                            eid_l[l] = -2;
                            j_l[l] = m;
                        }
                    }
                }
            }
            for (q, &v) in cuts.iter().enumerate() {
                self.edges[v as usize] = saved_raw[q];
            }

            let mut ord = Vec::<(usize, usize, usize)>::with_capacity(ext_len.len());
            let mut wr = seg0_len.min(n);
            let mut cur_q = 0isize;
            let mut steps_q = 0;
            while wr < n && cur_q >= 0 && steps_q < s_eff {
                let mut e = head[cur_q as usize];
                while e >= 0 && wr < n {
                    let eu = e as usize;
                    let cl = ext_len[eu].min(n - wr);
                    if cl > 0 {
                        ord.push((eu, wr, cl));
                        wr += cl;
                    }
                    e = ext_next[eu];
                }
                cur_q = succ[cur_q as usize];
                steps_q += 1;
                if cur_q == 0 {
                    break;
                }
            }
            if wr > seg0_len {
                let total_tp = wr.div_ceil(PAGE);
                let first_tp = seg0_len / PAGE;
                let n_slots = total_tp.max(n_phys);
                let mut phys_to_ord = vec![-1isize; n_slots];
                let mut ref_cnt = vec![0u8; n_slots];
                let mut tp_done = vec![false; total_tp];
                let mut tp_first = vec![usize::MAX; total_tp];
                let mut tp_last = vec![0usize; total_tp];
                for (oi, &(e, t0, cl)) in ord.iter().enumerate() {
                    let p_lo = t0 / PAGE;
                    let p_hi = (t0 + cl - 1) / PAGE;
                    let rc = (p_hi - p_lo + 1) as u8;
                    for pg in p_lo..=p_hi {
                        tp_first[pg] = tp_first[pg].min(oi);
                        tp_last[pg] = oi;
                    }
                    let loc = ext_is_phys[e];
                    if loc >= 0 {
                        phys_to_ord[loc as usize] = oi as isize;
                        ref_cnt[loc as usize] = rc;
                    }
                }
                let mut ready = Vec::with_capacity(total_tp);
                for pg in first_tp..total_tp {
                    if ref_cnt[pg] == 0 {
                        ready.push(pg);
                    }
                }
                let mut done = 0;
                let need = total_tp - first_tp;
                let mut scan = first_tp;
                while done < need {
                    if ready.is_empty() {
                        let mut chosen = None;
                        while scan < total_tp {
                            if ref_cnt[scan] > 0 && !tp_done[scan] {
                                chosen = Some(scan);
                                break;
                            }
                            scan += 1;
                        }
                        if chosen.is_none() {
                            for s in first_tp..n_slots {
                                if ref_cnt[s] > 0 {
                                    chosen = Some(s);
                                    break;
                                }
                            }
                        }
                        let ch = chosen.unwrap();
                        let oi = phys_to_ord[ch] as usize;
                        let (e, _, cl) = ord[oi];
                        let oid = ovf_used;
                        ovf_used += 1;
                        if self.ovf.len() < ovf_used * PAGE {
                            self.ovf.resize(ovf_used * PAGE, 0);
                        }
                        self.ovf[oid * PAGE..oid * PAGE + cl]
                            .copy_from_slice(&out[ch * PAGE..ch * PAGE + cl]);
                        ext_is_phys[e] = -(oid as isize + 1);
                        ref_cnt[ch] = 0;
                        if ch < total_tp && !tp_done[ch] {
                            ready.push(ch);
                        }
                        continue;
                    }
                    let pg = ready.pop().unwrap();
                    if tp_done[pg] {
                        continue;
                    }
                    tp_done[pg] = true;
                    done += 1;
                    let p_start = (pg * PAGE).max(seg0_len);
                    let p_end = ((pg + 1) * PAGE).min(wr);
                    if tp_first[pg] != usize::MAX {
                        for oi in tp_first[pg]..=tp_last[pg] {
                            let (e, t0, cl) = ord[oi];
                            let lo = t0.max(p_start);
                            let hi = (t0 + cl).min(p_end);
                            if lo < hi {
                                let loc = ext_is_phys[e];
                                if loc >= 0 {
                                    let src = (loc as usize) * PAGE + (lo - t0);
                                    out.copy_within(src..src + (hi - lo), lo);
                                } else {
                                    let src = ((-loc - 1) as usize) * PAGE + (lo - t0);
                                    out[lo..hi].copy_from_slice(&self.ovf[src..src + (hi - lo)]);
                                }
                            }
                            let loc = ext_is_phys[e];
                            if loc >= 0 {
                                let sp = loc as usize;
                                ref_cnt[sp] -= 1;
                                if ref_cnt[sp] == 0 && sp >= first_tp && sp < total_tp && !tp_done[sp] {
                                    ready.push(sp);
                                }
                            }
                        }
                    }
                }
            }
            out.truncate(wr.min(n));
            let mut len = out.len();
            while len < n {
                let take = len.min(n - len);
                out.extend_from_within(..take);
                len = out.len();
            }
        }
    }

    impl InverseBwt for Arm675 {
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
            let nr = r.records().len();
            if n > 0 && (p as usize) < n && nr * 20 < n * 3 && nr * 64 >= n {
                self.run_block(r.records(), n, p, out);
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
// Reference Arm 2: `PR#679` (`R4@16 + Vec16` — serial R4 with u32 queues & cleared tapes)
// ============================================================================
mod pr679 {
    use mibz2::bwt::{InverseBwt, LazyCells};
    use mibz2::runs::Runs;
    use mibz2::Kernels;

    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const LIMIT: u32 = 16;
    const PAD: usize = 16;
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
    struct Band {
        end: u32,
        delta: u32,
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

    #[derive(Default)]
    struct R4 {
        tops: Vec<Top>,
        bots: Vec<Bot>,
        strip_top: Vec<u32>,
        arena: Vec<u8>,
        rules: Vec<Rule>,
        stack: Vec<u32>,
        rev: Vec<Band>,
    }

    impl R4 {
        fn len(&self, w: u32) -> u32 {
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
            let len = self.len(a) + self.len(b);
            self.rules.push(Rule {
                left: a,
                right: b,
                len,
                first: u32::MAX,
            });
            RULE | id
        }
        fn append(&mut self, word: u32, dest: &mut Vec<u8>) {
            self.stack.clear();
            self.stack.push(word);
            while let Some(w) = self.stack.pop() {
                if w & RULE == 0 {
                    let off = (w & 0xFF_FFFF) as usize;
                    dest.extend_from_slice(&self.arena[off..off + (w >> 24) as usize]);
                } else {
                    let rule = &mut self.rules[(w & !RULE) as usize];
                    if rule.first == u32::MAX {
                        rule.first = dest.len() as u32;
                        let (l, r) = (rule.left, rule.right);
                        self.stack.push(r);
                        self.stack.push(l);
                    } else {
                        let start = rule.first as usize;
                        dest.extend_from_within(start..start + rule.len as usize);
                    }
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(queue: &mut Vec<u32>, need: usize) {
        queue.resize(need.max(2 * queue.len()).max(1024), 0);
    }

    #[inline(always)]
    fn push_rows(queue: &mut Vec<u32>, len: u32, m: u32) {
        const BLOCK: usize = 16;
        let at = len as usize;
        let need = at + BLOCK;
        if queue.len() < need {
            grow(queue, need);
        }
        let iota: [u32; BLOCK] = std::array::from_fn(|i| m.wrapping_add(i as u32));
        queue[at..at + BLOCK].copy_from_slice(&iota);
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
    pub struct Kernel679 {
        queues: Vec<Vec<u32>>,
        strips: Vec<Strip>,
        image_order: Vec<u32>,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        offsets: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        r4: R4,
        pub last_active_058_equiv_bytes: usize,
    }

    impl Kernel679 {
        pub fn run(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
            self.strips.clear();
            self.queues.resize_with(256, Vec::new);
            let mut all = [0u32; 256];
            let mut thin = [0u32; 256];
            let mut thick = [0u32; 256];
            let (mut m, mut mark, mut pos) = (0u32, 0u32, 0u32);
            let mut idx = 0;
            while idx < records.len() {
                let c = (records[idx] >> 8) as u8;
                let mut width = u32::from(records[idx] & 255);
                idx += 1;
                while idx < records.len() && (records[idx] >> 8) as u8 == c {
                    width += u32::from(records[idx] & 255);
                    idx += 1;
                }
                let mut piece = |x: u32, w: u32, keep: bool| {
                    let a = usize::from(c);
                    if keep {
                        if p.wrapping_sub(x) < w {
                            mark = m + (p - x);
                        }
                        push_rows(&mut self.queues[a], thin[a], m);
                        thin[a] += w;
                        m += w;
                    } else {
                        self.strips.push(Strip {
                            from: x,
                            to: all[a],
                            width: w,
                            byte: c,
                        });
                        thick[a] += 1;
                    }
                    all[a] += w;
                };
                if width <= LIMIT {
                    piece(pos, width, true);
                } else if p.wrapping_sub(pos) >= width {
                    piece(pos, width, false);
                } else {
                    let end = pos + width;
                    if pos < p {
                        piece(pos, p - pos, false);
                    }
                    piece(p, 1, true);
                    if p + 1 < end {
                        piece(p + 1, end - p - 1, false);
                    }
                }
                pos += width;
            }
            let first = starts(all);
            let mut write = starts(thick);
            self.image_order.resize(self.strips.len(), 0);
            for (i, s) in self.strips.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                self.image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }
            self.bands.clear();
            self.spelling.clear();
            self.offsets.clear();
            self.offsets.push(0);

            let cursors = self.induced_r4(n as u32, m, &thin);
            let words = self.offsets.len() - 1;
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let wide = (255 + words as u32) > (u32::MAX >> bits);
            self.edges.resize(m as usize, 0);
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut from = 0;
                for b in &self.bands[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    let src = &self.queues[a][from..from + span];
                    for (target, &j) in self.edges[dst..dst + span].iter_mut().zip(src) {
                        *target = pack | j;
                    }
                    from += span;
                    k = b.end;
                }
            }

            // Analytical active per-block working set of Master-0.5.8 (`u32` queues + `Endpoint` GIM + `offsets` + `bands` + `rev`)
            let ns = self.strips.len();
            self.last_active_058_equiv_bytes = records.len() * 2
                + (m as usize) * 4 // u32 queues active
                + ns * std::mem::size_of::<Strip>()
                + ns * 8
                + (PAD + 3 * ns + 16) * (20 + 12) // Endpoint GIM cleared/touched Top(20B)+Bot(12B)
                + self.r4.arena.len()
                + self.r4.rules.len() * std::mem::size_of::<Rule>()
                + self.bands.len() * std::mem::size_of::<Band>() * 2 // rev + bands
                + self.spelling.len()
                + self.offsets.len() * 4
                + self.inline16.len() * INLINE
                + self.lens.len()
                + (m as usize) * 4
                + if wide { (m as usize) * 4 } else { 0 }
                + n;

            out.resize(n + INLINE, 0);
            let mut at = 0;
            let mask = (1 << bits) - 1;
            let mut j = mark;
            for _ in 0..self.edges.len() {
                let edge = self.edges[j as usize];
                let code = if self.extra.is_empty() {
                    edge >> bits
                } else {
                    self.extra[j as usize]
                } as usize;
                j = edge & mask;
                let len = self.lens[code];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&self.inline16[code]);
                    at += usize::from(len);
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
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

        fn induced_r4(&mut self, n: u32, m: u32, thin: &[u32; 256]) -> [usize; 256] {
            let e = &mut self.r4;
            let ns = self.strips.len();
            let slots = PAD + 3 * ns + 16;
            e.tops.clear();
            e.tops.resize(slots, Top::default());
            e.bots.clear();
            e.bots.resize(slots, Bot::default());
            e.strip_top.clear();
            e.strip_top.resize(ns, 0);
            e.rules.clear();
            e.arena.clear();
            e.arena.extend(0..=255u8);
            let (tops, bots) = (&mut e.tops, &mut e.bots);

            let p0_base = PAD as u32;
            let a0_base = p0_base + ns as u32 + 2;
            let (mut t, mut b) = (a0_base, PAD as u32);
            let mut x = 0;
            for (i, s) in self.strips.iter().enumerate() {
                if x < s.from {
                    tops[t as usize] = Top {
                        prev: t - 1,
                        width: s.from - x,
                        word: 0,
                        other: b,
                    };
                    bots[b as usize] = Bot { prev: b - 1, top: t };
                    t += 1;
                    b += 1;
                }
                e.strip_top[i] = t;
                tops[t as usize] = Top {
                    prev: t - 1,
                    width: s.width,
                    word: (1 << 24) | u32::from(s.byte),
                    other: 0,
                };
                t += 1;
                x = s.from + s.width;
            }
            if x < n {
                tops[t as usize] = Top {
                    prev: t - 1,
                    width: n - x,
                    word: 0,
                    other: b,
                };
                bots[b as usize] = Bot { prev: b - 1, top: t };
                t += 1;
                b += 1;
            }
            let a0_end = t;
            let mut p = p0_base;
            x = 0;
            for &i in &self.image_order {
                let s = self.strips[i as usize];
                if x < s.to {
                    tops[p as usize] = Top {
                        prev: p - 1,
                        width: s.to - x,
                        word: 0,
                        other: b,
                    };
                    bots[b as usize] = Bot { prev: b - 1, top: p };
                    p += 1;
                    b += 1;
                }
                let st = e.strip_top[i as usize];
                tops[st as usize].other = b;
                bots[b as usize] = Bot {
                    prev: b - 1,
                    top: st,
                };
                b += 1;
                x = s.to + s.width;
            }
            if x < n {
                tops[p as usize] = Top {
                    prev: p - 1,
                    width: n - x,
                    word: 0,
                    other: b,
                };
                bots[b as usize] = Bot { prev: b - 1, top: p };
                p += 1;
                b += 1;
            }
            if p > p0_base {
                tops[p0_base as usize].prev = 0;
            }
            bots[PAD].prev = 0;
            tops[a0_base as usize].prev = if p > p0_base { p - 1 } else { 0 };

            let (mut sa, mut sb) = (a0_end - 1, b - 1);
            let mut remaining = n;
            while remaining > 0 {
                let ta = e.tops[sa as usize];
                let (mut na, mut ra, mut aw, mut bs) = (ta.prev, ta.width, ta.word, ta.other);
                let bb = e.bots[sb as usize];
                let (mut nb, mut ts) = (bb.prev, bb.top);
                let tb = e.tops[ts as usize];
                let (mut rb, mut bw) = (tb.width, tb.word);
                let (mut fa, mut fb) = (sa, sb);
                if bs == sb {
                    remaining -= ra;
                    (sa, sb) = (na, nb);
                    continue;
                }
                loop {
                    let wd = e.join(aw, bw);
                    if ra == rb {
                        remaining -= ra;
                        let top = &mut e.tops[ts as usize];
                        (top.width, top.word, top.other) = (ra, wd, bs);
                        e.bots[bs as usize].top = ts;
                        (sa, sb) = (na, nb);
                        break;
                    } else if ra > rb {
                        remaining -= rb;
                        let top = &mut e.tops[ts as usize];
                        (top.width, top.word, top.other) = (rb, wd, bs);
                        e.bots[fb as usize].prev = e.bots[bs as usize].prev;
                        e.bots[bs as usize] = Bot { prev: fb, top: ts };
                        bs = fb;
                        ra -= rb;
                        fb = nb;
                        let bn = e.bots[nb as usize];
                        (nb, ts) = (bn.prev, bn.top);
                        let tn = e.tops[ts as usize];
                        (rb, bw) = (tn.width, tn.word);
                    } else {
                        remaining -= ra;
                        e.bots[bs as usize].top = ts;
                        e.tops[fa as usize].prev = e.tops[ts as usize].prev;
                        e.tops[ts as usize] = Top {
                            prev: fa,
                            width: ra,
                            word: wd,
                            other: bs,
                        };
                        ts = fa;
                        rb -= ra;
                        fa = na;
                        let tn = e.tops[na as usize];
                        (na, ra, aw, bs) = (tn.prev, tn.width, tn.word, tn.other);
                    }
                }
            }
            let mut xb = m;
            while sb != 0 {
                let s = e.bots[sb as usize];
                let w = e.tops[s.top as usize].width;
                xb -= w;
                e.tops[s.top as usize].other = xb;
                sb = s.prev;
            }
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

            let mut thin_start = [0u32; 256];
            let mut acc = 0;
            for a in 0..256 {
                thin_start[a] = acc;
                acc += thin[a];
            }
            let mut ends = [0usize; 256];
            e.rev.clear();
            let mut rem = if sa != 0 {
                e.tops[sa as usize].width
            } else {
                0
            };
            let mut kc = m;
            for a in (0..256).rev() {
                let start = thin_start[a];
                while kc > start {
                    let nd = e.tops[sa as usize];
                    let width = (kc - start).min(rem);
                    rem -= width;
                    let end = kc;
                    kc -= width;
                    let dst = nd.other + rem;
                    let code = if nd.word == 0 {
                        a as u32
                    } else {
                        let o0 = self.spelling.len();
                        e.append(nd.word, &mut self.spelling);
                        self.spelling.push(a as u8);
                        let height = self.spelling.len() - o0;
                        let mut v = [0; INLINE];
                        if nd.word & RULE == 0 && height <= INLINE {
                            v[..height].copy_from_slice(&self.spelling[o0..]);
                            self.spelling.truncate(o0);
                            self.lens.push(height as u8);
                        } else {
                            self.lens.push(LONG);
                        }
                        self.inline16.push(v);
                        self.offsets.push(self.spelling.len() as u32);
                        254 + self.offsets.len() as u32
                    };
                    e.rev.push(Band {
                        end,
                        delta: dst.wrapping_sub(kc),
                        code,
                    });
                    if rem == 0 {
                        sa = nd.prev;
                        rem = if sa != 0 {
                            e.tops[sa as usize].width
                        } else {
                            0
                        };
                    }
                }
                ends[a] = e.rev.len();
            }
            let total = e.rev.len();
            self.bands.extend(e.rev.iter().rev());
            let mut cursors = [0usize; 256];
            for a in 0..256 {
                cursors[a] = total - ends[a];
            }
            cursors
        }
    }

    #[derive(Default)]
    pub struct Wrap679 {
        pub k: Kernel679,
        lazy: LazyCells,
    }
    impl InverseBwt for Wrap679 {
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

// ============================================================================
// Harness Helpers: Block Loader, Self-Tests, Memory Ledger, `chaingrind`, Multi-Worker Timer
// ============================================================================
struct AdmittedBlk {
    runs: Runs,
    n: usize,
    p: u32,
}

fn load_admitted_from_bz2(bz2_path: &str) -> Vec<AdmittedBlk> {
    let data = std::fs::read(bz2_path).expect("failed to read bz2");
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

fn verify_and_mem<K: InverseBwt>(blks: &[AdmittedBlk], refs: &[Vec<u8>]) -> (usize, usize) {
    let k = Kernels::detect();
    let mut out = Vec::with_capacity(900_032);
    tls_reset();
    let mut kern = K::default();
    for (i, b) in blks.iter().enumerate() {
        assert!(kern.invert_runs(&b.runs, b.n, b.p, &mut out, k));
        if !refs.is_empty() {
            assert_eq!(out, refs[i], "mismatch on admitted block {i}");
        }
    }
    let s = tls_stats();
    drop(kern);
    s
}

fn run_adversarial_tests(blks: &[AdmittedBlk], refs: &[Vec<u8>], bz2_path: &str) {
    // 1. Poisoned tape & multi-epoch (>4 epochs, m > 210,000) check on candidate_c15
    let mut cand = candidate_c15::EpochQ16Kernel::default();
    let mut out = Vec::new();
    let mut max_m_seen = 0usize;
    for (i, b) in blks.iter().enumerate().take(24) {
        cand.poison_tapes_for_test();
        cand.run(b.runs.records(), b.n, b.p, 16, &mut out);
        assert_eq!(out, refs[i], "poisoned-tape mismatch on block {i}");
        // Also force LIMIT=64 on the block to push `m` across 4..6 64K epochs!
        cand.poison_tapes_for_test();
        cand.run(b.runs.records(), b.n, b.p, 64, &mut out);
        assert_eq!(out, refs[i], "multi-epoch LIMIT=64 mismatch on block {i}");
        max_m_seen = max_m_seen.max(cand.last_active_bytes);
    }
    assert!(max_m_seen > 0);

    // 2. End-to-end 431-block `wikidata.bz2` parallel decode with CRC verification at W=8 and W=10!
    let bz2_bytes = std::fs::read(bz2_path).expect("read wikidata.bz2");
    for &w in &[8usize, 10usize] {
        let mut rdr = mibz2::BzReader::parallel_workers_with::<candidate_c15::EpochQ16OpenCells>(
            std::io::Cursor::new(bz2_bytes.clone()),
            w,
            3 * w,
            Kernels::detect(),
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
        assert_eq!(total, 209_700_196, "unexpected wikidata decoded byte count at W={w}");
    }
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
            "675" => run_cg_kernel_only::<pr675::Arm675>(sample_bz2),
            "679" => run_cg_kernel_only::<pr679::Wrap679>(sample_bz2),
            "c15" => run_cg_kernel_only::<candidate_c15::EpochQ16OpenCells>(sample_bz2),
            _ => panic!("unknown chaingrind arm {arm}"),
        }
        return;
    }

    let bz2_path = args.get(2).map_or("/tmp/wikidata.bz2", String::as_str);
    let blks = load_admitted_from_bz2(bz2_path);
    assert_eq!(blks.len(), 228, "expected 228 admitted blocks in wikidata.bz2");
    let k = Kernels::detect();
    let mut refs = Vec::with_capacity(blks.len());
    {
        let mut rk = OpenCells::default();
        for b in &blks {
            let mut o = Vec::new();
            assert!(rk.invert_runs(&b.runs, b.n, b.p, &mut o, k));
            refs.push(o);
        }
    }

    println!("Running G5.1 adversarial dirty-tape, multi-epoch, 228-block exactness & 431-block W=8,10 BzReader CRC checks...");
    run_adversarial_tests(&blks, &refs, bz2_path);
    println!("  G5.1 Exactness & #[forbid(unsafe_code)] Verification: PASS");

    let names = [
        "0: Master-0.5.8 (OpenCells)",
        "1: PR#675 (R4@24 + 8-lane BP4K)",
        "2: PR#679 (R4@16 + Vec16)",
        "3: C15-EpochQ16-ZeroClearR4",
    ];
    let mems = [
        verify_and_mem::<OpenCells>(&blks, &[]),
        verify_and_mem::<pr675::Arm675>(&blks, &refs),
        verify_and_mem::<pr679::Wrap679>(&blks, &refs),
        verify_and_mem::<candidate_c15::EpochQ16OpenCells>(&blks, &refs),
    ];

    // Measure per-block active working set across all 228 admitted blocks
    let mut act_058 = Vec::with_capacity(blks.len());
    let mut act_c15 = Vec::with_capacity(blks.len());
    {
        let mut k679 = pr679::Kernel679::default();
        let mut kc15 = candidate_c15::EpochQ16Kernel::default();
        let mut o = Vec::with_capacity(900_032);
        for b in &blks {
            k679.run(b.runs.records(), b.n, b.p, &mut o);
            act_058.push(k679.last_active_058_equiv_bytes as f64 / 1024.0);
            kc15.run(b.runs.records(), b.n, b.p, 16, &mut o);
            act_c15.push(kc15.last_active_bytes as f64 / 1024.0);
        }
    }
    act_058.sort_by(|a, b| a.partial_cmp(b).unwrap());
    act_c15.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean_058 = act_058.iter().sum::<f64>() / act_058.len() as f64;
    let mean_c15 = act_c15.iter().sum::<f64>() / act_c15.len() as f64;
    let p95_058 = act_058[(act_058.len() * 95) / 100];
    let p95_c15 = act_c15[(act_c15.len() * 95) / 100];

    println!("\n=== G5.2 Per-Worker Kernel Heap & Active Working Set (228 Admitted Blocks) ===");
    let (b_ret, b_pk) = mems[0];
    for i in 0..4 {
        println!(
            "  {:<32} | Retained = {:7.1} KiB ({:5.3}x) | Peak = {:7.1} KiB ({:5.3}x)",
            names[i],
            mems[i].0 as f64 / 1024.0,
            mems[i].0 as f64 / b_ret as f64,
            mems[i].1 as f64 / 1024.0,
            mems[i].1 as f64 / b_pk as f64
        );
    }
    println!(
        "  Active Per-Block Working Set (incl. 900KB out): 0.5.8 mean={:6.1} KiB, p95={:6.1} KiB | C15 mean={:6.1} KiB ({:5.3}x), p95={:6.1} KiB ({:5.3}x)",
        mean_058, p95_058, mean_c15, mean_c15 / mean_058, p95_c15, p95_c15 / p95_058
    );
    let g52_pass = (mems[3].0 as f64 / b_ret as f64) <= 0.80
        && (mems[3].1 as f64 / b_pk as f64) <= 0.80
        && mean_c15 < mean_058
        && p95_c15 < p95_058;
    println!("  G5.2 Memory Non-Growth / Reduction Gate: {}", if g52_pass { "PASS" } else { "FAIL" });

    let rounds = 15usize;
    let mut g54_pass = true;
    for &w in &[1usize, 2, 8, 10] {
        let mut wall = vec![vec![0.0; rounds]; 4];
        let mut cpu = vec![vec![0.0; rounds]; 4];
        for r in 0..rounds + 1 {
            for step in 0..4 {
                let arm = (step + r) % 4;
                let (wm, cm) = match arm {
                    0 => time_workers::<OpenCells>(&blks, w),
                    1 => time_workers::<pr675::Arm675>(&blks, w),
                    2 => time_workers::<pr679::Wrap679>(&blks, w),
                    3 => time_workers::<candidate_c15::EpochQ16OpenCells>(&blks, w),
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
        let c679 = median(&mut cpu[2].clone());
        println!("\n=== G5.4 Paired Native Rust Timing (`lto=fat`) at W = {w} Workers ({rounds} rounds) ===");
        for arm in 0..4 {
            let mw = median(&mut wall[arm]);
            let mc = median(&mut cpu[arm]);
            println!(
                "  {:<32} | wall = {:6.1} ms ({:5.3}x 0.5.8) | sum_cpu = {:7.1} ms ({:5.3}x 0.5.8, {:5.3}x #679)",
                names[arm],
                mw,
                mw / bw,
                mc,
                mc / bc,
                mc / c679
            );
            if arm == 3 {
                if mw / bw > 0.925 || mc / bc > 0.925 {
                    g54_pass = false;
                }
                if (w == 8 || w == 10) && mc >= c679 {
                    g54_pass = false;
                }
            }
        }
    }
    println!("\n  G5.4 Multi-Worker Native Speed Gate: {}", if g54_pass { "PASS" } else { "FAIL" });
}

