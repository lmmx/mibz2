use std::fs;
use std::process::Command;
use std::time::Instant;
use mibz2::{Kernels, Scratch, StreamDecoder};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::stream::{Event, Scanner, SliceSource};

#[repr(C)] #[derive(Default, Clone, Copy)] struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)] #[derive(Default, Clone, Copy)] struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval, ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64, ru_inblock: i64, ru_oublock: i64,
    ru_msgsnd: i64, ru_msgrcv: i64, ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" { fn getrusage(who: i32, usage: *mut Rusage) -> i32; }
fn rusage() -> Rusage { let mut r = Rusage::default(); unsafe { getrusage(0, &mut r); } r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

struct AbHash { bytes: u64, h: u64 }
impl AbHash {
    fn new() -> Self { Self { bytes: 0, h: 0xcbf29ce484222325 } }
    #[inline]
    fn feed(&mut self, s: &[u8]) {
        self.bytes += s.len() as u64;
        let mut h = self.h;
        let mut ch = s.chunks_exact(8);
        for c in &mut ch { h = (h ^ u64::from_le_bytes(c.try_into().unwrap())).wrapping_mul(0x100000001b3); }
        for &b in ch.remainder() { h = (h ^ b as u64).wrapping_mul(0x100000001b3); }
        self.h = h;
    }
}

fn run_stage_census(data: &[u8], k: Kernels) {
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let mut blocks = Vec::new();
    let t0 = Instant::now();
    loop {
        match sc.next(std::mem::take(&mut syms)).unwrap() {
            Event::Block { symbols, .. } => blocks.push(symbols),
            Event::Done => break,
        }
    }
    let t_scan = t0.elapsed().as_secs_f64() * 1e3;

    let mut runs = mibz2::runs::Runs::default();
    let mut inv = OpenCells::default();
    let mut scratch = Scratch::<OpenCells>::default();
    let mut out = Vec::with_capacity(960_000);
    let mut t_front = 0.0;
    let mut t_inv = 0.0;
    let mut t_decode_owned = 0.0;
    let mut n_full = 0usize;
    let mut n_tiny = 0usize;
    let mut n_rle_runs_total = 0usize;
    let mut n_shift_pos = 0usize;
    let mut n_min_d_ge0 = 0usize;
    let mut n_shift_gt_n16 = 0usize;
    let mut n_shift_gt_n = 0usize;
    let mut shift_copy_bytes = 0u64;
    let mut post_rle_bytes = 0u64;
    let mut syms_bytes = 0u64;

    // Pass A: isolate front + invert_runs + analyze exact RLE1 shift distribution on pre-RLE bytes
    for b in &blocks {
        syms_bytes += (b.syms.len() * 2) as u64;
        if b.syms.len() > 2048 { n_full += 1; } else { n_tiny += 1; }
        let tf0 = Instant::now();
        let n = mibz2::runs::front(b, &mut runs, k).unwrap();
        t_front += tf0.elapsed().as_secs_f64() * 1e3;

        let ti0 = Instant::now();
        inv.invert_runs(&runs, n, b.orig_ptr, &mut out, k);
        t_inv += ti0.elapsed().as_secs_f64() * 1e3;

        // Analyze exact RLE1 runs and shift on `out[..n]`
        let mut i = 0usize;
        let mut first = None;
        let mut grown = 0isize;
        let mut shift = 0isize;
        let mut min_d = 0isize;
        while i + 3 < n {
            let c = out[i];
            if out[i+1] == c && out[i+2] == c && out[i+3] == c {
                if first.is_none() { first = Some(i); }
                let extra = out[i+4];
                n_rle_runs_total += 1;
                grown += extra as isize - 1;
                if grown > shift { shift = grown; }
                if grown < min_d { min_d = grown; }
                i += 5;
            } else { i += 1; }
        }
        post_rle_bytes += (n as isize + grown) as u64;
        if let Some(f) = first {
            if shift > 0 {
                n_shift_pos += 1;
                shift_copy_bytes += (n - f) as u64;
                if shift as usize > n / 16 + 64 { n_shift_gt_n16 += 1; }
                if shift as usize > n { n_shift_gt_n += 1; }
            }
            if min_d >= 0 { n_min_d_ge0 += 1; }
        }
    }

    // Pass B: full #739 decode_block (front + invert_runs + unrle + CRC)
    let mut pool_out = Vec::with_capacity(960_000);
    let td0 = Instant::now();
    for b in &blocks {
        pool_out.clear();
        mibz2::decode_block(b, k, &mut scratch, &mut pool_out).unwrap();
    }
    let t_decode_block = td0.elapsed().as_secs_f64() * 1e3;
    let t_rle_crc_est = (t_decode_block - t_front - t_inv).max(0.0);

    println!("=== 1T STAGE & RLE1-SHIFT CENSUS ON 3296195e3 (431 BLKS) ===");
    println!("blocks={} (full={} tiny={}) out_bytes={} syms_MB={:.2}", n_full+n_tiny, n_full, n_tiny, post_rle_bytes, syms_bytes as f64 / 1e6);
    println!("t_scan={t_scan:.2} ms  t_front={t_front:.2} ms  t_invert_runs={t_inv:.2} ms  t_rle_crc_est={t_rle_crc_est:.2} ms  t_decode_block={t_decode_block:.2} ms");
    println!("RLE1 census: total_runs={n_rle_runs_total} blks_shift_pos={n_shift_pos} blks_min_d_ge0={n_min_d_ge0} blks_shift_gt_n16={n_shift_gt_n16} blks_shift_gt_n={n_shift_gt_n} shift_copy_MB={:.2}",
        shift_copy_bytes as f64 / 1e6);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--census") {
        let data = fs::read(&args[2]).unwrap();
        run_stage_census(&data, k);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let mode = args[3].as_str();
        let w: usize = args[4].parse().unwrap();
        let slots: usize = args[5].parse().unwrap();
        let r0 = rusage(); let t0 = Instant::now();
        let dec = StreamDecoder::with_inverse::<OpenCells>(w, slots, k);
        let mut h = AbHash::new();
        std::thread::scope(|s| {
            s.spawn(|| {
                for c in data.chunks(4 << 20) {
                    let mut buf = dec.spare();
                    buf.clear(); buf.extend_from_slice(c);
                    if dec.write(buf).is_err() { return; }
                }
                dec.close();
            });
            if mode == "read_block" {
                let mut spent = Vec::new();
                while let Some(b) = dec.read_block(spent).unwrap() {
                    h.feed(&b);
                    spent = b;
                }
            } else {
                let mut buf = vec![0u8; 4 << 20];
                loop { match dec.read(&mut buf).unwrap() { 0 => break, n => h.feed(&buf[..n]) } }
            }
        });
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {} {} {:.2} {} {:016x}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_nivcsw - r0.ru_nivcsw,
            r1.ru_maxrss as f64 / 1024.0, h.bytes, h.h);
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let path = "/tmp/wormhole_r6/wikidata_431b_210m.bz2";
    let c = Command::new(&exe).args(["--census", path]).output().unwrap();
    print!("{}", String::from_utf8(c.stdout).unwrap());

    for mode in ["read_block", "read_4m"] {
        println!("\n=== FRESH-PROCESS COLD-START SCALING ON 3296195e3 ({mode}, OpenCells, 12 MiB cap, 3-rd med) ===");
        println!("{:<4} {:>5} {:>8} {:>8} {:>7} {:>7} {:>6} {:>6} {:>7}", "W", "slots", "wall_ms", "user_ms", "sys_ms", "minflt", "vcsw", "ivcsw", "rss_MB");
        for &(w, slots) in &[(1,4),(2,8),(4,16),(6,24),(7,28),(8,32),(10,40),(10,20),(12,48),(16,64),(20,80)] {
            let mut rows = Vec::new();
            for _ in 0..3 {
                let o = Command::new(&exe).args(["--child", path, mode, &w.to_string(), &slots.to_string()]).output().unwrap();
                assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
                let s = String::from_utf8(o.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                rows.push((
                    p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                    p[3].parse::<i64>().unwrap(), p[4].parse::<i64>().unwrap(), p[5].parse::<i64>().unwrap(),
                    p[6].parse::<f64>().unwrap(),
                ));
            }
            rows.sort_by(|a,b| a.0.total_cmp(&b.0));
            let m = rows[1];
            println!("{:<4} {:>5} {:>8.2} {:>8.2} {:>7.2} {:>7} {:>6} {:>6} {:>7.2}", w, slots, m.0, m.1, m.2, m.3, m.4, m.5, m.6);
        }
    }
}
