use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BlockSpan, Error, ErrorKind, Kernels, Scratch, StreamDecoder};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::OpenCells;

// ---- Cycle 57: Forensics of Whole-Input ParScan vs Single-Window Barrier (C38b) vs Ping-Pong 2-Window (PR 4) ----
mod safe_c57 {
    #![forbid(unsafe_code)]
    use super::*;

    pub struct AbHash { pub bytes: u64, pub h: u64 }
    impl AbHash {
        pub fn new() -> Self { Self { bytes: 0, h: 0xcbf29ce484222325 } }
        #[inline]
        pub fn feed(&mut self, s: &[u8]) {
            self.bytes += s.len() as u64;
            let mut h = self.h;
            let mut ch = s.chunks_exact(8);
            for c in &mut ch { h = (h ^ u64::from_le_bytes(c.try_into().unwrap())).wrapping_mul(0x100000001b3); }
            for &b in ch.remainder() { h = (h ^ b as u64).wrapping_mul(0x100000001b3); }
            self.h = h;
        }
    }

    /// P0_WholeInput: Models #741 whole-input ParScan (copies full input Vec<u8> + unbounded BTreeMap ready, no 12M cap)
    pub fn run_p0_whole_input(data: &[u8], spans: &[BlockSpan], workers: usize, k: Kernels) -> (u64, u64) {
        let full_buf = Arc::new(data.to_vec()); // simulates holding full compressed input in RAM
        let n = spans.len();
        let next = Arc::new(AtomicUsize::new(0));
        struct St { ready: BTreeMap<usize, Result<Vec<u8>, Error>>, taken: usize, rw: bool, free_out: Vec<Vec<u8>> }
        let mu = Arc::new(Mutex::new(St { ready: BTreeMap::new(), taken: 0, rw: false, free_out: Vec::new() }));
        let cv = Arc::new(Condvar::new());
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            for _ in 0..workers {
                let (buf, nx, m, c) = (full_buf.clone(), next.clone(), mu.clone(), cv.clone());
                s.spawn(move || {
                    let mut syms = Vec::new();
                    let mut sc = Scratch::<OpenCells>::default();
                    loop {
                        let i = nx.fetch_add(1, Ordering::Relaxed);
                        if i >= n { return; }
                        let sp = &spans[i];
                        let mut br = Bits::at(&buf, sp.start_bit);
                        let res = match read_item(&mut br, sp.level, std::mem::take(&mut syms)) {
                            Ok(Item::Block(b)) => {
                                let mut out = m.lock().unwrap().free_out.pop().unwrap_or_default();
                                out.clear();
                                let r = mibz2::decode_block(&b, k, &mut sc, &mut out).map(|_| out);
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((e, _)) => Err(e),
                        }.map_err(|kind| Error { kind, block: Some(i as u64), bit: Some(sp.start_bit) });
                        let mut st = m.lock().unwrap();
                        st.ready.insert(i, res);
                        let wake = st.rw && i == st.taken; drop(st);
                        if wake { c.notify_one(); }
                    }
                });
            }
            let mut cur = Vec::new();
            loop {
                let spent = std::mem::take(&mut cur);
                let blk = {
                    let mut st = mu.lock().unwrap();
                    if spent.capacity() > 0 { st.free_out.push(spent); }
                    loop {
                        let t = st.taken;
                        if t == n { st.rw = false; break None; }
                        if let Some(res) = st.ready.remove(&t) { st.taken = t + 1; st.rw = false; break Some(res); }
                        st.rw = true; st = cv.wait(st).unwrap();
                    }
                };
                match blk { None => break, Some(Ok(v)) => { h.feed(&v); cur = v; } Some(Err(e)) => panic!("{e:?}") }
            }
        });
        (h.bytes, h.h)
    }

    /// Build bounded 4 MiB windows from raw stream + spans (modeling Mechanisms C38b anchor finder output per 4 MiB window)
    struct WinSpec {
        abs_base: usize,
        len: usize,
        // (global_place, rel_start_bit, level)
        blocks: Vec<(usize, u64, u32)>,
    }

    fn partition_windows(data_len: usize, spans: &[BlockSpan], win_cap: usize) -> Vec<WinSpec> {
        let mut wins = Vec::new();
        let mut abs_base = 0usize;
        let mut s_idx = 0usize;
        while s_idx < spans.len() {
            let win_end = (abs_base + win_cap).min(data_len);
            let mut blocks = Vec::new();
            while s_idx < spans.len() {
                let sp = &spans[s_idx];
                let end_byte = ((sp.end_bit + 7) / 8) as usize;
                let start_byte = (sp.start_bit / 8) as usize;
                if end_byte <= win_end || blocks.is_empty() {
                    let rel_bit = sp.start_bit - (abs_base as u64) * 8;
                    blocks.push((s_idx, rel_bit, sp.level));
                    s_idx += 1;
                    if end_byte > win_end {
                        // Single block exceeded win_cap: extend this window to fit it
                        wins.push(WinSpec { abs_base, len: end_byte - abs_base, blocks });
                        abs_base = end_byte;
                        blocks = Vec::new();
                        break;
                    }
                } else {
                    let next_base = start_byte;
                    wins.push(WinSpec { abs_base, len: win_end - abs_base, blocks });
                    abs_base = next_base;
                    blocks = Vec::new();
                    break;
                }
            }
            if !blocks.is_empty() {
                wins.push(WinSpec { abs_base, len: win_end - abs_base, blocks });
                break;
            }
        }
        wins
    }

    /// P1_Win1_Barrier: Bounded 4 MiB window WITH inter-window drain barrier (models C38b single-window epoch)
    /// + #739 Ready ring, 12 MiB ready cap, POOLED_FROM!
    pub fn run_p1_win1_barrier(data: &[u8], spans: &[BlockSpan], win_cap: usize, workers: usize, k: Kernels) -> ((u64, u64), usize) {
        const READY_CAP: usize = 12 << 20;
        const POOLED_FROM: usize = 64 << 10;
        let wins = partition_windows(data.len(), spans, win_cap);
        let n_wins = wins.len();
        let total_blks = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let floor = workers + 2;

        struct Epoch {
            win: Arc<Vec<u8>>,
            blocks: Vec<(usize, u64, u32)>,
            epoch: usize,
            active: usize,
            done: bool,
        }
        let ep = Arc::new((
            Mutex::new(Epoch { win: Arc::new(Vec::new()), blocks: Vec::new(), epoch: 0, active: 0, done: false }),
            Condvar::new(), // workers wait for next epoch
            Condvar::new(), // producer waits for active == 0 (the C38b barrier!)
        ));
        let cursor = Arc::new(AtomicUsize::new(0));

        struct Ready {
            ring: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize,
            taken: usize, ready_bytes: usize, rw: bool, ww: usize, free_out: Vec<Vec<u8>>,
        }
        let ready = Arc::new(Mutex::new(Ready {
            ring: (0..cap).map(|_| None).collect(), mask: cap - 1,
            taken: 0, ready_bytes: 0, rw: false, ww: 0, free_out: Vec::with_capacity(slots),
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            // Producer reads 1 bounded 4 MiB window at a time, waiting for epoch drain barrier before next window
            let (ep_p, cur_p) = (ep.clone(), cursor.clone());
            s.spawn(move || {
                let mut raw = Vec::with_capacity(win_cap);
                for wspec in wins {
                    raw.clear();
                    if raw.capacity() < wspec.len { raw.reserve_exact(wspec.len - raw.len()); }
                    raw.extend_from_slice(&data[wspec.abs_base..wspec.abs_base + wspec.len]);
                    let arc_w = Arc::new(raw);
                    {
                        let mut st = ep_p.0.lock().unwrap();
                        st.win = Arc::clone(&arc_w);
                        st.blocks = wspec.blocks;
                        st.active = workers;
                        cur_p.store(0, Ordering::Release);
                        st.epoch += 1;
                        ep_p.1.notify_all();
                        // BARRIER: wait for all workers to finish this 4 MiB window so we can reclaim `raw`!
                        while st.active > 0 { st = ep_p.2.wait(st).unwrap(); }
                        st.win = Arc::new(Vec::new());
                    }
                    raw = Arc::try_unwrap(arc_w).unwrap_or_else(|_| Vec::with_capacity(win_cap));
                }
                { let mut st = ep_p.0.lock().unwrap(); st.done = true; ep_p.1.notify_all(); }
            });

            for _ in 0..workers {
                let (ep_w, cur_w, rw, cvr, cvw) = (ep.clone(), cursor.clone(), ready.clone(), cv_r.clone(), cv_w.clone());
                s.spawn(move || {
                    let mut seen = 0usize;
                    let mut syms = Vec::new();
                    let mut sc = Scratch::<OpenCells>::default();
                    loop {
                        let (win_arc, blks) = {
                            let mut st = ep_w.0.lock().unwrap();
                            while !st.done && st.epoch == seen { st = ep_w.1.wait(st).unwrap(); }
                            if st.done && st.epoch == seen { return; }
                            seen = st.epoch;
                            (Arc::clone(&st.win), st.blocks.clone())
                        };
                        loop {
                            let i = cur_w.fetch_add(1, Ordering::Relaxed);
                            if i >= blks.len() { break; }
                            let (place, rel_bit, level) = blks[i];
                            {
                                let mut st = rw.lock().unwrap();
                                while place >= st.taken + slots || (st.ready_bytes >= READY_CAP && place >= st.taken + floor) {
                                    st.ww += 1; st = cvw.wait(st).unwrap(); st.ww -= 1;
                                }
                            }
                            let mut br = Bits::at(&win_arc, rel_bit);
                            let res = match read_item(&mut br, level, std::mem::take(&mut syms)) {
                                Ok(Item::Block(b)) => {
                                    let is_pool = b.syms.len() >= 2048;
                                    let want = b.max_len + b.max_len / 16 + 64;
                                    let mut out = if is_pool {
                                        let mut v = rw.lock().unwrap().free_out.pop().unwrap_or_else(|| Vec::with_capacity(want));
                                        if v.capacity() < want { v.reserve_exact(want - v.len()); }
                                        v
                                    } else { Vec::new() };
                                    out.clear();
                                    let r = mibz2::decode_block(&b, k, &mut sc, &mut out).map(|_| out);
                                    syms = b.syms;
                                    r
                                }
                                Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                                Err((e, _)) => Err(e),
                            }.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(rel_bit) });
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            let mut st = rw.lock().unwrap();
                            let idx = place & st.mask;
                            st.ring[idx] = Some(res);
                            st.ready_bytes += bbytes;
                            let wake = st.rw && place == st.taken;
                            drop(st);
                            if wake { cvr.notify_one(); }
                        }
                        drop(win_arc);
                        let mut st = ep_w.0.lock().unwrap();
                        st.active -= 1;
                        if st.active == 0 { ep_w.2.notify_one(); }
                    }
                });
            }

            let mut cur = Vec::new();
            loop {
                let spent = std::mem::take(&mut cur);
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if spent.capacity() >= POOLED_FROM && st.free_out.len() < slots { st.free_out.push(spent); }
                    loop {
                        let t = st.taken;
                        if t == total_blks { st.rw = false; break None; }
                        let idx = t & st.mask;
                        if let Some(res) = st.ring[idx].take() {
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                            st.taken = t + 1;
                            let ww = st.ww > 0; st.rw = false; drop(st);
                            if ww { cv_w.notify_one(); }
                            break Some(res);
                        }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk { None => break, Some(Ok(v)) => { h.feed(&v); cur = v; } Some(Err(e)) => panic!("{e:?}") }
            }
        });
        ((h.bytes, h.h), n_wins)
    }

    /// P2_Win2_NoBarrier (PR 4 Prototype): Bounded Ping-Pong 2-Window Streaming `BlockWindow` (`<= 2 * 4 MiB` input RAM)!
    /// ZERO inter-window epoch barrier: workers claim `(Arc<Vec<u8>>, rel_bit, level)` spans seamlessly across windows!
    pub fn run_p2_win2_nobarrier(data: &[u8], spans: &[BlockSpan], win_cap: usize, workers: usize, k: Kernels) -> ((u64, u64), usize) {
        const READY_CAP: usize = 12 << 20;
        const POOLED_FROM: usize = 64 << 10;
        const MAX_LIVE_WINS: usize = 2;
        let wins = partition_windows(data.len(), spans, win_cap);
        let n_wins = wins.len();
        let total_blks = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let floor = workers + 2;

        struct SpanTask { win: Arc<Vec<u8>>, place: usize, rel_bit: u64, level: u32 }
        struct TaskQ { q: VecDeque<SpanTask>, closed: bool, waiting: usize, free_win: Vec<Vec<u8>> }
        let tq = Arc::new((
            Mutex::new(TaskQ { q: VecDeque::with_capacity(256), closed: false, waiting: 0, free_win: Vec::with_capacity(MAX_LIVE_WINS) }),
            Condvar::new(), // workers wait for tasks
            Condvar::new(), // producer waits for a recycled window buffer (bounds live input RAM to <= 2 * win_cap!)
        ));
        let live_wins = Arc::new(AtomicUsize::new(0));

        struct Ready {
            ring: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize,
            taken: usize, ready_bytes: usize, rw: bool, ww: usize, free_out: Vec<Vec<u8>>,
        }
        let ready = Arc::new(Mutex::new(Ready {
            ring: (0..cap).map(|_| None).collect(), mask: cap - 1,
            taken: 0, ready_bytes: 0, rw: false, ww: 0, free_out: Vec::with_capacity(slots),
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            // Producer publishes up to MAX_LIVE_WINS=2 bounded windows concurrently (ping-pong overlap, 0 barrier!)
            let (tq_p, lw_p) = (tq.clone(), live_wins.clone());
            s.spawn(move || {
                for wspec in wins {
                    // Acquire a recycled window buffer or allocate up to MAX_LIVE_WINS
                    let mut buf = {
                        let mut q = tq_p.0.lock().unwrap();
                        loop {
                            if let Some(v) = q.free_win.pop() { break v; }
                            if lw_p.load(Ordering::Acquire) < MAX_LIVE_WINS {
                                lw_p.fetch_add(1, Ordering::AcqRel);
                                break Vec::with_capacity(win_cap);
                            }
                            q = tq_p.2.wait(q).unwrap();
                        }
                    };
                    buf.clear();
                    if buf.capacity() < wspec.len { buf.reserve_exact(wspec.len - buf.len()); }
                    buf.extend_from_slice(&data[wspec.abs_base..wspec.abs_base + wspec.len]);
                    let arc_w = Arc::new(buf);
                    let mut q = tq_p.0.lock().unwrap();
                    for (place, rel_bit, level) in wspec.blocks {
                        q.q.push_back(SpanTask { win: Arc::clone(&arc_w), place, rel_bit, level });
                    }
                    let wake = q.waiting > 0;
                    drop(q);
                    drop(arc_w);
                    if wake { tq_p.1.notify_all(); }
                }
                { let mut q = tq_p.0.lock().unwrap(); q.closed = true; }
                tq_p.1.notify_all();
            });

            for _ in 0..workers {
                let (tq_w, rw, cvr, cvw) = (tq.clone(), ready.clone(), cv_r.clone(), cv_w.clone());
                s.spawn(move || {
                    let mut syms = Vec::new();
                    let mut sc = Scratch::<OpenCells>::default();
                    loop {
                        let task = {
                            let mut q = tq_w.0.lock().unwrap();
                            loop {
                                if let Some(t) = q.q.pop_front() { break Some(t); }
                                if q.closed { break None; }
                                q.waiting += 1; q = tq_w.1.wait(q).unwrap(); q.waiting -= 1;
                            }
                        };
                        let Some(SpanTask { win, place, rel_bit, level }) = task else { return; };
                        {
                            let mut st = rw.lock().unwrap();
                            while place >= st.taken + slots || (st.ready_bytes >= READY_CAP && place >= st.taken + floor) {
                                st.ww += 1; st = cvw.wait(st).unwrap(); st.ww -= 1;
                            }
                        }
                        let mut br = Bits::at(&win, rel_bit);
                        let res = match read_item(&mut br, level, std::mem::take(&mut syms)) {
                            Ok(Item::Block(b)) => {
                                // Done reading compressed bits! Try reclaiming `win` RIGHT NOW if we were its last holder!
                                if let Ok(reclaimed) = Arc::try_unwrap(win) {
                                    let mut q = tq_w.0.lock().unwrap();
                                    q.free_win.push(reclaimed);
                                    drop(q);
                                    tq_w.2.notify_one();
                                }
                                let is_pool = b.syms.len() >= 2048;
                                let want = b.max_len + b.max_len / 16 + 64;
                                let mut out = if is_pool {
                                    let mut v = rw.lock().unwrap().free_out.pop().unwrap_or_else(|| Vec::with_capacity(want));
                                    if v.capacity() < want { v.reserve_exact(want - v.len()); }
                                    v
                                } else { Vec::new() };
                                out.clear();
                                let r = mibz2::decode_block(&b, k, &mut sc, &mut out).map(|_| out);
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => {
                                if let Ok(reclaimed) = Arc::try_unwrap(win) {
                                    let mut q = tq_w.0.lock().unwrap(); q.free_win.push(reclaimed); drop(q); tq_w.2.notify_one();
                                }
                                Err(ErrorKind::Truncated)
                            }
                            Err((e, _)) => {
                                if let Ok(reclaimed) = Arc::try_unwrap(win) {
                                    let mut q = tq_w.0.lock().unwrap(); q.free_win.push(reclaimed); drop(q); tq_w.2.notify_one();
                                }
                                Err(e)
                            }
                        }.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(rel_bit) });

                        let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                        let mut st = rw.lock().unwrap();
                        let idx = place & st.mask;
                        st.ring[idx] = Some(res);
                        st.ready_bytes += bbytes;
                        let wake = st.rw && place == st.taken;
                        drop(st);
                        if wake { cvr.notify_one(); }
                    }
                });
            }

            let mut cur = Vec::new();
            loop {
                let spent = std::mem::take(&mut cur);
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if spent.capacity() >= POOLED_FROM && st.free_out.len() < slots { st.free_out.push(spent); }
                    loop {
                        let t = st.taken;
                        if t == total_blks { st.rw = false; break None; }
                        let idx = t & st.mask;
                        if let Some(res) = st.ring[idx].take() {
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                            st.taken = t + 1;
                            let ww = st.ww > 0; st.rw = false; drop(st);
                            if ww { cv_w.notify_one(); }
                            break Some(res);
                        }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk { None => break, Some(Ok(v)) => { h.feed(&v); cur = v; } Some(Err(e)) => panic!("{e:?}") }
            }
        });
        ((h.bytes, h.h), n_wins)
    }
}

#[repr(C)] #[derive(Default, Clone, Copy)] struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)] #[derive(Default, Clone, Copy)] struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval, ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64, ru_inblock: i64, ru_oublock: i64,
    ru_msgsnd: i64, ru_msgrcv: i64, ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" { fn getrusage(who: i32, usage: *mut Rusage) -> i32; }
fn rusage() -> Rusage { let mut r = Rusage::default(); unsafe { getrusage(0, &mut r); } r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = if logs.len() >= 12 { 2.201 } else { 2.776 };
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn load_spans(path: &str) -> Vec<BlockSpan> {
    fs::read_to_string(format!("{path}.spans")).unwrap().lines().map(|l| {
        let p: Vec<u64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
        BlockSpan { index: p[0], start_bit: p[1], end_bit: p[2], crc: p[3] as u32, stream: p[4], level: p[5] as u32 }
    }).collect()
}

fn run_master_739(data: &[u8], w: usize, k: Kernels) -> (u64, u64) {
    let dec = StreamDecoder::with_inverse::<OpenCells>(w, 4 * w, k);
    let mut h = safe_c57::AbHash::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in data.chunks(4 << 20) {
                let mut v = dec.spare(); v.clear(); v.extend_from_slice(c);
                if dec.write(v).is_err() { return; }
            }
            dec.close();
        });
        let mut spent = Vec::new();
        while let Some(b) = dec.read_block(spent).unwrap() { h.feed(&b); spent = b; }
    });
    (h.bytes, h.h)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let arm = args[3].as_str();
        let w: usize = args[4].parse().unwrap();
        let spans = if arm.starts_with("P") { load_spans(&args[2]) } else { Vec::new() };
        let r0 = rusage(); let t0 = Instant::now();
        let (bytes, hash) = match arm {
            "M739_A" | "M739_B" => run_master_739(&data, w, k),
            "P0_WholeIn" => safe_c57::run_p0_whole_input(&data, &spans, w, k),
            "P1_Win1_Bar" => safe_c57::run_p1_win1_barrier(&data, &spans, 4 << 20, w, k).0,
            "P2_Win2_NoBar" => safe_c57::run_p2_win2_nobarrier(&data, &spans, 4 << 20, w, k).0,
            _ => panic!(),
        };
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {bytes} {hash:016x}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let path = "/tmp/wormhole_r6/wikidata_431b_210m.bz2";
    let arms = ["M739_A", "M739_B", "P0_WholeIn", "P1_Win1_Bar", "P2_Win2_NoBar"];
    let mut ref_bh: Option<(String, String)> = None;
    for &(w, rounds) in &[(4usize, 12usize), (6, 8), (10, 5), (16, 5)] {
        println!("\n=== {rounds}-ROUND ROTATED {w}W PARSCAN FORENSICS ON 3296195e3 (210 MB, 431 BLKS) ===");
        let mut ws: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut us: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut fl: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut vc: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rs: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        for r in 0..rounds {
            for i in 0..arms.len() {
                let arm = arms[(r + i) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, &w.to_string()]).output().unwrap();
                assert!(out.status.success(), "{arm}: {}", String::from_utf8_lossy(&out.stderr));
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                let bh = (p[6].to_string(), p[7].to_string());
                if let Some(ref e) = ref_bh { assert_eq!(&bh, e); } else { ref_bh = Some(bh); }
                ws.entry(arm).or_default().push(p[0].parse().unwrap());
                us.entry(arm).or_default().push(p[1].parse().unwrap());
                ss.entry(arm).or_default().push(p[2].parse().unwrap());
                fl.entry(arm).or_default().push(p[3].parse().unwrap());
                vc.entry(arm).or_default().push(p[4].parse().unwrap());
                rs.entry(arm).or_default().push(p[5].parse().unwrap());
            }
        }
        let ma = &ws["M739_A"];
        println!("{:<15} {:>7} {:>24} {:>7} {:>6} {:>7} {:>5} {:>7}", "Arm", "med_ms", "vs M739_A [95% CI]", "user", "sys", "minflt", "vcsw", "rss_MB");
        for &arm in &arms {
            let mut wv = ws[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut uv = us[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = ss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = fl[arm].clone(); fv.sort();
            let mut cv = vc[arm].clone(); cv.sort();
            let mut rv = rs[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(ws[arm].iter().copied()).collect::<Vec<_>>());
            let m = rounds / 2;
            println!("{:<15} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>7.1} {:>6.2} {:>7} {:>5} {:>7.2}",
                arm, wv[m], rat, lo, hi, uv[m], sv[m], fv[m], cv[m], rv[m]);
        }
    }
}
