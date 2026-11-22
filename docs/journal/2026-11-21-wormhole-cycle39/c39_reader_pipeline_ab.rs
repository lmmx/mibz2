use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::{self, Cursor, IoSlice, Read, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, Source};

// ---- Cycle 39: Reader & Consumer Output Assembly (decode_parallel & bzcat write_vectored) ----
mod safe_c39 {
    #![forbid(unsafe_code)]
    use super::*;

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

    /// Slice-based zero-copy parallel decoder (no feeder thread, no 4 MiB chunk copies when input is &[u8]!)
    /// + direct block handoff to consumer closure!
    pub fn decode_slice_p1c(
        data: &[u8],
        workers: usize,
        k: Kernels,
        mut on_block: impl FnMut(Vec<u8>) -> Option<Vec<u8>>, // returns buffer to recycle, or None if consumer takes ownership!
    ) -> Result<(), Error> {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let cancelled = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0, end: None,
            reader_waiting: false, scanner_waiting: false,
            free_out: Vec::with_capacity(slots), free_syms: Vec::with_capacity(slots),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_scanner = Arc::new(Condvar::new());
        let job_deque = Arc::new((Mutex::new(JobQueue { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }), Condvar::new()));

        std::thread::scope(|s| {
            let (sr, scr, scs, sc, sjq) = (ready.clone(), cv_reader.clone(), cv_scanner.clone(), cancelled.clone(), job_deque.clone());
            s.spawn(move || {
                let mut scnr = Scanner::new(mibz2::stream::SliceSource(data));
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

            for _ in 0..workers {
                let (wr, wcr, wc, wjq) = (ready.clone(), cv_reader.clone(), cancelled.clone(), job_deque.clone());
                s.spawn(move || {
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        let job = {
                            let mut jq = wjq.0.lock().unwrap();
                            loop {
                                if let Some(j) = jq.q.pop_front() { break Some(j); }
                                if jq.closed { break None; }
                                jq.waiting += 1; jq = wjq.1.wait(jq).unwrap(); jq.waiting -= 1;
                            }
                        };
                        let Some((place, bit, symbols)) = job else { return; };
                        if wc.load(Ordering::Relaxed) { continue; }
                        let mut out = wr.lock().unwrap().free_out.pop().unwrap_or_default();
                        out.clear();
                        if symbols.syms.len() > 2048 && out.capacity() < 960_000 {
                            out.reserve_exact(960_000);
                        }
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
                });
            }

            let mut recycle: Option<Vec<u8>> = None;
            loop {
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if let Some(v) = recycle.take() {
                        if v.capacity() >= 65_536 && st.free_out.len() < slots { st.free_out.push(v); }
                    }
                    loop {
                        let t = st.taken;
                        let idx = (t as usize) & st.mask;
                        if let Some(res) = st.slots_arr[idx].take() {
                            st.taken = t + 1;
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
                    None => return Ok(()),
                    Some(Ok(v)) => { recycle = on_block(v); }
                    Some(Err(e)) => {
                        cancelled.store(true, Ordering::Release);
                        cv_scanner.notify_all(); job_deque.1.notify_all();
                        return Err(e);
                    }
                }
            }
        })
    }

    /// R1: Direct-Extend contiguous Vec<u8> (drops 4 MiB bounce buf + 1 full 211 MB copy + slice input!)
    pub fn decode_parallel_direct(data: &[u8], workers: usize, k: Kernels) -> Result<Vec<u8>, Error> {
        // Better initial guess: 8x compressed (still safe for small files) vs master's 4x
        let mut out = Vec::with_capacity(data.len().saturating_mul(8));
        decode_slice_p1c(data, workers, k, |blk| {
            out.extend_from_slice(&blk);
            Some(blk) // recycle `blk` immediately!
        })?;
        Ok(out)
    }

    /// R2: Zero-Copy Owned Chunked Output Vec<Vec<u8>> (ZERO bytes copied on reader thread!)
    pub fn decode_parallel_chunks(data: &[u8], workers: usize, k: Kernels) -> Result<Vec<Vec<u8>>, Error> {
        let mut chunks = Vec::with_capacity(512);
        decode_slice_p1c(data, workers, k, |blk| {
            chunks.push(blk);
            None // consumer owns the decoded block directly — ZERO memcpy!
        })?;
        Ok(chunks)
    }

    /// R3: Zero-Copy Sink / `write_all` (e.g. `bzcat` / `/dev/null` / pipe write directly from `&blk`, ZERO intermediate 16 MiB copy!)
    pub fn decode_to_writer<W: Write>(data: &[u8], workers: usize, k: Kernels, mut w: W) -> Result<usize, Error> {
        let mut total = 0usize;
        decode_slice_p1c(data, workers, k, |blk| {
            let _ = w.write_all(&blk);
            total += blk.len();
            Some(blk)
        })?;
        Ok(total)
    }
    #[allow(dead_code)]
    fn _use_ioslice(_: IoSlice<'_>, _: PooledSource) {}
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval,
    ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64,
    ru_inblock: i64, ru_oublock: i64, ru_msgsnd: i64, ru_msgrcv: i64,
    ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
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
    let t = 2.201;
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn run_arm(arm: &str, data: &[u8], w: usize) -> (f64, f64, f64, i64, f64, usize) {
    let k = Kernels::detect();
    let r0 = rusage();
    let t0 = Instant::now();
    let len = match arm {
        "MasterA_dec_par" | "MasterB_dec_par" => {
            // Force OpenCells at `w` workers via StreamDecoder::workers + exact master decode_parallel loop!
            let dec = mibz2::StreamDecoder::workers(w, 4 * w, k, true);
            let mut out = Vec::with_capacity(data.len().saturating_mul(4));
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in data.chunks(4 << 20) {
                        if dec.write(c.to_vec()).is_err() { return; }
                    }
                    dec.close();
                });
                let mut buf = vec![0u8; 4 << 20];
                loop {
                    match dec.read(&mut buf).unwrap() {
                        0 => break,
                        n => out.extend_from_slice(&buf[..n]),
                    }
                }
            });
            let n = out.len();
            std::hint::black_box(&out);
            n
        }
        "R1_DirectExtend" => {
            let out = safe_c39::decode_parallel_direct(data, w, k).unwrap();
            let n = out.len();
            std::hint::black_box(&out);
            n
        }
        "R2_ZeroCopyChunks" => {
            let chunks = safe_c39::decode_parallel_chunks(data, w, k).unwrap();
            let n: usize = chunks.iter().map(|v| v.len()).sum();
            std::hint::black_box(&chunks);
            n
        }
        "R3_ZeroCopySink" => {
            let f = std::fs::OpenOptions::new().write(true).open("/dev/null").unwrap();
            safe_c39::decode_to_writer(data, w, k, f).unwrap()
        }
        "Master_bzcat_null" => {
            let dec = mibz2::StreamDecoder::workers(w, 4 * w, k, true);
            let mut f = std::fs::OpenOptions::new().write(true).open("/dev/null").unwrap();
            let mut total = 0usize;
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in data.chunks(8 << 20) {
                        if dec.write(c.to_vec()).is_err() { return; }
                    }
                    dec.close();
                });
                let mut buf = vec![0u8; 16 << 20];
                loop {
                    match dec.read(&mut buf).unwrap() {
                        0 => break,
                        n => { f.write_all(&buf[..n]).unwrap(); total += n; }
                    }
                }
            });
            total
        }
        _ => panic!("unknown arm {arm}"),
    };
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0, len)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let w: usize = args[4].parse().unwrap();
        let (wall, u, s, flt, rss, len) = run_arm(&args[3], &data, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {rss:.2} {len}");
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let path = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    let arms = ["MasterA_dec_par", "MasterB_dec_par", "R1_DirectExtend", "R2_ZeroCopyChunks", "Master_bzcat_null", "R3_ZeroCopySink"];
    println!("=== 12-ROUND ROTATED FRESH-PROCESS 4W A/B: decode_parallel & bzcat OUTPUT ASSEMBLY (211 MB) ===");
    let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut users: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for r in 0..12 {
        for k in 0..arms.len() {
            let arm = arms[(r + k) % arms.len()];
            let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
            let s = String::from_utf8(out.stdout).unwrap();
            let p: Vec<&str> = s.split_whitespace().collect();
            assert_eq!(p[5], "211000232");
            walls.entry(arm).or_default().push(p[0].parse().unwrap());
            users.entry(arm).or_default().push(p[1].parse().unwrap());
            syss.entry(arm).or_default().push(p[2].parse().unwrap());
            flts.entry(arm).or_default().push(p[3].parse().unwrap());
            rsss.entry(arm).or_default().push(p[4].parse().unwrap());
        }
    }
    let ma = &walls["MasterA_dec_par"];
    let mbz = &walls["Master_bzcat_null"];
    println!("{:<18} {:>8} {:>24} {:>8} {:>7} {:>7} {:>7}", "Arm", "med_ms", "ratio vs Base [95%]", "user_ms", "sys_ms", "minflt", "rss_MB");
    for &arm in &arms {
        let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut uv = users[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = flts[arm].clone(); fv.sort();
        let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let base = if arm == "R3_ZeroCopySink" || arm == "Master_bzcat_null" { mbz } else { ma };
        let pairs: Vec<(f64, f64)> = base.iter().copied().zip(walls[arm].iter().copied()).collect();
        let (rat, lo, hi) = ci95_ratio(&pairs);
        println!("{:<18} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>8.2} {:>7.2} {:>7} {:>7.2}",
            arm, wv[6], rat, lo, hi, uv[6], sv[6], fv[6], rv[6]);
    }
}
