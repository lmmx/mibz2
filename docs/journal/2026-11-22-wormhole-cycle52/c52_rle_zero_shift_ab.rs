use std::collections::BTreeMap;
use std::fs;
use std::process::Command;
use std::time::Instant;
use mibz2::{ErrorKind, Kernels, StreamDecoder};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::stream::{Event, Scanner, SliceSource};

// ---- Cycle 52: Theorem 52.1 Skip-Free Zero-Shift In-Place RLE1 vs #739 src/rle.rs:138 Double-Shift ----
mod safe_c52 {
    #![forbid(unsafe_code)]
    use super::*;

    pub type Run = (u32, u8, u8);
    const TILE: usize = 64 << 10;

    /// Exact #739 src/rle.rs:103-170 logic (given pre-found runs from AVX2 or scalar)
    pub fn unrle_739_shift(buf: &mut Vec<u8>, runs: &[Run], k: Kernels) -> (u32, u64) {
        let n = buf.len();
        let Some(&(first, _, _)) = runs.first() else {
            return (mibz2::crc::crc32(buf, k), 0);
        };
        let (mut grown, mut shift) = (0isize, 0isize);
        for &(_, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            shift = shift.max(grown);
        }
        let (shift, len) = (shift as usize, (n as isize + grown) as usize);
        let first = first as usize;
        let mut copied: u64 = 0;
        if shift > 0 {
            if n + shift > buf.capacity() { buf.reserve_exact(n + shift - buf.len()); }
            buf.resize(n + shift, 0);
            buf.copy_within(first..n, first + shift);
            copied += (n - first) as u64;
        }
        let (mut s, mut at) = (!0u32, 0usize);
        let mut tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= TILE {
                *s = mibz2::crc::update_with(*s, &buf[*at..*at + TILE], k);
                *at += TILE;
            }
        };
        tiles(&mut s, &mut at, buf, first);
        let (mut src, mut dst) = (first + shift, first);
        for &(j, c, extra) in runs.iter() {
            let from = j as usize + shift;
            if from > src {
                if dst != src {
                    buf.copy_within(src..from, dst);
                    copied += (from - src) as u64;
                }
                dst += from - src;
            }
            let run = 4 + extra as usize;
            buf[dst..dst + run].fill(c);
            dst += run;
            src = from + 5;
            tiles(&mut s, &mut at, buf, dst);
        }
        if n + shift > src && dst != src {
            buf.copy_within(src..n + shift, dst);
            copied += (n + shift - src) as u64;
        }
        buf.truncate(len);
        s = mibz2::crc::update_with(s, &buf[at..], k);
        (!s, copied)
    }

    /// Theorem 52.1: Skip-Free Zero-Shift In-Place RLE1 Expansion!
    /// Decomposes runs at g=0 crossings into disjoint valleys (g <= 0, left-to-right) and hills (g > 0, right-to-left).
    /// ZERO pre-shift copy, ZERO s.ll fallback, exact single copy per moved literal!
    pub fn unrle_zero_shift(buf: &mut Vec<u8>, runs: &[Run], k: Kernels) -> (u32, u64) {
        let n = buf.len();
        if runs.is_empty() {
            return (mibz2::crc::crc32(buf, k), 0);
        }
        let mut grown = 0isize;
        let mut max_end = n;
        for &(j, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            let end_after = (j as usize + 5) as isize + grown;
            if end_after as usize > max_end { max_end = end_after as usize; }
        }
        let final_len = (n as isize + grown) as usize;
        if final_len > max_end { max_end = final_len; }
        if max_end > buf.len() {
            if max_end > buf.capacity() { buf.reserve_exact(max_end - buf.len()); }
            buf.resize(max_end, 0);
        }

        let mut copied: u64 = 0;
        let (mut crc_s, mut crc_at) = (!0u32, 0usize);
        let mut crc_flush = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= TILE {
                *s = mibz2::crc::update_with(*s, &buf[*at..*at + TILE], k);
                *at += TILE;
            }
        };

        let rlen = runs.len();
        let mut idx = 0usize;
        let mut src_pos = 0usize; // start of next unprocessed input byte
        let mut g: isize = 0;     // current output offset - input offset at src_pos

        while idx < rlen {
            // Check whether run `idx` enters a hill (g_after > 0) or stays in a valley (g_after <= 0)
            let (j0, c0, e0) = runs[idx];
            let j0u = j0 as usize;
            let g_after0 = g + (e0 as isize - 1);

            if g <= 0 && g_after0 <= 0 {
                // Valley step: literal [src_pos..j0u) has g <= 0 -> dst <= src, safe left-to-right!
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos {
                    buf.copy_within(src_pos..j0u, dst_lit);
                    copied += (j0u - src_pos) as u64;
                }
                let dst_run = (j0u as isize + g) as usize;
                let rl = 4 + e0 as usize;
                buf[dst_run..dst_run + rl].fill(c0);
                src_pos = j0u + 5;
                g = g_after0;
                idx += 1;
                crc_flush(&mut crc_s, &mut crc_at, buf, dst_run + rl);
            } else {
                // Entering a HILL: first flush the literal [src_pos..j0u) with g <= 0 left-to-right
                debug_assert!(g <= 0 && g_after0 > 0);
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos {
                    buf.copy_within(src_pos..j0u, dst_lit);
                    copied += (j0u - src_pos) as u64;
                }
                crc_flush(&mut crc_s, &mut crc_at, buf, (j0u as isize + g) as usize);

                // Scan forward to find the end of this hill:
                // Hill ends at the first run `end_idx` where g returns to 0 (skip-free downwards: g_before=1, extra=0 -> g_after=0),
                // OR at end of all runs (rlen) if g stays > 0 through EOF.
                let hill_start = idx;
                let g_start = g; // <= 0
                let mut cur_g = g_after0; // > 0
                let mut k_i = idx + 1;
                let mut closed_by_run = false;
                while k_i < rlen {
                    let (_, _, ek) = runs[k_i];
                    let next_g = cur_g + (ek as isize - 1);
                    if next_g == 0 {
                        // cur_g == 1 and ek == 0! Run k_i closes the hill cleanly at output == input == j_k + 5!
                        closed_by_run = true;
                        break;
                    }
                    cur_g = next_g;
                    k_i += 1;
                }

                // Determine the rightmost input & output boundaries of this hill
                let (hill_end_idx, mut se, mut de, next_src, next_g) = if closed_by_run {
                    let jk = runs[k_i].0 as usize;
                    // Run k_i itself ends at dst = (jk + 1) + 4 = jk + 5 == src!
                    (k_i + 1, jk + 5, jk + 5, jk + 5, 0isize)
                } else {
                    // Hill runs to end of block `n` with final offset `cur_g > 0`
                    (rlen, n, (n as isize + cur_g) as usize, n, cur_g)
                };

                // Walk the hill RIGHT-TO-LEFT from `hill_end_idx - 1` down to `hill_start`!
                // Every literal inside the hill has g > 0 -> dst > src, so right-to-left is 100% non-clobbering!
                for r_i in (hill_start..hill_end_idx).rev() {
                    let (rj, rc, re) = runs[r_i];
                    let rju = rj as usize;
                    let lit_s = rju + 5;
                    if se > lit_s {
                        let ll = se - lit_s;
                        de -= ll;
                        if de != lit_s {
                            buf.copy_within(lit_s..se, de);
                            copied += ll as u64;
                        }
                    }
                    let rl = 4 + re as usize;
                    de -= rl;
                    buf[de..de + rl].fill(rc);
                    se = rju;
                }
                debug_assert_eq!(de, (j0u as isize + g_start) as usize);

                // Flush CRC across the completed hill (bytes [crc_at..hill_dst_end) are now final!)
                let hill_dst_end = if closed_by_run { runs[k_i].0 as usize + 5 } else { (n as isize + cur_g) as usize };
                crc_flush(&mut crc_s, &mut crc_at, buf, hill_dst_end);

                idx = hill_end_idx;
                src_pos = next_src;
                g = next_g;
            }
        }

        // Trailing literal [src_pos..n) if we ended in a valley (g <= 0)
        if src_pos < n {
            let dst_lit = (src_pos as isize + g) as usize;
            if dst_lit != src_pos {
                buf.copy_within(src_pos..n, dst_lit);
                copied += (n - src_pos) as u64;
            }
        }
        buf.truncate(final_len);
        crc_s = mibz2::crc::update_with(crc_s, &buf[crc_at..], k);
        (!crc_s, copied)
    }

    pub fn find_runs_scalar(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let mut i = 0;
        while i + 3 < n {
            let c = rle[i];
            if rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
                let Some(&extra) = rle.get(i + 4) else { return Err(ErrorKind::RunUnfinished); };
                runs.push((i as u32, c, extra));
                i += 5;
            } else { i += 1; }
        }
        Ok(())
    }
}

fn verify_theorem_52_1(k: Kernels) {
    let mut seed: u64 = 0x52_2026_1122;
    let mut next = || -> u64 {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; seed
    };
    let mut runs = Vec::new();
    for tc in 0..5000usize {
        let len = 5 + (next() as usize % 2048);
        let mut raw = Vec::with_capacity(len);
        while raw.len() < len {
            let mode = next() % 10;
            if mode < 3 && raw.len() + 5 <= len {
                let c = (next() % 8) as u8;
                let extra = match next() % 5 {
                    0 | 1 => 0u8, // shrinking delta = -1
                    2 => 1u8,     // neutral delta = 0
                    3 => (2 + (next() % 12)) as u8, // small hill
                    _ => (next() % 256) as u8,      // large hill (even > n!)
                };
                raw.extend_from_slice(&[c, c, c, c, extra]);
            } else {
                raw.push((next() % 256) as u8);
            }
        }
        // Fix any accidental unfinished 4-byte run at end
        if raw.len() >= 4 {
            let m = raw.len();
            if raw[m-4] == raw[m-3] && raw[m-3] == raw[m-2] && raw[m-2] == raw[m-1] {
                raw[m-1] ^= 1;
            }
        }
        if safe_c52::find_runs_scalar(&raw, &mut runs).is_err() { continue; }
        // Reference scalar expansion
        let mut ref_out = Vec::new();
        let mut i = 0;
        while i < raw.len() {
            let c = raw[i];
            if i + 3 < raw.len() && raw[i+1] == c && raw[i+2] == c && raw[i+3] == c {
                let extra = raw[i+4] as usize;
                for _ in 0..(4 + extra) { ref_out.push(c); }
                i += 5;
            } else { ref_out.push(c); i += 1; }
        }
        let ref_crc = mibz2::crc::crc32(&ref_out, k);

        let mut b739 = raw.clone();
        let (c739, _) = safe_c52::unrle_739_shift(&mut b739, &runs, k);
        let mut b52 = raw.clone();
        let (c52, _) = safe_c52::unrle_zero_shift(&mut b52, &runs, k);
        assert_eq!(b739, ref_out, "739 mismatch tc={tc}");
        assert_eq!(c739, ref_crc, "739 crc mismatch tc={tc}");
        assert_eq!(b52, ref_out, "Thm 52.1 mismatch tc={tc}");
        assert_eq!(c52, ref_crc, "Thm 52.1 crc mismatch tc={tc}");
    }
    println!("THEOREM 52.1 STRESS GATE: 5,000/5,000 randomized adversarial hill/valley blocks EXACT (bytes + CRC)!");
}

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = if logs.len() >= 15 { 2.145 } else { 2.201 };
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();

    if args.get(1).map(String::as_str) == Some("--bzcat-child") {
        let data = fs::read(&args[2]).unwrap();
        let mode = args[3].as_str();
        let w: usize = args[4].parse().unwrap();
        let r0 = rusage(); let t0 = Instant::now();
        let dec = StreamDecoder::with_inverse::<OpenCells>(w, 4 * w, k);
        let mut total = 0usize;
        std::thread::scope(|s| {
            s.spawn(|| {
                let chunk_sz = match mode {
                    "bzcat_739_8M_z" => 8 << 20,
                    "bzcat_8M_noz" => 8 << 20,
                    "bzcat_1M_pool" => 1 << 20,
                    _ => 4 << 20,
                };
                for c in data.chunks(chunk_sz) {
                    let mut buf = dec.spare();
                    if mode == "bzcat_739_8M_z" {
                        if buf.capacity() < chunk_sz { buf = vec![0u8; chunk_sz]; }
                        buf.resize(chunk_sz, 0);
                        buf[..c.len()].copy_from_slice(c);
                        buf.truncate(c.len());
                    } else {
                        buf.clear();
                        if buf.capacity() < c.len() { buf.reserve_exact(c.len()); }
                        buf.extend_from_slice(c);
                    }
                    if dec.write(buf).is_err() { return; }
                }
                dec.close();
            });
            let mut spent = Vec::new();
            while let Some(b) = dec.read_block(spent).unwrap() {
                total += b.len();
                spent = b;
            }
        });
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {:.2} {total}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0);
        return;
    }

    if args.get(1).map(String::as_str) == Some("--micro") {
        verify_theorem_52_1(k);
        let data = fs::read("/tmp/wormhole_r6/wikidata_431b_210m.bz2").unwrap();
        let mut sc = Scanner::new(SliceSource(&data));
        let mut syms = Vec::new();
        let mut runs_s = mibz2::runs::Runs::default();
        let mut inv = OpenCells::default();
        let mut pre_blocks: Vec<(Vec<u8>, Vec<safe_c52::Run>, u32)> = Vec::new();
        loop {
            match sc.next(std::mem::take(&mut syms)).unwrap() {
                Event::Block { symbols, .. } => {
                    let n = mibz2::runs::front(&symbols, &mut runs_s, k).unwrap();
                    let mut out = Vec::with_capacity(n + n / 16 + 64);
                    inv.invert_runs(&runs_s, n, symbols.orig_ptr, &mut out, k);
                    let mut rr = Vec::new();
                    safe_c52::find_runs_scalar(&out, &mut rr).unwrap();
                    pre_blocks.push((out, rr, symbols.crc));
                    syms = symbols.syms;
                }
                Event::Done => break,
            }
        }
        let (mut copy_739, mut copy_52) = (0u64, 0u64);
        for (raw, rr, exp_crc) in &pre_blocks {
            let mut b1 = Vec::with_capacity(raw.len() + raw.len() / 16 + 64);
            b1.extend_from_slice(raw);
            let (c1, m1) = safe_c52::unrle_739_shift(&mut b1, rr, k);
            let mut b2 = Vec::with_capacity(raw.len() + raw.len() / 16 + 64);
            b2.extend_from_slice(raw);
            let (c2, m2) = safe_c52::unrle_zero_shift(&mut b2, rr, k);
            assert_eq!(c1, *exp_crc); assert_eq!(c2, *exp_crc); assert_eq!(b1, b2);
            copy_739 += m1; copy_52 += m2;
        }
        println!("\n431-BLOCK WIKIDATA EXACT PARITY OK: copy_within bytes: #739={:.2} MB vs Thm52.1_ZeroShift={:.2} MB ({:.1}% reduction!)",
            copy_739 as f64 / 1e6, copy_52 as f64 / 1e6, (1.0 - copy_52 as f64 / copy_739 as f64) * 100.0);

        let micro_arms = ["R739_A", "R739_B", "ZeroShift52"];
        let mut m_times: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut scratch_bufs: Vec<Vec<u8>> = pre_blocks.iter().map(|(r, _, _)| Vec::with_capacity(r.len() + r.len() / 16 + 64)).collect();
        for r in 0..15 {
            for i in 0..micro_arms.len() {
                let arm = micro_arms[(r + i) % micro_arms.len()];
                for (dst, (raw, _, _)) in scratch_bufs.iter_mut().zip(pre_blocks.iter()) {
                    dst.clear(); dst.extend_from_slice(raw);
                }
                let t0 = Instant::now();
                let mut acc = 0u32;
                for (buf, (_, rr, _)) in scratch_bufs.iter_mut().zip(pre_blocks.iter()) {
                    let (c, _) = match arm {
                        "R739_A" | "R739_B" => safe_c52::unrle_739_shift(buf, rr, k),
                        "ZeroShift52" => safe_c52::unrle_zero_shift(buf, rr, k),
                        _ => unreachable!(),
                    };
                    acc ^= c;
                }
                let ms = t0.elapsed().as_secs_f64() * 1e3;
                std::hint::black_box(acc);
                m_times.entry(arm).or_default().push(ms);
            }
        }
        println!("\n=== PART 1: 15-ROUND 1T MICRO A/B (IN-PLACE RLE1 EXPAND + TILED CRC, 431 BLKS, 210 MB) ===");
        let ma = &m_times["R739_A"];
        for &arm in &micro_arms {
            let mut v = m_times[arm].clone(); v.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(m_times[arm].iter().copied()).collect::<Vec<_>>());
            println!("{:<14} med={:>6.2} ms   vs R739_A = {:.4} [{:.4}, {:.4}]", arm, v[7], rat, lo, hi);
        }
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let m_out = Command::new(&exe).args(["--micro"]).output().unwrap();
    assert!(m_out.status.success(), "micro failed: {}", String::from_utf8_lossy(&m_out.stderr));
    print!("{}", String::from_utf8(m_out.stdout).unwrap());

    // 3. Part 2: 12-round fresh-process 4W A/B on examples/bzcat.rs:32 feeder chunking (8M resize(0) vs 8M no-zero vs 1M pool)
    let path = "/tmp/wormhole_r6/wikidata_431b_210m.bz2";
    let b_arms = ["bzcat_739_8M_z", "bzcat_8M_noz", "bzcat_1M_pool"];
    let mut bw: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut bs: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut bf: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut br: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for r in 0..12 {
        for i in 0..b_arms.len() {
            let arm = b_arms[(r + i) % b_arms.len()];
            let out = Command::new(&exe).args(["--bzcat-child", path, arm, "4"]).output().unwrap();
            let s = String::from_utf8(out.stdout).unwrap();
            let p: Vec<&str> = s.split_whitespace().collect();
            bw.entry(arm).or_default().push(p[0].parse().unwrap());
            bs.entry(arm).or_default().push(p[2].parse().unwrap());
            bf.entry(arm).or_default().push(p[3].parse().unwrap());
            br.entry(arm).or_default().push(p[4].parse().unwrap());
        }
    }
    println!("\n=== PART 2: 12-ROUND 4W FEEDER CHUNK & ZERO-FILL A/B ON 3296195e3 (210 MB) ===");
    let ba = &bw["bzcat_739_8M_z"];
    println!("{:<16} {:>7} {:>24} {:>7} {:>7} {:>7}", "Arm", "med_ms", "vs 739_8M [95% CI]", "sys_ms", "minflt", "rss_MB");
    for &arm in &b_arms {
        let mut wv = bw[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = bs[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = bf[arm].clone(); fv.sort();
        let mut rv = br[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let (rat, lo, hi) = ci95_ratio(&ba.iter().copied().zip(bw[arm].iter().copied()).collect::<Vec<_>>());
        println!("{:<16} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>7.2} {:>7} {:>7.2}", arm, wv[6], rat, lo, hi, sv[6], fv[6], rv[6]);
    }
}
