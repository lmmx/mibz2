use std::collections::BTreeMap;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BlockSpan, Error, ErrorKind, Kernels, Scratch};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::OpenCells;

// ---- Cycle 47: Parallel calloc + W-Way Scatter Flatten for API-Identical `decode_parallel -> Result<Vec<u8>>` ----
mod safe_c47 {
    #![forbid(unsafe_code)]
    use super::*;

    /// Decode all blocks in parallel (ParScan-coupled TLS syms) into owned Vec<Vec<u8>> (0 serial copy!)
    pub fn decode_to_chunks(data: &[u8], spans: &[BlockSpan], workers: usize, k: Kernels) -> Result<Vec<Vec<u8>>, Error> {
        let n = spans.len();
        let next_idx = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));
        let slots: Vec<Mutex<Option<Result<Vec<u8>, Error>>>> = (0..n).map(|_| Mutex::new(None)).collect();

        std::thread::scope(|s| {
            let slots_ref = &slots;
            for _ in 0..workers {
                let (wnext, wc) = (next_idx.clone(), cancelled.clone());
                s.spawn(move || {
                    let mut syms: Vec<u16> = Vec::new();
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        if wc.load(Ordering::Relaxed) { return; }
                        let place = wnext.fetch_add(1, Ordering::Relaxed);
                        if place >= n { return; }
                        let sp = &spans[place];
                        let mut br = Bits::at(data, sp.start_bit);
                        let res = match read_item(&mut br, sp.level, std::mem::take(&mut syms)) {
                            Ok(Item::Block(b)) => {
                                let mut out = if b.syms.len() > 2048 {
                                    let mut v = Vec::new();
                                    v.reserve_exact(960_000);
                                    v
                                } else { Vec::new() };
                                let r = mibz2::decode_block(&b, k, &mut scratch, &mut out).map(|_| out);
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((kind, _)) => Err(kind),
                        };
                        let res = res.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(sp.start_bit) });
                        if res.is_err() { wc.store(true, Ordering::Release); }
                        *slots_ref[place].lock().unwrap() = Some(res);
                    }
                });
            }
        });

        let mut chunks = Vec::with_capacity(n);
        for m in slots {
            chunks.push(m.into_inner().unwrap().unwrap()?);
        }
        Ok(chunks)
    }

    /// F1: API-identical `Result<Vec<u8>>` via `vec![0u8; total]` (Linux demand-zero calloc!) + W-way parallel scatter!
    pub fn decode_parallel_calloc_scatter(data: &[u8], spans: &[BlockSpan], workers: usize, k: Kernels) -> Result<Vec<u8>, Error> {
        let chunks = decode_to_chunks(data, spans, workers, k)?;
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        // `vec![0u8; total]` uses calloc -> single anonymous mmap in ~3 us (0 serial zero-fill!)
        let mut out = vec![0u8; total];
        // Split `out` into `workers` disjoint byte ranges aligned to block boundaries using safe `split_at_mut`!
        let mut ranges: Vec<(&[Vec<u8>], &mut [u8])> = Vec::with_capacity(workers);
        let mut rem_chunks = &chunks[..];
        let mut rem_out = &mut out[..];
        let target_per_w = (total + workers - 1) / workers;
        for w_i in 0..workers {
            if rem_chunks.is_empty() { break; }
            if w_i + 1 == workers {
                ranges.push((rem_chunks, rem_out));
                break;
            }
            let mut acc = 0usize;
            let mut cnt = 0usize;
            while cnt < rem_chunks.len() && (acc < target_per_w || cnt == 0) {
                acc += rem_chunks[cnt].len();
                cnt += 1;
            }
            let (c_head, c_tail) = rem_chunks.split_at(cnt);
            let (o_head, o_tail) = rem_out.split_at_mut(acc);
            ranges.push((c_head, o_head));
            rem_chunks = c_tail;
            rem_out = o_tail;
        }
        std::thread::scope(|s| {
            for (c_slice, o_slice) in ranges {
                s.spawn(move || {
                    let mut at = 0usize;
                    for blk in c_slice {
                        o_slice[at..at + blk.len()].copy_from_slice(blk);
                        at += blk.len();
                    }
                });
            }
        });
        drop(chunks);
        Ok(out)
    }

    /// R1_ParScan_SerialExtend: ParScan-coupled decode -> serial out.extend_from_slice (pre-sized 8x) with buffer recycling!
    pub fn decode_parallel_serial_extend(data: &[u8], spans: &[BlockSpan], workers: usize, k: Kernels) -> Result<Vec<u8>, Error> {
        let n = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let next_idx = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));
        struct Ready { arr: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize, taken: usize, rw: bool, ww: usize, free: Vec<Vec<u8>> }
        let ready = Arc::new(Mutex::new(Ready {
            arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0, rw: false, ww: 0, free: Vec::with_capacity(slots),
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let mut out = Vec::with_capacity(data.len().saturating_mul(16)); // generous guess for Wikidata

        std::thread::scope(|s| {
            for _ in 0..workers {
                let (wr, wcr, wcw, wc, wnext) = (ready.clone(), cv_r.clone(), cv_w.clone(), cancelled.clone(), next_idx.clone());
                s.spawn(move || {
                    let mut syms: Vec<u16> = Vec::new();
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        if wc.load(Ordering::Relaxed) { return; }
                        let place = wnext.fetch_add(1, Ordering::Relaxed);
                        if place >= n { return; }
                        let sp = &spans[place];
                        {
                            let mut st = wr.lock().unwrap();
                            while !wc.load(Ordering::Relaxed) && place >= st.taken + slots {
                                st.ww += 1; st = wcw.wait(st).unwrap(); st.ww -= 1;
                            }
                            if wc.load(Ordering::Relaxed) { return; }
                        }
                        let mut br = Bits::at(data, sp.start_bit);
                        let res = match read_item(&mut br, sp.level, std::mem::take(&mut syms)) {
                            Ok(Item::Block(b)) => {
                                let mut buf = if b.syms.len() > 2048 {
                                    wr.lock().unwrap().free.pop().unwrap_or_else(|| { let mut v = Vec::new(); v.reserve_exact(960_000); v })
                                } else { Vec::new() };
                                buf.clear();
                                let r = mibz2::decode_block(&b, k, &mut scratch, &mut buf).map(|_| buf);
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((kind, _)) => Err(kind),
                        }.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(sp.start_bit) });
                        let mut st = wr.lock().unwrap();
                        if !wc.load(Ordering::Relaxed) { let idx = place & st.mask; st.arr[idx] = Some(res); }
                        let wake = st.rw && place == st.taken;
                        drop(st);
                        if wake { wcr.notify_one(); }
                    }
                });
            }
            let mut cur: Vec<u8> = Vec::new();
            loop {
                let blk = {
                    let mut st = ready.lock().unwrap();
                    let sp = std::mem::take(&mut cur);
                    if sp.capacity() >= 65_536 && st.free.len() < slots { st.free.push(sp); }
                    loop {
                        let t = st.taken;
                        if t == n { st.rw = false; break None; }
                        let idx = t & st.mask;
                        if let Some(res) = st.arr[idx].take() {
                            st.taken = t + 1;
                            let ww = st.ww > 0; st.rw = false; drop(st);
                            if ww { cv_w.notify_one(); }
                            break Some(res);
                        }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk {
                    None => return Ok(()),
                    Some(Ok(v)) => { out.extend_from_slice(&v); cur = v; }
                    Some(Err(e)) => { cancelled.store(true, Ordering::Release); cv_w.notify_all(); return Err(e); }
                }
            }
        })?;
        Ok(out)
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
    let t = 2.201;
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn load_spans(path: &str) -> Vec<BlockSpan> {
    let txt = fs::read_to_string(format!("{path}.spans")).unwrap();
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
        let r0 = rusage(); let t0 = Instant::now();
        let len = match args[3].as_str() {
            "MasterA" | "MasterB" => {
                let dec = mibz2::StreamDecoder::workers(w, 4 * w, k, true);
                let mut out = Vec::with_capacity(data.len().saturating_mul(4));
                std::thread::scope(|s| {
                    s.spawn(|| {
                        for c in data.chunks(4 << 20) { if dec.write(c.to_vec()).is_err() { return; } }
                        dec.close();
                    });
                    let mut buf = vec![0u8; 4 << 20];
                    loop { match dec.read(&mut buf).unwrap() { 0 => break, n => out.extend_from_slice(&buf[..n]) } }
                });
                let l = out.len(); std::hint::black_box(&out); l
            }
            "Par_SerialExt" => {
                let out = safe_c47::decode_parallel_serial_extend(&data, &spans, w, k).unwrap();
                let l = out.len(); std::hint::black_box(&out); l
            }
            "F1_CallocScat" => {
                let out = safe_c47::decode_parallel_calloc_scatter(&data, &spans, w, k).unwrap();
                let l = out.len(); std::hint::black_box(&out); l
            }
            "R2_Chunks" => {
                let ch = safe_c47::decode_to_chunks(&data, &spans, w, k).unwrap();
                let l: usize = ch.iter().map(|v| v.len()).sum(); std::hint::black_box(&ch); l
            }
            _ => panic!(),
        };
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {:.2} {len}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0);
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let path = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    let arms = ["MasterA", "MasterB", "Par_SerialExt", "F1_CallocScat", "R2_Chunks"];
    println!("=== 12-ROUND ROTATED FRESH-PROCESS 4W A/B: decode_parallel CONTIGUOUS vs SCATTER vs CHUNKS (211 MB) ===");
    let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut users: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for r in 0..12 {
        for i in 0..arms.len() {
            let arm = arms[(r + i) % arms.len()];
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
    let ma = &walls["MasterA"];
    println!("{:<15} {:>8} {:>24} {:>8} {:>7} {:>7} {:>7}", "Arm", "med_ms", "vs MasterA [95%]", "user_ms", "sys_ms", "minflt", "rss_MB");
    for &arm in &arms {
        let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut uv = users[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = flts[arm].clone(); fv.sort();
        let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
        println!("{:<15} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>8.2} {:>7.2} {:>7} {:>7.2}", arm, wv[6], rat, lo, hi, uv[6], sv[6], fv[6], rv[6]);
    }
}
