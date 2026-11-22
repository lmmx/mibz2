use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, SliceSource};

// ---- Cycle 54: SliceSource Zero-Copy Feeder & Online Adaptive-Ratio out.reserve for decode_parallel ----
mod safe_c54 {
    #![forbid(unsafe_code)]
    use super::*;

    /// Runs `#739`-style worker pool directly over `SliceSource(data)` inside `thread::scope`
    /// (0 feeder thread, 0 `4 MiB` chunk copies, 0 `ChannelSource` window), returning either:
    /// - `adapt_cap == false`: `Vec::with_capacity(data.len() * 4)` (exact `#739` growth)
    /// - `adapt_cap == true`: online empirical expansion ratio `reserve()` after first full block (1 alloc, 0 bloat on random!)
    /// - `chunks_only == true`: zero-copy `Vec<Vec<u8>>`
    pub fn decode_slice_direct(
        data: &[u8], workers: usize, k: Kernels, adapt_cap: bool,
    ) -> Result<(Vec<u8>, usize), Error> {
        const READY_CAP: usize = 12 << 20;
        const POOLED_FROM: usize = 64 << 10;
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let floor = workers + 2;

        type Job = (u64, u64, mibz2::block::Symbols);
        struct Jq { q: VecDeque<Job>, closed: bool, waiting: usize }
        let jobs = Arc::new((Mutex::new(Jq { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }), Condvar::new()));

        struct Ready {
            ring: Vec<Option<(u64, Result<Vec<u8>, Error>)>>, mask: usize,
            issued: usize, taken: usize, end: Option<usize>, ready_bytes: usize,
            rw: bool, sw: bool, free_out: Vec<Vec<u8>>, free_syms: Vec<Vec<u16>>,
        }
        let ready = Arc::new(Mutex::new(Ready {
            ring: (0..cap).map(|_| None).collect(), mask: cap - 1,
            issued: 0, taken: 0, end: None, ready_bytes: 0,
            rw: false, sw: false, free_out: Vec::with_capacity(slots), free_syms: Vec::with_capacity(slots),
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_s = Arc::new(Condvar::new());
        let stopped = Arc::new(AtomicBool::new(false));

        let mut out = if adapt_cap { Vec::new() } else { Vec::with_capacity(data.len().saturating_mul(4)) };
        let mut reallocs = 0usize;

        std::thread::scope(|s| -> Result<(), Error> {
            let (js, rs, cvr, cvs, stp) = (jobs.clone(), ready.clone(), cv_r.clone(), cv_s.clone(), stopped.clone());
            s.spawn(move || {
                let mut sc = Scanner::new(SliceSource(data));
                loop {
                    let syms = {
                        let mut st = rs.lock().unwrap();
                        while !stp.load(Ordering::Relaxed) && (
                            st.issued - st.taken >= slots
                            || (st.ready_bytes >= READY_CAP && st.issued - st.taken >= floor)
                        ) {
                            st.sw = true; st = cvs.wait(st).unwrap();
                        }
                        st.sw = false;
                        if stp.load(Ordering::Relaxed) { break; }
                        st.free_syms.pop().unwrap_or_default()
                    };
                    match sc.next(syms) {
                        Ok(Event::Block { bit, symbols, .. }) => {
                            let place = { let mut st = rs.lock().unwrap(); let p = st.issued as u64; st.issued += 1; p };
                            let mut q = js.0.lock().unwrap();
                            q.q.push_back((place, bit, symbols));
                            let wake = q.waiting > 0; drop(q);
                            if wake { js.1.notify_one(); }
                        }
                        Ok(Event::Done) => {
                            { let mut q = js.0.lock().unwrap(); q.closed = true; } js.1.notify_all();
                            let wake = { let mut st = rs.lock().unwrap(); st.end = Some(st.issued); st.rw };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                        Err(e) => {
                            { let mut q = js.0.lock().unwrap(); q.closed = true; } js.1.notify_all();
                            let wake = {
                                let mut st = rs.lock().unwrap();
                                let at = st.issued; st.issued += 1;
                                let idx = at & st.mask;
                                st.ring[idx] = Some((0, Err(e))); st.end = Some(at + 1); st.rw
                            };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                    }
                }
            });

            for _ in 0..workers {
                let (jw, rw, cvr, stp) = (jobs.clone(), ready.clone(), cv_r.clone(), stopped.clone());
                s.spawn(move || {
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        let job = {
                            let mut q = jw.0.lock().unwrap();
                            loop {
                                if let Some(j) = q.q.pop_front() { break Some(j); }
                                if q.closed { break None; }
                                q.waiting += 1; q = jw.1.wait(q).unwrap(); q.waiting -= 1;
                            }
                        };
                        let Some((place, bit, b)) = job else { return; };
                        if stp.load(Ordering::Acquire) { continue; }
                        let is_pool = b.syms.len() >= 2048;
                        let want = b.max_len + b.max_len / 16 + 64;
                        let mut buf = if is_pool {
                            let mut v = rw.lock().unwrap().free_out.pop().unwrap_or_else(|| Vec::with_capacity(want));
                            if v.capacity() < want { v.reserve_exact(want - v.len()); }
                            v
                        } else { Vec::new() };
                        buf.clear();
                        let res = mibz2::decode_block(&b, k, &mut scratch, &mut buf)
                            .map(|_| buf).map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                        let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                        let mut st = rw.lock().unwrap();
                        if b.syms.capacity() > 0 && st.free_syms.len() < slots { st.free_syms.push(b.syms); }
                        let idx = (place as usize) & st.mask;
                        st.ring[idx] = Some((bit, res));
                        st.ready_bytes += bbytes;
                        let wake = st.rw && place as usize == st.taken;
                        drop(st);
                        if wake { cvr.notify_one(); }
                    }
                });
            }

            let mut cur: Vec<u8> = Vec::new();
            let mut sized = !adapt_cap;
            let mut prev_bit = 0u64;
            loop {
                let spent = std::mem::take(&mut cur);
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if spent.capacity() >= POOLED_FROM && st.free_out.len() < slots { st.free_out.push(spent); }
                    loop {
                        let t = st.taken;
                        let idx = t & st.mask;
                        if let Some((bit, res)) = st.ring[idx].take() {
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                            st.taken = t + 1;
                            let ws = st.sw; st.rw = false; drop(st);
                            if ws { cv_s.notify_one(); }
                            break Some((bit, res));
                        }
                        if st.end == Some(t) { st.rw = false; break None; }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk {
                    None => break Ok(()),
                    Some((bit, Ok(v))) => {
                        if !sized && bit > prev_bit && out.len() >= 100_000 {
                            // Block 1 has arrived: bit - prev_bit is exact compressed bits of Block 0!
                            let comp_bytes = ((bit - prev_bit) / 8).max(1) as f64;
                            let est = ((out.len() as f64 / comp_bytes) * (data.len() as f64) * 1.06) as usize;
                            let est = est.min(data.len().saturating_mul(40)).max(out.len() + v.len());
                            if est > out.capacity() { out.reserve_exact(est - out.len()); }
                            sized = true;
                        }
                        if prev_bit == 0 { prev_bit = bit; }
                        let c0 = out.capacity();
                        out.extend_from_slice(&v);
                        if out.capacity() > c0 { reallocs += 1; }
                        cur = v;
                    }
                    Some((_, Err(e))) => { stopped.store(true, Ordering::Release); cv_s.notify_all(); break Err(e); }
                }
            }
        })?;
        Ok((out, reallocs))
    }

    pub fn decode_slice_chunks(data: &[u8], workers: usize, k: Kernels) -> Result<Vec<Vec<u8>>, Error> {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        type Job = (u64, u64, mibz2::block::Symbols);
        struct Jq { q: VecDeque<Job>, closed: bool, waiting: usize }
        let jobs = Arc::new((Mutex::new(Jq { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }), Condvar::new()));
        struct Ready { ring: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize, issued: usize, taken: usize, end: Option<usize>, rw: bool, sw: bool, free_syms: Vec<Vec<u16>> }
        let ready = Arc::new(Mutex::new(Ready { ring: (0..cap).map(|_| None).collect(), mask: cap - 1, issued: 0, taken: 0, end: None, rw: false, sw: false, free_syms: Vec::with_capacity(slots) }));
        let cv_r = Arc::new(Condvar::new());
        let cv_s = Arc::new(Condvar::new());
        let stopped = Arc::new(AtomicBool::new(false));
        let mut chunks = Vec::new();
        std::thread::scope(|s| -> Result<(), Error> {
            let (js, rs, cvr, cvs, stp) = (jobs.clone(), ready.clone(), cv_r.clone(), cv_s.clone(), stopped.clone());
            s.spawn(move || {
                let mut sc = Scanner::new(SliceSource(data));
                loop {
                    let syms = {
                        let mut st = rs.lock().unwrap();
                        while !stp.load(Ordering::Relaxed) && st.issued - st.taken >= slots { st.sw = true; st = cvs.wait(st).unwrap(); }
                        st.sw = false;
                        if stp.load(Ordering::Relaxed) { break; }
                        st.free_syms.pop().unwrap_or_default()
                    };
                    match sc.next(syms) {
                        Ok(Event::Block { bit, symbols, .. }) => {
                            let place = { let mut st = rs.lock().unwrap(); let p = st.issued as u64; st.issued += 1; p };
                            let mut q = js.0.lock().unwrap(); q.q.push_back((place, bit, symbols));
                            let wake = q.waiting > 0; drop(q); if wake { js.1.notify_one(); }
                        }
                        Ok(Event::Done) => {
                            { let mut q = js.0.lock().unwrap(); q.closed = true; } js.1.notify_all();
                            let wake = { let mut st = rs.lock().unwrap(); st.end = Some(st.issued); st.rw };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                        Err(e) => {
                            { let mut q = js.0.lock().unwrap(); q.closed = true; } js.1.notify_all();
                            let wake = { let mut st = rs.lock().unwrap(); let at = st.issued; st.issued += 1; let idx = at & st.mask; st.ring[idx] = Some(Err(e)); st.end = Some(at + 1); st.rw };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                    }
                }
            });
            for _ in 0..workers {
                let (jw, rw, cvr, stp) = (jobs.clone(), ready.clone(), cv_r.clone(), stopped.clone());
                s.spawn(move || {
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        let job = {
                            let mut q = jw.0.lock().unwrap();
                            loop {
                                if let Some(j) = q.q.pop_front() { break Some(j); }
                                if q.closed { break None; }
                                q.waiting += 1; q = jw.1.wait(q).unwrap(); q.waiting -= 1;
                            }
                        };
                        let Some((place, bit, b)) = job else { return; };
                        if stp.load(Ordering::Acquire) { continue; }
                        let mut buf = if b.syms.len() >= 2048 { Vec::with_capacity(b.max_len + b.max_len / 16 + 64) } else { Vec::new() };
                        let res = mibz2::decode_block(&b, k, &mut scratch, &mut buf)
                            .map(|_| buf).map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                        let mut st = rw.lock().unwrap();
                        if b.syms.capacity() > 0 && st.free_syms.len() < slots { st.free_syms.push(b.syms); }
                        let idx = (place as usize) & st.mask;
                        st.ring[idx] = Some(res);
                        let wake = st.rw && place as usize == st.taken; drop(st);
                        if wake { cvr.notify_one(); }
                    }
                });
            }
            loop {
                let blk = {
                    let mut st = ready.lock().unwrap();
                    loop {
                        let t = st.taken; let idx = t & st.mask;
                        if let Some(res) = st.ring[idx].take() {
                            st.taken = t + 1; let ws = st.sw; st.rw = false; drop(st);
                            if ws { cv_s.notify_one(); }
                            break Some(res);
                        }
                        if st.end == Some(t) { st.rw = false; break None; }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk {
                    None => break Ok(()),
                    Some(Ok(v)) => chunks.push(v),
                    Some(Err(e)) => { stopped.store(true, Ordering::Release); cv_s.notify_all(); break Err(e); }
                }
            }
        })?;
        Ok(chunks)
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
    (mean.exp(), (mean - 2.201 * se).exp(), (mean + 2.201 * se).exp())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let arm = args[3].as_str();
        let w: usize = args[4].parse().unwrap();
        let r0 = rusage(); let t0 = Instant::now();
        let (len, reallocs) = match arm {
            "D739_A" | "D739_B" => {
                // Force OpenCells via with_inverse matching decode_parallel's exact 4x reserve + 4 MiB feeder + read_block
                let dec = mibz2::StreamDecoder::with_inverse::<OpenCells>(w, 4 * w, k);
                let mut out = Vec::with_capacity(data.len().saturating_mul(4));
                let mut rlc = 0usize;
                std::thread::scope(|s| {
                    s.spawn(|| {
                        for c in data.chunks(4 << 20) {
                            let mut ch = dec.spare(); ch.clear(); ch.extend_from_slice(c);
                            if dec.write(ch).is_err() { return; }
                        }
                        dec.close();
                    });
                    let mut spent = Vec::new();
                    while let Some(b) = dec.read_block(spent).unwrap() {
                        let c0 = out.capacity();
                        out.extend_from_slice(&b);
                        if out.capacity() > c0 { rlc += 1; }
                        spent = b;
                    }
                });
                let l = out.len(); std::hint::black_box(&out); (l, rlc)
            }
            "D1_SliceSrc" => {
                let (out, rlc) = safe_c54::decode_slice_direct(&data, w, k, false).unwrap();
                let l = out.len(); std::hint::black_box(&out); (l, rlc)
            }
            "D2_AdaptCap" => {
                let (out, rlc) = safe_c54::decode_slice_direct(&data, w, k, true).unwrap();
                let l = out.len(); std::hint::black_box(&out); (l, rlc)
            }
            "D3_Chunks" => {
                let ch = safe_c54::decode_slice_chunks(&data, w, k).unwrap();
                let l: usize = ch.iter().map(|v| v.len()).sum(); std::hint::black_box(&ch); (l, 0)
            }
            _ => panic!(),
        };
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {:.2} {len} {reallocs}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0);
        return;
    }

    let exe = std::env::current_exe().unwrap();
    for &(label, path) in &[
        ("wikidata-l9 (15.6 MiB -> 209.8 MB, 13.5x ratio)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2"),
        ("random-l9 (10.7 MiB -> 10.7 MB, 1.0x ratio)", "/tmp/wormhole_r6/random_l9_11m.bz2"),
    ] {
        let arms = ["D739_A", "D739_B", "D1_SliceSrc", "D2_AdaptCap", "D3_Chunks"];
        println!("\n=== 12-ROUND ROTATED FRESH-PROCESS 4W decode_parallel A/B: {label} ===");
        let mut ws: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut us: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut fs_: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rs: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut rls: BTreeMap<&str, usize> = BTreeMap::new();
        let mut exp_len: Option<usize> = None;
        for r in 0..12 {
            for i in 0..arms.len() {
                let arm = arms[(r + i) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                let len: usize = p[5].parse().unwrap();
                if let Some(el) = exp_len { assert_eq!(len, el); } else { exp_len = Some(len); }
                ws.entry(arm).or_default().push(p[0].parse().unwrap());
                us.entry(arm).or_default().push(p[1].parse().unwrap());
                ss.entry(arm).or_default().push(p[2].parse().unwrap());
                fs_.entry(arm).or_default().push(p[3].parse().unwrap());
                rs.entry(arm).or_default().push(p[4].parse().unwrap());
                rls.insert(arm, p[6].parse().unwrap());
            }
        }
        let ma = &ws["D739_A"];
        println!("{:<14} {:>7} {:>24} {:>7} {:>6} {:>7} {:>7} {:>5}", "Arm", "med_ms", "vs D739_A [95% CI]", "user", "sys", "minflt", "rss_MB", "grow");
        for &arm in &arms {
            let mut wv = ws[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut uv = us[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = ss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = fs_[arm].clone(); fv.sort();
            let mut rv = rs[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(ws[arm].iter().copied()).collect::<Vec<_>>());
            println!("{:<14} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>7.1} {:>6.2} {:>7} {:>7.2} {:>5}",
                arm, wv[6], rat, lo, hi, uv[6], sv[6], fv[6], rv[6], rls[arm]);
        }
    }
}
