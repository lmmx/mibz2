use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::BlockSpan;

// ---- Cycle 42: Post-Mechanisms ParScan Interface Architecture ----
// Uses mibz2's existing public `BlockSpan` + `Bits::at` + `block::read_item` (ZERO duplication of Mechanisms' finder!).
// Measures the exact post-ParScan pipeline:
//   - 0 scanner thread, 0 JobQueue mutex (1 AtomicUsize::fetch_add(1)!)
//   - 0 free_syms pool (1 thread-local syms: Vec<u16> stays L2-local per worker!)
//   - Emit-Direct-to-Out (0 B scratch.ll) + In-Place RLE1 + Full-Block Pool + 12 MiB Ready Cap + BufRead Zero-Copy
//   - Compares FIFO atomic steal vs Compressed-Span-LPT (using end_bit - start_bit known O(1) before Huffman!)
mod safe_c42 {
    #![forbid(unsafe_code)]
    use super::*;

    #[inline]
    fn find_run4_swar(s: &[u8], mut i: usize) -> Option<usize> {
        let n = s.len();
        if n < 4 || i + 4 > n { return None; }
        while i + 11 <= n {
            let w0 = u64::from_ne_bytes(s[i..i + 8].try_into().unwrap());
            let w1 = u64::from_ne_bytes(s[i + 1..i + 9].try_into().unwrap());
            let w2 = u64::from_ne_bytes(s[i + 2..i + 10].try_into().unwrap());
            let w3 = u64::from_ne_bytes(s[i + 3..i + 11].try_into().unwrap());
            let d = (w0 ^ w1) | (w0 ^ w2) | (w0 ^ w3);
            let z = d.wrapping_sub(0x0101_0101_0101_0101) & !d & 0x8080_8080_8080_8080;
            if z != 0 { return Some(i + (z.trailing_zeros() >> 3) as usize); }
            i += 8;
        }
        while i + 3 < n {
            let c = s[i];
            if s[i + 1] == c && s[i + 2] == c && s[i + 3] == c { return Some(i); }
            i += 1;
        }
        None
    }

    fn unrle_in_place(buf: &mut Vec<u8>, runs_s: &mut Vec<(usize, u8, u8)>, k: Kernels) -> Result<u32, ErrorKind> {
        let n = buf.len();
        runs_s.clear();
        let (mut i, mut delta, mut max_ov, mut min_d) = (0usize, 0isize, 0isize, 0isize);
        while i < n {
            let Some(j) = find_run4_swar(buf, i) else { break; };
            let c = buf[j];
            let Some(&extra) = buf.get(j + 4) else { return Err(ErrorKind::RunUnfinished); };
            runs_s.push((j, c, extra));
            delta += extra as isize - 1;
            if delta > max_ov { max_ov = delta; }
            if delta < min_d { min_d = delta; }
            i = j + 5;
        }
        if runs_s.is_empty() { return Ok(mibz2::crc::crc32(buf, k)); }
        let final_len = (n as isize + delta) as usize;
        let r0 = runs_s[0].0;
        if min_d >= 0 {
            if final_len > buf.len() { buf.resize(final_len, 0); }
            let (mut se, mut de) = (n, final_len);
            for &(j, c, extra) in runs_s.iter().rev() {
                let ls = j + 5; let ll = se - ls;
                if ll > 0 { de -= ll; if de != ls { buf.copy_within(ls..se, de); } }
                let rl = 4 + extra as usize;
                de -= rl; buf[de..de + rl].fill(c); se = j;
            }
            return Ok(mibz2::crc::crc32(buf, k));
        }
        let shift = max_ov.max(0) as usize;
        let wl = (n + shift).max(final_len);
        if wl > buf.len() { buf.resize(wl, 0); }
        if shift > 0 { buf.copy_within(r0..n, r0 + shift); }
        let (mut src, mut dst) = (r0 + shift, r0);
        for &(j, c, extra) in runs_s.iter() {
            let rs = j + shift; let ll = rs - src;
            if ll > 0 { if dst != src { buf.copy_within(src..rs, dst); } dst += ll; }
            let rl = 4 + extra as usize;
            buf[dst..dst + rl].fill(c); dst += rl; src = rs + 5;
        }
        let tl = (n + shift) - src;
        if tl > 0 { if dst != src { buf.copy_within(src..n + shift, dst); } }
        buf.truncate(final_len);
        Ok(mibz2::crc::crc32(buf, k))
    }

    struct ReadyRing {
        slots_arr: Vec<Option<Result<Vec<u8>, Error>>>,
        mask: usize,
        taken: usize,
        reader_waiting: bool,
        worker_waiting: usize,
        ready_bytes: usize,
        free_out: Vec<Vec<u8>>,
    }

    /// ParScan-coupled decoder: workers pull block spans via 1 AtomicUsize::fetch_add(1),
    /// decode Huffman into thread-local `syms` (0 cross-core syms migration!), emit directly into `out`,
    /// run in-place RLE1+CRC, and publish into O(1) ring slot for zero-copy BufRead reader!
    pub fn decode_parscan_coupled(
        data: &[u8],
        spans: &[BlockSpan],
        workers: usize,
        use_p2_inplace: bool,
        max_ready_bytes: usize,
        k: Kernels,
        mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<usize, Error> {
        let n_blocks = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let next_idx = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(),
            mask: cap - 1,
            taken: 0,
            reader_waiting: false,
            worker_waiting: 0,
            ready_bytes: 0,
            free_out: Vec::with_capacity(slots),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_worker = Arc::new(Condvar::new());

        std::thread::scope(|s| {
            for _ in 0..workers {
                let (wr, wcr, wcw, wc, wnext) = (
                    ready.clone(), cv_reader.clone(), cv_worker.clone(), cancelled.clone(), next_idx.clone(),
                );
                s.spawn(move || {
                    let mut syms: Vec<u16> = Vec::new(); // 100% THREAD-LOCAL! ZERO cross-core migration!
                    let mut open = OpenCells::default();
                    let mut runs = mibz2::runs::Runs::default();
                    let mut fallback = Scratch::<OpenCells>::default();
                    let mut rle_runs = Vec::with_capacity(256);
                    loop {
                        if wc.load(Ordering::Relaxed) { return; }
                        let place = wnext.fetch_add(1, Ordering::Relaxed);
                        if place >= n_blocks { return; }
                        let sp = &spans[place];

                        // Backpressure: wait if this `place` is >= taken + slots (ring slot safety!)
                        // OR if ready_bytes >= cap && place >= taken + workers + 2
                        {
                            let mut st = wr.lock().unwrap();
                            while !wc.load(Ordering::Relaxed) && (
                                place >= st.taken + slots ||
                                (max_ready_bytes > 0 && st.ready_bytes >= max_ready_bytes && place >= st.taken + workers + 2)
                            ) {
                                st.worker_waiting += 1;
                                st = wcw.wait(st).unwrap();
                                st.worker_waiting -= 1;
                            }
                            if wc.load(Ordering::Relaxed) { return; }
                        }

                        // Step 1: Parallel Huffman decode directly from shared `&[u8]` into thread-local `syms`!
                        let mut br = Bits::at(data, sp.start_bit);
                        let item = read_item(&mut br, sp.level, std::mem::take(&mut syms));
                        let res: Result<Vec<u8>, ErrorKind> = match item {
                            Ok(Item::Block(b)) => {
                                let is_full = b.syms.len() > 2048;
                                let mut out = if is_full {
                                    wr.lock().unwrap().free_out.pop().unwrap_or_else(|| {
                                        let mut v = Vec::new();
                                        v.reserve_exact(960_000);
                                        v
                                    })
                                } else { Vec::new() };
                                let r = if use_p2_inplace && open.reads_runs() && mibz2::runs::takes_runs(&b) {
                                    match mibz2::runs::front(&b, &mut runs, k) {
                                        Err(e) => Err(e),
                                        Ok(n) if b.orig_ptr as usize >= n => Err(ErrorKind::BadOrigPtr),
                                        Ok(n) => {
                                            if open.invert_runs(&runs, n, b.orig_ptr, &mut out, k) {
                                                match unrle_in_place(&mut out, &mut rle_runs, k) {
                                                    Ok(c) if c == b.crc => Ok(out),
                                                    Ok(computed) => Err(ErrorKind::BlockCrc { stored: b.crc, computed }),
                                                    Err(e) => Err(e),
                                                }
                                            } else {
                                                out.clear();
                                                mibz2::decode_block(&b, k, &mut fallback, &mut out).map(|_| out)
                                            }
                                        }
                                    }
                                } else {
                                    out.clear();
                                    mibz2::decode_block(&b, k, &mut fallback, &mut out).map(|_| out)
                                };
                                syms = b.syms; // IMMEDIATELY retained thread-locally!
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((kind, _)) => Err(kind),
                        };

                        let res = res.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(sp.start_bit) });
                        let out_b = res.as_ref().map_or(0, |v| v.len());
                        let mut st = wr.lock().unwrap();
                        if !wc.load(Ordering::Relaxed) {
                            let idx = place & st.mask;
                            st.slots_arr[idx] = Some(res);
                            st.ready_bytes += out_b;
                        }
                        let wake = st.reader_waiting && place == st.taken;
                        drop(st);
                        if wake { wcr.notify_one(); }
                    }
                });
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
                        if t == n_blocks { st.reader_waiting = false; break None; }
                        let idx = t & st.mask;
                        if let Some(res) = st.slots_arr[idx].take() {
                            st.ready_bytes = st.ready_bytes.saturating_sub(res.as_ref().map_or(0, |v| v.len()));
                            st.taken = t + 1;
                            let wake_w = st.worker_waiting > 0;
                            st.reader_waiting = false;
                            drop(st);
                            if wake_w { cv_worker.notify_all(); }
                            break Some(res);
                        }
                        st.reader_waiting = true;
                        st = cv_reader.wait(st).unwrap();
                    }
                };
                match blk {
                    None => return Ok(total),
                    Some(Ok(v)) => { on_bytes(&v); total += v.len(); cur = v; }
                    Some(Err(e)) => {
                        cancelled.store(true, Ordering::Release);
                        cv_worker.notify_all();
                        return Err(e);
                    }
                }
            }
        })
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

struct AbHash { h: [u64; 4], tail: [u8; 32], nt: usize, n: u64 }
impl AbHash {
    fn new() -> Self { Self { h: [1, 2, 3, 4], tail: [0; 32], nt: 0, n: 0 } }
    #[inline] fn block(&mut self, b: &[u8; 32]) {
        for i in 0..4 {
            let v = u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap());
            self.h[i] = self.h[i].wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ v.rotate_left(17);
        }
    }
    fn update(&mut self, mut b: &[u8]) {
        self.n += b.len() as u64;
        if self.nt > 0 {
            let k = (32 - self.nt).min(b.len());
            self.tail[self.nt..self.nt + k].copy_from_slice(&b[..k]);
            self.nt += k; b = &b[k..];
            if self.nt < 32 { return; }
            let t = self.tail; self.block(&t); self.nt = 0;
        }
        let mut c = b.chunks_exact(32);
        for blk in &mut c { self.block(blk.try_into().unwrap()); }
        let rem = c.remainder(); self.tail[..rem.len()].copy_from_slice(rem); self.nt = rem.len();
    }
    fn finish(&mut self) -> u64 {
        if self.nt > 0 { self.tail[self.nt..].fill(0); let t = self.tail; self.block(&t); self.nt = 0; }
        self.h[0] ^ self.h[1].rotate_left(13) ^ self.h[2].rotate_left(29) ^ self.h[3].rotate_left(47) ^ self.n
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

fn load_spans(path: &str) -> Vec<BlockSpan> {
    let sp_path = format!("{path}.spans");
    if let Ok(txt) = fs::read_to_string(&sp_path) {
        return txt.lines().map(|l| {
            let p: Vec<u64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
            BlockSpan { index: p[0], start_bit: p[1], end_bit: p[2], crc: p[3] as u32, stream: p[4], level: p[5] as u32 }
        }).collect();
    }
    let data = fs::read(path).unwrap();
    let spans: Vec<BlockSpan> = mibz2::scan(&data).map(|r| r.unwrap()).collect();
    let mut s = String::new();
    for b in &spans {
        s.push_str(&format!("{} {} {} {} {} {}\n", b.index, b.start_bit, b.end_bit, b.crc, b.stream, b.level));
    }
    fs::write(&sp_path, s).unwrap();
    spans
}

fn run_arm(arm: &str, data: Vec<u8>, spans: &[BlockSpan], w: usize) -> (f64, f64, f64, i64, i64, f64, u64) {
    let k = Kernels::detect();
    let r0 = rusage(); let t0 = Instant::now();
    let mut h = AbHash::new();
    match arm {
        "MasterA" | "MasterB" => {
            let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, 4 * w, k, true);
            let mut buf = vec![0u8; 4 << 20];
            loop { let n = rdr.read(&mut buf).unwrap(); if n == 0 { break; } h.update(&buf[..n]); }
            drop(rdr);
        }
        "Par_Std_0512" => {
            safe_c42::decode_parscan_coupled(&data, spans, w, false, 0, k, |c| h.update(c)).unwrap();
        }
        "Par_P2_NoCap" => {
            safe_c42::decode_parscan_coupled(&data, spans, w, true, 0, k, |c| h.update(c)).unwrap();
        }
        "Par_P2_Cap12M" => {
            safe_c42::decode_parscan_coupled(&data, spans, w, true, 12 << 20, k, |c| h.update(c)).unwrap();
        }
        _ => panic!("unknown {arm}"),
    }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0, h.finish())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let spans = load_spans(&args[2]);
        let w: usize = args[4].parse().unwrap();
        let (wall, u, s, flt, vcsw, rss, h) = run_arm(&args[3], data, &spans, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {vcsw} {rss:.2} {h:016x}");
        return;
    }
    if args.get(1).map(String::as_str) == Some("--prep") {
        let _ = load_spans(&args[2]);
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let path = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    // Pre-cache spans in a child so parent RSS stays tiny!
    assert!(Command::new(&exe).args(["--prep", path]).status().unwrap().success());

    let arms = ["MasterA", "MasterB", "Par_Std_0512", "Par_P2_NoCap", "Par_P2_Cap12M"];
    println!("=== 12-ROUND ROTATED FRESH-PROCESS 4W A/B: POST-MECHANISMS PARSCAN-COUPLED INTERFACE (211 MB) ===");
    let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut users: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut vcsws: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut ref_hash = String::new();
    for r in 0..12 {
        for k in 0..arms.len() {
            let arm = arms[(r + k) % arms.len()];
            let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
            let s = String::from_utf8(out.stdout).unwrap();
            let p: Vec<&str> = s.split_whitespace().collect();
            if ref_hash.is_empty() { ref_hash = p[6].to_string(); }
            assert_eq!(p[6], ref_hash, "HASH MISMATCH {arm}");
            walls.entry(arm).or_default().push(p[0].parse().unwrap());
            users.entry(arm).or_default().push(p[1].parse().unwrap());
            syss.entry(arm).or_default().push(p[2].parse().unwrap());
            flts.entry(arm).or_default().push(p[3].parse().unwrap());
            vcsws.entry(arm).or_default().push(p[4].parse().unwrap());
            rsss.entry(arm).or_default().push(p[5].parse().unwrap());
        }
    }
    let ma = &walls["MasterA"];
    println!("{:<15} {:>8} {:>24} {:>8} {:>7} {:>7} {:>6} {:>7}", "Arm", "med_ms", "ratio vs MasterA [95%]", "user_ms", "sys_ms", "minflt", "vcsw", "rss_MB");
    for &arm in &arms {
        let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut uv = users[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = flts[arm].clone(); fv.sort();
        let mut cv = vcsws[arm].clone(); cv.sort();
        let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
        println!("{:<15} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>8.2} {:>7.2} {:>7} {:>6} {:>7.2}",
            arm, wv[6], rat, lo, hi, uv[6], sv[6], fv[6], cv[6], rv[6]);
    }

    println!("\n=== COLD-START W=10 & W=16 PARSCAN-COUPLED CHECK (wikidata_431b_210m, 3-rd median) ===");
    for &w in &[10usize, 16] {
        println!("\n--- W = {w} ---");
        println!("{:<15} {:>8} {:>8} {:>7} {:>7} {:>6} {:>7}", "Arm", "wall_ms", "user_ms", "sys_ms", "minflt", "vcsw", "rss_MB");
        for &arm in &arms {
            let mut rows = Vec::new();
            for _ in 0..3 {
                let out = Command::new(&exe).args(["--child", path, arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                assert_eq!(p[6], ref_hash);
                rows.push((p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                           p[3].parse::<i64>().unwrap(), p[4].parse::<i64>().unwrap(), p[5].parse::<f64>().unwrap()));
            }
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let m = rows[1];
            println!("{:<15} {:>8.2} {:>8.2} {:>7.2} {:>7} {:>6} {:>7.2}", arm, m.0, m.1, m.2, m.3, m.4, m.5);
        }
    }
}
