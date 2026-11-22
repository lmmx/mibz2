use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, SliceSource, Source};

// ---- Cycle 41: Tail-Latency & Straggler Scheduling (Graham LPT via O(1) syms.len() proxy) ----
mod safe_c41 {
    #![forbid(unsafe_code)]
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum SchedPolicy {
        Fifo,            // v0.5.12 / P1c baseline: strict FIFO
        UrgentPlusLpt,   // S1: dispatch place <= taken + 2 first, else max syms.len() (Graham LPT)
        LookaheadHeavy,  // S2: if any queued block has syms.len() >= 220_000 (dense/fallback), start it immediately unless place == taken!
    }

    type Job = (u64, u64, mibz2::block::Symbols);
    struct PooledSource {
        rx: mpsc::Receiver<Vec<u8>>, tx_free: mpsc::Sender<Vec<u8>>,
        buf: Vec<u8>, base: u64, released: u64, eof: bool, cancelled: Arc<AtomicBool>,
    }
    impl Source for PooledSource {
        fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
            const COMPACT: usize = 4 << 20;
            while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
                match self.rx.recv() {
                    Ok(mut chunk) => {
                        let drop_n = (self.released - self.base) as usize;
                        if drop_n == self.buf.len() {
                            let mut old = std::mem::replace(&mut self.buf, chunk);
                            self.base = self.released; old.clear();
                            let _ = self.tx_free.send(old);
                        } else {
                            if drop_n >= COMPACT { self.buf.drain(..drop_n); self.base = self.released; }
                            self.buf.extend_from_slice(&chunk);
                            chunk.clear(); let _ = self.tx_free.send(chunk);
                        }
                    }
                    Err(_) => self.eof = true,
                }
                if self.cancelled.load(Ordering::Relaxed) { return Err(ErrorKind::Cancelled); }
            }
            let i = (from - self.base) as usize;
            Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
        }
        fn release(&mut self, upto: u64) { self.released = self.released.max(upto); }
    }

    struct ReadyRing {
        slots_arr: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize, taken: u64, end: Option<u64>,
        reader_waiting: bool, scanner_waiting: bool, free_out: Vec<Vec<u8>>, free_syms: Vec<Vec<u16>>,
    }
    struct JobQueue { q: VecDeque<Job>, closed: bool, waiting: usize }

    fn pop_job(q: &mut VecDeque<Job>, policy: SchedPolicy, taken: u64) -> Option<Job> {
        if q.len() <= 1 {
            return q.pop_front();
        }
        match policy {
            SchedPolicy::Fifo => q.pop_front(),
            SchedPolicy::UrgentPlusLpt => {
                // If front is urgent (within 2 blocks of reader's `taken`), preserve HOL order!
                if q[0].0 <= taken + 2 {
                    return q.pop_front();
                }
                // Otherwise pick the heaviest queued block (max syms.len()) — O(slots <= 40) scan!
                let mut best_i = 0usize;
                let mut best_len = q[0].2.syms.len();
                for (i, j) in q.iter().enumerate().skip(1) {
                    let l = j.2.syms.len();
                    if l > best_len {
                        best_len = l;
                        best_i = i;
                    }
                }
                q.remove(best_i)
            }
            SchedPolicy::LookaheadHeavy => {
                if q[0].0 <= taken + 1 {
                    return q.pop_front();
                }
                // Preempt only genuinely heavy (dense/random) blocks (>= 220K syms) so they never sit in tail!
                if let Some(idx) = q.iter().position(|j| j.2.syms.len() >= 220_000) {
                    return q.remove(idx);
                }
                q.pop_front()
            }
        }
    }

    pub fn decode_c41<R: Read + Send + 'static>(
        mut r: R, workers: usize, policy: SchedPolicy, k: Kernels, mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<usize, Error> {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let cancelled = Arc::new(AtomicBool::new(false));
        let taken_atomic = Arc::new(AtomicU64::new(0));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0, end: None,
            reader_waiting: false, scanner_waiting: false,
            free_out: Vec::with_capacity(slots), free_syms: Vec::with_capacity(slots),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_scanner = Arc::new(Condvar::new());
        let job_deque = Arc::new((Mutex::new(JobQueue { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }), Condvar::new()));
        let (in_tx, in_rx) = mpsc::sync_channel::<Vec<u8>>(2);
        let (free_tx, free_rx) = mpsc::channel::<Vec<u8>>();

        let feeder = std::thread::spawn(move || {
            let mut pool: Vec<Vec<u8>> = (0..3).map(|_| Vec::with_capacity(1 << 20)).collect();
            loop {
                while let Ok(b) = free_rx.try_recv() { pool.push(b); }
                let mut buf = pool.pop().unwrap_or_else(|| Vec::with_capacity(1 << 20));
                buf.resize(1 << 20, 0);
                let mut n = 0;
                while n < buf.len() {
                    match r.read(&mut buf[n..]) {
                        Ok(0) => break, Ok(g) => n += g,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => return,
                    }
                }
                if n == 0 { break; }
                buf.truncate(n);
                if in_tx.send(buf).is_err() { return; }
            }
        });

        let (sr, scr, scs, sc, sjq) = (ready.clone(), cv_reader.clone(), cv_scanner.clone(), cancelled.clone(), job_deque.clone());
        let scanner_h = std::thread::spawn(move || {
            let src = PooledSource { rx: in_rx, tx_free: free_tx, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: sc.clone() };
            let mut scnr = Scanner::new(src);
            let mut issued: u64 = 0;
            let finish = |err: Option<Error>, issued: u64| {
                { let mut jq = sjq.0.lock().unwrap(); jq.closed = true; }
                sjq.1.notify_all();
                let mut st = sr.lock().unwrap();
                match err {
                    Some(e) if e.kind != ErrorKind::Cancelled => {
                        let idx = (issued as usize) & st.mask;
                        st.slots_arr[idx] = Some(Err(e));
                        st.end = Some(issued + 1);
                    }
                    _ => st.end = Some(issued),
                }
                drop(st); scr.notify_one();
            };
            loop {
                let syms = {
                    let mut st = sr.lock().unwrap();
                    while !sc.load(Ordering::Relaxed) && issued - st.taken >= slots as u64 {
                        st.scanner_waiting = true; st = scs.wait(st).unwrap();
                    }
                    st.scanner_waiting = false;
                    if sc.load(Ordering::Relaxed) { drop(st); return finish(None, issued); }
                    st.free_syms.pop().unwrap_or_default()
                };
                match scnr.next(syms) {
                    Ok(Event::Block { bit, symbols, .. }) => {
                        let place = issued; issued += 1;
                        let mut jq = sjq.0.lock().unwrap();
                        jq.q.push_back((place, bit, symbols));
                        let wake = jq.waiting > 0;
                        drop(jq);
                        if wake { sjq.1.notify_one(); }
                    }
                    Ok(Event::Done) => return finish(None, issued),
                    Err(e) => return finish(Some(e), issued),
                }
            }
        });

        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (wr, wcr, wc, wjq, wt) = (ready.clone(), cv_reader.clone(), cancelled.clone(), job_deque.clone(), taken_atomic.clone());
            whs.push(std::thread::spawn(move || {
                let mut scratch = Scratch::<OpenCells>::default();
                loop {
                    let job = {
                        let mut jq = wjq.0.lock().unwrap();
                        loop {
                            let t = wt.load(Ordering::Relaxed);
                            if let Some(j) = pop_job(&mut jq.q, policy, t) { break Some(j); }
                            if jq.closed { break None; }
                            jq.waiting += 1; jq = wjq.1.wait(jq).unwrap(); jq.waiting -= 1;
                        }
                    };
                    let Some((place, bit, symbols)) = job else { return; };
                    if wc.load(Ordering::Relaxed) { continue; }
                    let mut out = wr.lock().unwrap().free_out.pop().unwrap_or_default();
                    out.clear();
                    if symbols.syms.len() > 2048 && out.capacity() < 960_000 { out.reserve_exact(960_000); }
                    let res = mibz2::decode_block(&symbols, k, &mut scratch, &mut out)
                        .map(|_| out)
                        .map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    let mut st = wr.lock().unwrap();
                    if !wc.load(Ordering::Relaxed) {
                        let idx = (place as usize) & st.mask;
                        st.slots_arr[idx] = Some(res);
                    }
                    if st.free_syms.len() < slots { st.free_syms.push(symbols.syms); }
                    let wake = st.reader_waiting && place == st.taken;
                    drop(st);
                    if wake { wcr.notify_one(); }
                }
            }));
        }

        let mut total = 0usize;
        let mut cur: Vec<u8> = Vec::new();
        loop {
            let blk = {
                let mut st = ready.lock().unwrap();
                let spent = std::mem::take(&mut cur);
                if spent.capacity() >= 65_536 && st.free_out.len() < slots { st.free_out.push(spent); }
                loop {
                    let t = st.taken;
                    let idx = (t as usize) & st.mask;
                    if let Some(res) = st.slots_arr[idx].take() {
                        st.taken = t + 1;
                        taken_atomic.store(t + 1, Ordering::Relaxed);
                        let wake_s = st.scanner_waiting;
                        st.reader_waiting = false;
                        drop(st);
                        if wake_s { cv_scanner.notify_one(); }
                        break Some(res);
                    }
                    if st.end == Some(st.taken) { st.reader_waiting = false; break None; }
                    st.reader_waiting = true;
                    st = cv_reader.wait(st).unwrap();
                }
            };
            match blk {
                None => break,
                Some(Ok(v)) => { on_bytes(&v); total += v.len(); cur = v; }
                Some(Err(e)) => {
                    cancelled.store(true, Ordering::Release);
                    cv_scanner.notify_all(); job_deque.1.notify_all();
                    let _ = feeder.join(); let _ = scanner_h.join();
                    for h in whs { let _ = h.join(); }
                    return Err(e);
                }
            }
        }
        let _ = feeder.join(); let _ = scanner_h.join();
        for h in whs { let _ = h.join(); }
        Ok(total)
    }

    /// Exact discrete-event 10W/16W makespan simulator (post-Mechanisms ParScan where all block offsets are known!)
    /// Tests FIFO vs LPT (sorted by syms.len() / compressed byte span!) on real measured block durations.
    pub fn sim_makespan(blocks: &[(usize, f64)], w: usize, lpt_by_proxy: bool) -> (f64, f64, f64) {
        let mut order: Vec<usize> = (0..blocks.len()).collect();
        if lpt_by_proxy {
            order.sort_by(|&a, &b| blocks[b].0.cmp(&blocks[a].0));
        }
        let mut finish = vec![0.0_f64; w];
        for idx in order {
            let (mi, _) = finish.iter().enumerate().min_by(|a, b| a.1.total_cmp(b.1)).unwrap();
            finish[mi] += blocks[idx].1;
        }
        let makespan = finish.iter().copied().fold(0.0_f64, f64::max);
        let sum: f64 = blocks.iter().map(|b| b.1).sum();
        let ideal = sum / w as f64;
        (makespan, ideal, makespan - ideal)
    }
}

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = 2.201;
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn measure_corr_and_sim(label: &str, path: &str) {
    let data = fs::read(path).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut scratch = Scratch::<OpenCells>::default();
    let mut out = Vec::with_capacity(960_000);
    let mut blocks: Vec<(usize, f64)> = Vec::new(); // (syms.len(), decode_ms)
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        let sl = symbols.syms.len();
        out.clear();
        let t0 = Instant::now();
        mibz2::decode_block(&symbols, k, &mut scratch, &mut out).unwrap();
        let dt = t0.elapsed().as_secs_f64() * 1e3;
        blocks.push((sl, dt));
        syms = symbols.syms;
    }
    // Pearson r between syms.len() and decode_ms
    let n = blocks.len() as f64;
    let mx = blocks.iter().map(|b| b.0 as f64).sum::<f64>() / n;
    let my = blocks.iter().map(|b| b.1).sum::<f64>() / n;
    let cov: f64 = blocks.iter().map(|b| (b.0 as f64 - mx) * (b.1 - my)).sum();
    let vx: f64 = blocks.iter().map(|b| (b.0 as f64 - mx).powi(2)).sum();
    let vy: f64 = blocks.iter().map(|b| (b.1 - my).powi(2)).sum();
    let r = cov / (vx * vy).sqrt();
    let max_b = blocks.iter().map(|b| b.1).fold(0.0_f64, f64::max);
    println!("\n=== PART 1: SYMS.LEN() CORRELATION & EXACT 10W/16W TAIL SIM ({label}, {} blks, max={max_b:.2}ms, Pearson r={r:.4}) ===", blocks.len());
    for &w in &[4usize, 10, 16] {
        let (f_ms, ideal, f_tail) = safe_c41::sim_makespan(&blocks, w, false);
        let (l_ms, _, l_tail) = safe_c41::sim_makespan(&blocks, w, true);
        // Scale to i9 (~0.75x) for modelled ms
        println!("  W={w:>2}: ideal={ideal:>6.2}ms | FIFO={f_ms:>6.2}ms (tail +{f_tail:.2}ms = +{:.2}%) | LPT={l_ms:>6.2}ms (tail +{l_tail:.2}ms = +{:.2}%) | saved={:.2}ms ({:.2}%)",
            100.0 * f_tail / ideal, 100.0 * l_tail / ideal, f_ms - l_ms, 100.0 * (f_ms - l_ms) / f_ms);
    }
}

fn run_arm(arm: &str, data: Vec<u8>, w: usize) -> (f64, usize) {
    let k = Kernels::detect();
    let t0 = Instant::now();
    let mut total = 0usize;
    match arm {
        "MasterA" | "MasterB" => {
            let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, 4 * w, k, true);
            let mut buf = vec![0u8; 4 << 20];
            loop { let n = rdr.read(&mut buf).unwrap(); if n == 0 { break; } total += n; }
        }
        "P1c_Fifo" => { total = safe_c41::decode_c41(Cursor::new(data), w, safe_c41::SchedPolicy::Fifo, k, |_| {}).unwrap(); }
        "S1_UrgentLpt" => { total = safe_c41::decode_c41(Cursor::new(data), w, safe_c41::SchedPolicy::UrgentPlusLpt, k, |_| {}).unwrap(); }
        "S2_HeavyOnly" => { total = safe_c41::decode_c41(Cursor::new(data), w, safe_c41::SchedPolicy::LookaheadHeavy, k, |_| {}).unwrap(); }
        _ => panic!("unknown {arm}"),
    }
    (t0.elapsed().as_secs_f64() * 1e3, total)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let w: usize = args[4].parse().unwrap();
        let (ms, tot) = run_arm(&args[3], data, w);
        println!("{ms:.3} {tot}");
        return;
    }
    if args.get(1).map(String::as_str) == Some("--sim") {
        measure_corr_and_sim("wikidata_431b_210m", "/tmp/wormhole_r5/wikidata_431b_210m.bz2");
        measure_corr_and_sim("hetero_60b", "/tmp/wormhole_r5/hetero_60b.bz2");
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let out = Command::new(&exe).arg("--sim").output().unwrap();
    print!("{}", String::from_utf8_lossy(&out.stdout));

    let arms = ["MasterA", "MasterB", "P1c_Fifo", "S1_UrgentLpt", "S2_HeavyOnly"];
    for (label, path) in [
        ("wikidata_431b_210m", "/tmp/wormhole_r5/wikidata_431b_210m.bz2"),
        ("hetero_60b", "/tmp/wormhole_r5/hetero_60b.bz2"),
    ] {
        println!("\n=== PART 2: 12-ROUND ROTATED FRESH-PROCESS 4W A/B ({label}) ===");
        let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        for r in 0..12 {
            for k in 0..arms.len() {
                let arm = arms[(r + k) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                walls.entry(arm).or_default().push(p[0].parse().unwrap());
            }
        }
        let ma = &walls["MasterA"];
        let pf = &walls["P1c_Fifo"];
        println!("{:<14} {:>8} {:>24} {:>24}", "Arm", "med_ms", "vs MasterA [95%]", "vs P1c_Fifo [95%]");
        for &arm in &arms {
            let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let (r1, l1, h1) = ci95_ratio(&ma.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
            let (r2, l2, h2) = ci95_ratio(&pf.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
            println!("{:<14} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>6.4} [{:.4}, {:.4}]", arm, wv[6], r1, l1, h1, r2, l2, h2);
        }
    }
}
