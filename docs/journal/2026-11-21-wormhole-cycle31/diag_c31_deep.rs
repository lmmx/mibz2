use std::fs;
use std::io::{Cursor, Read};
use std::process::Command;
use std::time::Instant;
use mibz2::{BzReader, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, SliceSource};

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

// Exact standalone mirror of src/rle.rs + src/crc.rs (scalar/clmul) for isolated stage timing
// Wait: we can also time via mibz2::rle? No, rle/crc are private in lib.rs, so measure exact stage deltas
// by copying the exact unrle & crc32 functions or comparing decode_block vs components!
// Even better: let's measure exact unrle_scalar/avx2 & crc32 on the decoded blocks AND measure cold-process scaling!

fn child_run(path: &str, w: usize, slots: usize, lazy: bool) {
    let data = fs::read(path).unwrap();
    let k = Kernels::detect();
    let mut read_buf = vec![0u8; 4 << 20];
    // Touch read_buf before timing so we separate decoder faults from caller buf?
    // Actually in real `ab::drain`, read_buf is allocated inside drain()! Let's record faults BEFORE and AFTER.
    let r0 = rusage();
    let t0 = Instant::now();
    let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, slots, k, lazy);
    let mut bytes = 0usize;
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut read_calls = 0usize;
    loop {
        let n = rdr.read(&mut read_buf).unwrap();
        if n == 0 { break; }
        read_calls += 1;
        bytes += n;
        // Fast 8-byte XOR-fold (similar to ab's FNV/wyhash-like reader drain touch)
        for c in read_buf[..n].chunks_exact(8) {
            let v = u64::from_le_bytes(c.try_into().unwrap());
            hash = hash.wrapping_mul(0x100000001b3) ^ v;
        }
    }
    drop(rdr);
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    println!("W={w} slots={slots} lazy={lazy} wall_ms={wall:.2} user_ms={:.2} sys_ms={:.2} minflt={} nvcsw={} nivcsw={} rss_MiB={:.2} reads={read_calls} bytes={bytes} hash={hash:016x}",
        tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime),
        tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
        r1.ru_minflt - r0.ru_minflt,
        r1.ru_nvcsw - r0.ru_nvcsw,
        r1.ru_nivcsw - r0.ru_nivcsw,
        r1.ru_maxrss as f64 / 1024.0);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let path = &args[2];
        let w: usize = args[3].parse().unwrap();
        let slots: usize = args[4].parse().unwrap();
        let lazy: bool = args[5].parse().unwrap();
        child_run(path, w, slots, lazy);
        return;
    }
    let path = args.get(1).cloned().unwrap_or_else(|| "/tmp/wormhole_r5/wikidata_431b_210m.bz2".into());
    let exe = std::env::current_exe().unwrap();
    println!("=== FRESH-PROCESS COLD-START SCALING ({path}) ===");
    for &(w, slots) in &[(1,4),(2,8),(4,16),(4,8),(7,28),(7,14),(8,32),(10,40),(10,20),(12,48),(16,64),(20,80)] {
        let mut vals = Vec::new();
        for _ in 0..3 {
            let out = Command::new(&exe)
                .args(["--child", &path, &w.to_string(), &slots.to_string(), "true"])
                .output().unwrap();
            let s = String::from_utf8(out.stdout).unwrap();
            vals.push(s.trim().to_string());
        }
        // Print median by wall_ms
        vals.sort_by(|a, b| {
            let fa: f64 = a.split("wall_ms=").nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
            let fb: f64 = b.split("wall_ms=").nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
            fa.total_cmp(&fb)
        });
        println!("{}", vals[1]);
    }

    // Also measure exact post-BWT stage times on the 6 authenticated blocks x 39 = 234 blocks
    let data = fs::read(&path).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms_buf = Vec::new();
    let mut scratch = Scratch::<OpenCells>::default();
    let k = Kernels::detect();
    let mut out = Vec::new();
    let mut inv_blocks: Vec<u8> = Vec::new();
    let mut dec_blocks = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(syms_buf).unwrap() {
        out.clear();
        mibz2::decode_block(&symbols, k, &mut scratch, &mut out).unwrap();
        if dec_blocks.len() < 18 && out.len() > 100000 {
            dec_blocks.push(out.clone());
        }
        syms_buf = symbols.syms;
    }
    // Time 13x passes over those 18 blocks (= 234 blocks = 210.6 MB) for memcpy + hash (reader cost)
    let mut reader_buf = vec![0u8; 4 << 20];
    let mut t_copy_v = Vec::new();
    let mut t_copy_hash_v = Vec::new();
    for _ in 0..5 {
        let t0 = Instant::now();
        let mut acc = 0usize;
        for _ in 0..13 {
            for b in &dec_blocks {
                reader_buf[..b.len()].copy_from_slice(b);
                acc = acc.wrapping_add(reader_buf[0] as usize);
            }
        }
        std::hint::black_box(acc);
        t_copy_v.push(t0.elapsed().as_secs_f64() * 1e3);

        let t1 = Instant::now();
        let mut h: u64 = 0xcbf29ce484222325;
        for _ in 0..13 {
            for b in &dec_blocks {
                reader_buf[..b.len()].copy_from_slice(b);
                for c in reader_buf[..b.len()].chunks_exact(8) {
                    h = h.wrapping_mul(0x100000001b3) ^ u64::from_le_bytes(c.try_into().unwrap());
                }
            }
        }
        std::hint::black_box(h);
        t_copy_hash_v.push(t1.elapsed().as_secs_f64() * 1e3);
    }
    t_copy_v.sort_by(|a,b| a.total_cmp(b));
    t_copy_hash_v.sort_by(|a,b| a.total_cmp(b));
    println!("\n=== READER SERIAL COST ON 210.6 MB (234 blocks) ===");
    println!("  memcpy(r.cur -> 4MiB buf):       {:.2} ms ({:.1} GB/s)", t_copy_v[2], 210.6 / t_copy_v[2]);
    println!("  memcpy + 8B-hash (ab::drain):    {:.2} ms ({:.1} GB/s)", t_copy_hash_v[2], 210.6 / t_copy_hash_v[2]);
    let _ = inv_blocks;
}
