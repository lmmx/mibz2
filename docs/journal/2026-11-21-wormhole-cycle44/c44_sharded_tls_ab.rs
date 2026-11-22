use std::collections::BTreeMap;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BlockSpan, Error, ErrorKind, Kernels, Scratch};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::{InverseBwt, OpenCells};

// ---- Cycle 44: Atomic Backpressure Fast-Path (L1) & Per-Slot Sharded Ready Ring (L2) ----
mod safe_c44 {
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

    #[derive(Clone, Copy, Debug)]
    pub enum SyncMode {
        C42Base,      // Single ReadyRing mutex (3 lock ops/blk: backpressure, pop out, publish)
        L1AtomicFast, // Fuse backpressure+pop via AtomicUsize fast-path (2 lock ops/blk: pop out, publish)
        L2Sharded,    // Per-slot Mutex<Option<...>> + AtomicUsize taken/ready_bytes + separate free_out
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

    pub fn decode_c44(
        data: &[u8],
        spans: &[BlockSpan],
        workers: usize,
        mode: SyncMode,
        k: Kernels,
        mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<usize, Error> {
        let n_blocks = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let mask = cap - 1;
        const CAP_BYTES: usize = 12 << 20;

        let next_idx = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));
        let taken_at = Arc::new(AtomicUsize::new(0));
        let rbytes_at = Arc::new(AtomicUsize::new(0));
        let reader_wait_at = Arc::new(AtomicBool::new(false));
        let worker_wait_at = Arc::new(AtomicUsize::new(0));

        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(), mask, taken: 0,
            reader_waiting: false, worker_waiting: 0, ready_bytes: 0,
            free_out: Vec::with_capacity(slots),
        }));
        let sharded_slots: Arc<Vec<Mutex<Option<Result<Vec<u8>, Error>>>>> =
            Arc::new((0..cap).map(|_| Mutex::new(None)).collect());
        let sharded_free_out = Arc::new(Mutex::new(Vec::<Vec<u8>>::with_capacity(slots)));
        let wake_mu = Arc::new(Mutex::new(()));
        let cv_reader = Arc::new(Condvar::new());
        let cv_worker = Arc::new(Condvar::new());

        std::thread::scope(|s| {
            for _ in 0..workers {
                let (wr, wsh, wfo, wwm, wcr, wcw, wc, wnext, wt, wrb, wrw, www) = (
                    ready.clone(), sharded_slots.clone(), sharded_free_out.clone(), wake_mu.clone(),
                    cv_reader.clone(), cv_worker.clone(), cancelled.clone(), next_idx.clone(),
                    taken_at.clone(), rbytes_at.clone(), reader_wait_at.clone(), worker_wait_at.clone(),
                );
                s.spawn(move || {
                    let mut syms: Vec<u16> = Vec::new();
                    let mut open = OpenCells::default();
                    let mut runs = mibz2::runs::Runs::default();
                    let mut fallback = Scratch::<OpenCells>::default();
                    let mut rle_runs = Vec::with_capacity(256);
                    loop {
                        if wc.load(Ordering::Relaxed) { return; }
                        let place = wnext.fetch_add(1, Ordering::Relaxed);
                        if place >= n_blocks { return; }
                        let sp = &spans[place];

                        // Backpressure check
                        match mode {
                            SyncMode::C42Base => {
                                let mut st = wr.lock().unwrap();
                                while !wc.load(Ordering::Relaxed) && (
                                    place >= st.taken + slots ||
                                    (st.ready_bytes >= CAP_BYTES && place >= st.taken + workers + 2)
                                ) {
                                    st.worker_waiting += 1; st = wcw.wait(st).unwrap(); st.worker_waiting -= 1;
                                }
                                if wc.load(Ordering::Relaxed) { return; }
                            }
                            SyncMode::L1AtomicFast => {
                                let t = wt.load(Ordering::Acquire);
                                let rb = wrb.load(Ordering::Relaxed);
                                if place >= t + slots || (rb >= CAP_BYTES && place >= t + workers + 2) {
                                    let mut st = wr.lock().unwrap();
                                    while !wc.load(Ordering::Relaxed) && (
                                        place >= st.taken + slots ||
                                        (st.ready_bytes >= CAP_BYTES && place >= st.taken + workers + 2)
                                    ) {
                                        st.worker_waiting += 1; st = wcw.wait(st).unwrap(); st.worker_waiting -= 1;
                                    }
                                    if wc.load(Ordering::Relaxed) { return; }
                                }
                            }
                            SyncMode::L2Sharded => {
                                let t = wt.load(Ordering::Acquire);
                                let rb = wrb.load(Ordering::Relaxed);
                                if place >= t + slots || (rb >= CAP_BYTES && place >= t + workers + 2) {
                                    let mut g = wwm.lock().unwrap();
                                    www.fetch_add(1, Ordering::AcqRel);
                                    while !wc.load(Ordering::Relaxed) {
                                        let t2 = wt.load(Ordering::Acquire);
                                        let rb2 = wrb.load(Ordering::Relaxed);
                                        if place < t2 + slots && (rb2 < CAP_BYTES || place < t2 + workers + 2) { break; }
                                        g = wcw.wait(g).unwrap();
                                    }
                                    www.fetch_sub(1, Ordering::AcqRel);
                                    if wc.load(Ordering::Relaxed) { return; }
                                }
                            }
                        }

                        // Decode Huffman -> runs::front -> OpenCells -> InPlaceRLE -> CLMUL
                        let mut br = Bits::at(data, sp.start_bit);
                        let item = read_item(&mut br, sp.level, std::mem::take(&mut syms));
                        let res: Result<Vec<u8>, ErrorKind> = match item {
                            Ok(Item::Block(b)) => {
                                let is_full = b.syms.len() > 2048;
                                let mut out = if is_full {
                                    let pop = match mode {
                                        SyncMode::L2Sharded => wfo.lock().unwrap().pop(),
                                        _ => wr.lock().unwrap().free_out.pop(),
                                    };
                                    pop.unwrap_or_else(|| { let mut v = Vec::new(); v.reserve_exact(960_000); v })
                                } else { Vec::new() };
                                let r = if open.reads_runs() && mibz2::runs::takes_runs(&b) {
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
                                                out.clear(); mibz2::decode_block(&b, k, &mut fallback, &mut out).map(|_| out)
                                            }
                                        }
                                    }
                                } else {
                                    out.clear(); mibz2::decode_block(&b, k, &mut fallback, &mut out).map(|_| out)
                                };
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((kind, _)) => Err(kind),
                        };

                        let res = res.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(sp.start_bit) });
                        let out_b = res.as_ref().map_or(0, |v| v.len());

                        match mode {
                            SyncMode::C42Base | SyncMode::L1AtomicFast => {
                                let mut st = wr.lock().unwrap();
                                if !wc.load(Ordering::Relaxed) {
                                    let idx = place & st.mask;
                                    st.slots_arr[idx] = Some(res);
                                    st.ready_bytes += out_b;
                                    wrb.store(st.ready_bytes, Ordering::Relaxed);
                                }
                                let wake = st.reader_waiting && place == st.taken;
                                drop(st);
                                if wake { wcr.notify_one(); }
                            }
                            SyncMode::L2Sharded => {
                                if !wc.load(Ordering::Relaxed) {
                                    *wsh[place & mask].lock().unwrap() = Some(res);
                                    wrb.fetch_add(out_b, Ordering::Release);
                                }
                                if place == wt.load(Ordering::Acquire) && wrw.load(Ordering::Acquire) {
                                    let _g = wwm.lock().unwrap();
                                    wcr.notify_one();
                                }
                            }
                        }
                    }
                });
            }

            let mut total = 0usize;
            let mut cur: Vec<u8> = Vec::new();
            loop {
                let blk = match mode {
                    SyncMode::C42Base | SyncMode::L1AtomicFast => {
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
                                taken_at.store(t + 1, Ordering::Release);
                                rbytes_at.store(st.ready_bytes, Ordering::Relaxed);
                                let wake_w = st.worker_waiting > 0;
                                st.reader_waiting = false;
                                drop(st);
                                if wake_w { cv_worker.notify_one(); }
                                break Some(res);
                            }
                            st.reader_waiting = true; st = cv_reader.wait(st).unwrap();
                        }
                    }
                    SyncMode::L2Sharded => {
                        let spent = std::mem::take(&mut cur);
                        if spent.capacity() >= 65_536 {
                            let mut fo = sharded_free_out.lock().unwrap();
                            if fo.len() < slots { fo.push(spent); }
                        }
                        let t = taken_at.load(Ordering::Relaxed);
                        if t == n_blocks {
                            None
                        } else {
                            let idx = t & mask;
                            let res = loop {
                                if let Some(r) = sharded_slots[idx].lock().unwrap().take() {
                                    break r;
                                }
                                let mut g = wake_mu.lock().unwrap();
                                reader_wait_at.store(true, Ordering::Release);
                                if let Some(r) = sharded_slots[idx].lock().unwrap().take() {
                                    reader_wait_at.store(false, Ordering::Release);
                                    break r;
                                }
                                g = cv_reader.wait(g).unwrap();
                                reader_wait_at.store(false, Ordering::Release);
                                drop(g);
                            };
                            let blen = res.as_ref().map_or(0, |v| v.len());
                            rbytes_at.fetch_sub(blen, Ordering::AcqRel);
                            taken_at.store(t + 1, Ordering::Release);
                            if worker_wait_at.load(Ordering::Acquire) > 0 {
                                let _g = wake_mu.lock().unwrap();
                                cv_worker.notify_all();
                            }
                            Some(res)
                        }
                    }
                };
                match blk {
                    None => return Ok(total),
                    Some(Ok(v)) => { on_bytes(&v); total += v.len(); cur = v; }
                    Some(Err(e)) => { cancelled.store(true, Ordering::Release); cv_worker.notify_all(); return Err(e); }
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
    let txt = fs::read_to_string(&sp_path).unwrap();
    txt.lines().map(|l| {
        let p: Vec<u64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
        BlockSpan { index: p[0], start_bit: p[1], end_bit: p[2], crc: p[3] as u32, stream: p[4], level: p[5] as u32 }
    }).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let spans = load_spans(&args[2]);
        let w: usize = args[4].parse().unwrap();
        let mode = match args[3].as_str() {
            "C42A" | "C42B" => safe_c44::SyncMode::C42Base,
            "L1_AtomicFast" => safe_c44::SyncMode::L1AtomicFast,
            "L2_Sharded" => safe_c44::SyncMode::L2Sharded,
            _ => panic!(),
        };
        let r0 = rusage(); let t0 = Instant::now();
        let mut h = AbHash::new();
        safe_c44::decode_c44(&data, &spans, w, mode, k, |c| h.update(c)).unwrap();
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {:016x}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0, h.finish());
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let path = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    let arms = ["C42A", "C42B", "L1_AtomicFast", "L2_Sharded"];
    for &(w, rounds) in &[(4usize, 12usize), (10, 5), (16, 5)] {
        println!("\n=== {rounds}-ROUND ROTATED FRESH-PROCESS W={w} SYNC ABLATION (wikidata_431b_210m) ===");
        let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut vcsws: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut ref_h = String::new();
        for r in 0..rounds {
            for idx in 0..arms.len() {
                let arm = arms[(r + idx) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                if ref_h.is_empty() { ref_h = p[6].to_string(); }
                assert_eq!(p[6], ref_h);
                walls.entry(arm).or_default().push(p[0].parse().unwrap());
                syss.entry(arm).or_default().push(p[2].parse().unwrap());
                flts.entry(arm).or_default().push(p[3].parse().unwrap());
                vcsws.entry(arm).or_default().push(p[4].parse().unwrap());
            }
        }
        let ba = &walls["C42A"];
        println!("{:<15} {:>8} {:>24} {:>7} {:>7} {:>6}", "Arm", "med_ms", "vs C42A [95%]", "sys_ms", "minflt", "vcsw");
        for &arm in &arms {
            let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = flts[arm].clone(); fv.sort();
            let mut cv = vcsws[arm].clone(); cv.sort();
            let (rat, lo, hi) = ci95_ratio(&ba.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
            println!("{:<15} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>7.2} {:>7} {:>6}", arm, wv[rounds/2], rat, lo, hi, sv[rounds/2], fv[rounds/2], cv[rounds/2]);
        }
    }
}
