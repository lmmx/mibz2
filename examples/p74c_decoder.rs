//! The P74-C full-decoder trial (docs/journal/2026-10-31-p74c-decoder-protocol.md): one
//! executable, both MTF settings, every measurement its own child process. Preparation
//! and checking by default; timing only with `--trial`, under the runner's approval.
//!
//! - `--prepare DIR TEXT_ROOT`: the controls (random, records, text, mixed-independent), as
//!   the stage trial's `examples/p74_experiment.rs --prepare` wrote them; `text` from the
//!   sources under TEXT_ROOT (amendment 1: the v1.0 pin's tree, so that harness edits
//!   leave the corpus unchanged).
//! - `--windows OUT WINDOWS...`: the stage trial's window tool, unchanged: each window's
//!   whole blocks, their bits untouched, as a stream of its own; the streams one after
//!   another.
//! - `--schedule MANIFEST WORKERS`: the registered order (no timing).
//! - `--check MANIFEST WORKERS`: every corpus decoded by arms A and B at every w through
//!   the trial's decode, and sequentially, each output's SHA-256 the expected one; which
//!   kernel each block ran (no timing).
//! - `--audit MANIFEST`: the selector audit of every block, by an independent spelling of
//!   the frozen policy: bypassed, P74 to the block's end, or the handoff's position.
//! - `--trial MANIFEST WORKERS ALLOC`: the timed trial (needs `P74C_APPROVED_RUNNER`).
//! - `--child ARM W PATH SHA256 BYTES`: one measurement (the trial's children).
//!
//! `P74C_SYNTHETIC_TIMES=1` (the runner's harness self-test, before any timing): `--trial`
//! and `--child` run every step, but every time and CPU figure they write is a fixed
//! synthetic value, never a measurement; no runner approval is needed for it.
//!
//! **The arms differ only in the MTF choice** (`decode_once`'s `k`): A and A′ pass
//! `Kernels::detect()`, B passes `Kernels::detect().with_mtf(MtfKernel::P74c)`; the same
//! binary, entry point (`StreamDecoder::planned(Plan::Workers(w), 2w, k)`), feeder, reader,
//! scanner and workers. The decoder reads `k.mtf` once a block (`decode_block`).
//!
//! A MANIFEST is TSV: name, role, path, compressed SHA-256, decoded SHA-256, decoded bytes.
//! WORKERS and ALLOC are comma-separated (`1,2,3,4`; CPUs `1,2,3,4`).
#[path = "../tests/common/mod.rs"]
mod common;
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item, BLOCK_MAGIC};
use mibz2::mtf::selections;
use mibz2::parallel::Plan;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{Kernels, MtfKernel, StreamDecoder};
use std::io::Write as _;
use std::path::Path;
use std::time::Instant;

fn windows(out: &str, paths: &[String]) {
    let mut corpus = Vec::new();
    for path in paths {
        let data = std::fs::read(path).unwrap();
        let mut bit = (0..data.len() as u64 * 8 - 48)
            .find(|&at| {
                let mut r = Bits::at(&data, at);
                (r.bits(24) as u64) << 24 | r.bits(24) as u64 == BLOCK_MAGIC
            })
            .unwrap();
        let mut blocks = Vec::new();
        let mut crcs = Vec::new();
        let mut want = Vec::new();
        let mut scratch: mibz2::Scratch = mibz2::Scratch::default();
        let mut syms = Vec::new();
        let mut level = 9;
        loop {
            let mut r = Bits::at(&data, bit);
            let item = read_item(&mut r, level, syms);
            if r.overran() {
                break;
            }
            let end = r.bit_pos();
            match item.unwrap_or_else(|e| panic!("{path}: {e:?}")) {
                Item::Block(b) => {
                    crcs.push(
                        mibz2::decode_block(&b, Kernels::detect(), &mut scratch, &mut want)
                            .unwrap(),
                    );
                    blocks.push((bit, end));
                    syms = b.syms;
                    bit = end;
                }
                Item::End(_) => {
                    let at = end.div_ceil(8) as usize;
                    if let Some([b'B', b'Z', b'h', l @ b'1'..=b'9']) = data.get(at..at + 4) {
                        level = (l - b'0') as u32;
                        bit = (at as u64 + 4) * 8;
                        syms = Vec::new();
                    } else {
                        break;
                    }
                }
            }
        }
        let mut w = common::BitWriter::default();
        for c in b"BZh9" {
            w.put(*c as u64, 8);
        }
        for (start, end) in blocks {
            let mut r = Bits::at(&data, start);
            let mut left = end - start;
            while left > 0 {
                let n = left.min(32) as u32;
                w.put(r.bits(n) as u64, n);
                left -= n as u64;
            }
        }
        w.put(0x1772_4538_5090, 48);
        w.put(mibz2::combine(crcs) as u64, 32);
        let z = w.finish();
        assert_eq!(common::oracle(&z).unwrap(), want);
        assert_eq!(mibz2::decode(&z).unwrap(), want);
        corpus.extend(z);
    }
    std::fs::write(out, corpus).unwrap();
}

/// The controls, exactly as the stage trial's `--prepare` wrote them
/// `common::corpora`'s text corpus, built from the sources under `root` instead of this
/// checkout's: the top-level files of `src`, `tests` and `examples` in sorted order, then
/// `README.md`, repeated with a separator up to 3,000,000 × `scale` bytes
fn text_at(root: &Path, scale: usize) -> Vec<u8> {
    let mut text = Vec::new();
    for dir in ["src", "tests", "examples"] {
        let mut files: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        files.sort();
        for f in files.into_iter().filter(|f| f.is_file()) {
            text.extend(std::fs::read(f).unwrap());
        }
    }
    text.extend(std::fs::read(root.join("README.md")).unwrap());
    let one = text.clone();
    let mut i = 0;
    while text.len() < 3_000_000 * scale {
        text.extend(format!("\n-- copy {i} --\n").bytes());
        text.extend(&one);
        i += 1;
    }
    text
}

fn prepare(dir: &Path, text_root: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    for (name, data) in common::corpora(4) {
        if ["random", "records"].contains(&name) {
            std::fs::write(dir.join(format!("{name}.bz2")), common::compress(&data, 9)).unwrap();
        }
    }
    std::fs::write(
        dir.join("text.bz2"),
        common::compress(&text_at(text_root, 4), 9),
    )
    .unwrap();
    let mut rng = common::Rng(0x74c0_2026_1031_0001);
    let data: Vec<u8> = (0..2_000_000)
        .map(|i| {
            if (i / 10000) % 2 == 0 {
                b"mixed-control"[rng.below(13) as usize]
            } else {
                rng.next() as u8
            }
        })
        .collect();
    std::fs::write(
        dir.join("mixed-independent.bz2"),
        common::compress(&data, 9),
    )
    .unwrap();
}

// ---------------------------------------------------------------- SHA-256 (FIPS 180-4)

fn sha256(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut tail = data[data.len() / 64 * 64..].to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    tail.extend((data.len() as u64 * 8).to_be_bytes());
    for block in data.chunks_exact(64).chain(tail.chunks_exact(64)) {
        let mut w = [0u32; 64];
        for (i, c) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            (hh, g, f, e, d, c, b, a) = (g, f, e, d.wrapping_add(t1), c, b, a, t1.wrapping_add(t2));
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

// ---------------------------------------------------------------- the process's resources

/// `struct rusage` (Linux, 64-bit)
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rusage {
    utime: [i64; 2],
    stime: [i64; 2],
    maxrss: i64,
    ixrss: i64,
    idrss: i64,
    isrss: i64,
    minflt: i64,
    majflt: i64,
    nswap: i64,
    inblock: i64,
    oublock: i64,
    msgsnd: i64,
    msgrcv: i64,
    nsignals: i64,
    nvcsw: i64,
    nivcsw: i64,
}

impl Rusage {
    fn user_us(&self) -> i64 {
        self.utime[0] * 1_000_000 + self.utime[1]
    }
    fn sys_us(&self) -> i64 {
        self.stime[0] * 1_000_000 + self.stime[1]
    }
}

#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
    fn wait4(pid: i32, status: *mut i32, options: i32, usage: *mut Rusage) -> i32;
}

/// This process's resources so far, all its threads (RUSAGE_SELF)
fn rusage_self() -> Rusage {
    let mut r = Rusage::default();
    #[cfg(all(target_os = "linux", target_pointer_width = "64"))]
    // SAFETY: a valid out-pointer to a `struct rusage`
    assert_eq!(unsafe { getrusage(0, &mut r) }, 0);
    r
}

/// Wait for child `pid`: its exit status and its whole life's resources (wait4)
fn wait_child(pid: u32) -> (i32, Rusage) {
    let mut r = Rusage::default();
    let mut status = 0;
    #[cfg(all(target_os = "linux", target_pointer_width = "64"))]
    // SAFETY: valid out-pointers; `pid` is our unreaped child
    assert_eq!(
        unsafe { wait4(pid as i32, &mut status, 0, &mut r) },
        pid as i32
    );
    (status, r)
}

/// A `Cpus_allowed_list` ("1-4,6") as CPUs
fn cpu_list(s: &str) -> Vec<usize> {
    let mut v = Vec::new();
    for part in s.trim().split(',').filter(|p| !p.is_empty()) {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        v.extend(a.parse::<usize>().unwrap()..=b.parse::<usize>().unwrap());
    }
    v
}

/// The physical cores of `cpus` (sysfs package and core ids)
fn physical_cores(cpus: &[usize]) -> usize {
    let mut cores = std::collections::BTreeSet::new();
    for c in cpus {
        let t = format!("/sys/devices/system/cpu/cpu{c}/topology");
        let core = std::fs::read_to_string(format!("{t}/core_id")).unwrap_or_default();
        let pkg = std::fs::read_to_string(format!("{t}/physical_package_id")).unwrap_or_default();
        cores.insert((pkg.trim().to_string(), core.trim().to_string()));
    }
    cores.len()
}

fn status_field(status: &str, key: &str) -> String {
    status
        .lines()
        .find_map(|l| l.strip_prefix(key))
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

/// A thread's record from its /proc directory: `name|allowed|psr|vcs|nvcs`
fn thread_record(dir: &Path) -> String {
    let status = std::fs::read_to_string(dir.join("status")).unwrap_or_default();
    let stat = std::fs::read_to_string(dir.join("stat")).unwrap_or_default();
    // fields after the command: state is field 3, processor (psr) field 39
    let psr = stat
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().nth(36).map(str::to_string))
        .unwrap_or_default();
    format!(
        "{}|{}|{}|{}|{}",
        status_field(&status, "Name:"),
        status_field(&status, "Cpus_allowed_list:"),
        psr,
        status_field(&status, "voluntary_ctxt_switches:"),
        status_field(&status, "nonvoluntary_ctxt_switches:"),
    )
}

/// The calling thread's record
fn thread_self() -> String {
    thread_record(Path::new("/proc/thread-self"))
}

/// Every thread of this process (/proc/self/task/*): how many, and their records joined
/// by `;`
fn threads() -> (usize, String) {
    let mut v = Vec::new();
    if let Ok(dir) = std::fs::read_dir("/proc/self/task") {
        for e in dir.flatten() {
            v.push(thread_record(&e.path()));
        }
    }
    (v.len(), v.join(";"))
}

// ---------------------------------------------------------------- one decode

const ARMS: [&str; 3] = ["A", "B", "A2"];

/// The arm's kernels: the ONLY difference between the arms
fn arm_kernels(arm: usize) -> Kernels {
    let base = Kernels::detect();
    match arm {
        1 => base.with_mtf(MtfKernel::P74c),
        0 | 2 => base,
        _ => panic!("arm {arm}"),
    }
}

struct Decoded {
    out: Vec<u8>,
    wall_ns: u128,
    cpu: (i64, i64),
    selections: [u64; 10],
    threads_spawned: (usize, String),
}

/// `z` through the production parallel decoder at `Plan::Workers(w)`, fed and read as
/// `decode_parallel` does (4 MiB chunks from a feeder thread, 4 MiB reads): the wall and
/// process CPU from just before the decoder is built to the read that returns 0
fn decode_once(z: &[u8], w: usize, k: Kernels, expected_len: usize) -> Decoded {
    let mut out = Vec::with_capacity(expected_len);
    let mut buf = vec![0u8; 4 << 20];
    let feeder = std::sync::Mutex::new(String::new());
    let sel0 = selections::snapshot();
    let ru0 = rusage_self();
    let t0 = Instant::now();
    let dec = StreamDecoder::planned(Plan::Workers(w), 2 * w, k);
    // every decoder thread spawned and blocked (no input yet: none can have ended): the
    // workers, the scanner and main, their allowed CPUs; the same cost in every arm
    // (a thread's name can still be the parent's until it starts)
    let mut spawned = threads();
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .name("p74c-feeder".into())
            .spawn_scoped(s, || {
                // (the feeder's own allowed CPUs, as it starts)
                *feeder.lock().unwrap() = thread_self();
                for c in z.chunks(4 << 20) {
                    if dec.write(c.to_vec()).is_err() {
                        return;
                    }
                }
                dec.close();
            })
            .expect("spawn the feeder");
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => panic!("decode failed: {e}"),
            }
        }
    });
    let wall_ns = t0.elapsed().as_nanos();
    let ru1 = rusage_self();
    let sel1 = selections::snapshot();
    assert_eq!(dec.threads(), w, "the decoder's threads");
    dec.join();
    let feeder = feeder.into_inner().unwrap();
    assert!(!feeder.is_empty(), "the feeder's record");
    spawned.0 += 1;
    spawned.1 = format!("{};{feeder}", spawned.1);
    Decoded {
        out,
        wall_ns,
        cpu: (ru1.user_us() - ru0.user_us(), ru1.sys_us() - ru0.sys_us()),
        selections: std::array::from_fn(|i| sel1[i] - sel0[i]),
        threads_spawned: spawned,
    }
}

/// The checks every child and `--check` make: the plan the default policy would take
/// here is `Workers(w)`; the selections are the arm's kernel's only
fn plan_and_cpus(w: usize) -> (Plan, String, usize) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let allowed = status_field(&status, "Cpus_allowed_list:");
    let phys = physical_cores(&cpu_list(&allowed));
    let plan = Plan::for_threads(w);
    assert_eq!(
        plan,
        Plan::Workers(w),
        "the default plan here is not Workers({w})"
    );
    assert!(w <= phys, "{w} workers on {phys} physical cores");
    (plan, allowed, phys)
}

fn check_selections(arm: usize, s: &[u64; 10]) -> u64 {
    let (n, others): (u64, u64) = if arm == 1 {
        assert_eq!(s[2], s[4] + s[5] + s[6], "P74-C's entries: {s:?}");
        assert_eq!(s[6], 0, "P74-C without AVX2: {s:?}");
        (s[2], s[0] + s[1] + s[3] + s[7] + s[8] + s[9])
    } else {
        (s[0], s[1..].iter().sum())
    };
    assert!(n > 0 && others == 0, "arm {}: selections {s:?}", ARMS[arm]);
    n
}

struct Corpus {
    name: String,
    role: String,
    path: String,
    compressed: String,
    decoded: String,
    bytes: usize,
}

fn manifest(path: &str) -> Vec<Corpus> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 6, "manifest line {l:?}");
            Corpus {
                name: f[0].into(),
                role: f[1].into(),
                path: f[2].into(),
                compressed: f[3].into(),
                decoded: f[4].into(),
                bytes: f[5].parse().unwrap(),
            }
        })
        .collect()
}

fn list(s: &str) -> Vec<usize> {
    s.split(',').map(|x| x.parse().unwrap()).collect()
}

// ---------------------------------------------------------------- the schedule

/// The registered schedule (protocol §6): for `cells` cells, each cell's 20 arm orders
/// (the 6 permutations of A, B, A′ three times, then (A, B, A′) and (B, A, A′), shuffled
/// per cell), then each round's cell order (shuffled afresh each round), from xorshift
/// seed 0x74c0202610310201: (orders[cell][round], rounds[round] = cell order)
fn shuffle<T>(v: &mut [T], rng: &mut common::Rng) {
    for i in (1..v.len()).rev() {
        v.swap(i, rng.below(i as u64 + 1) as usize);
    }
}

fn schedule(cells: usize) -> (Vec<Vec<[usize; 3]>>, Vec<Vec<usize>>) {
    const PERMS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut rng = common::Rng(0x74c0_2026_1031_0201);
    let mut orders = Vec::new();
    for _ in 0..cells {
        let mut seq: Vec<[usize; 3]> = PERMS.iter().chain(&PERMS).chain(&PERMS).copied().collect();
        seq.extend([[0, 1, 2], [1, 0, 2]]);
        shuffle(&mut seq, &mut rng);
        orders.push(seq);
    }
    let mut rounds = Vec::new();
    for _ in 0..ROUNDS {
        let mut cell: Vec<usize> = (0..cells).collect();
        shuffle(&mut cell, &mut rng);
        rounds.push(cell);
    }
    (orders, rounds)
}

const ROUNDS: usize = 20;

// ---------------------------------------------------------------- the modes

/// One measurement, in this (child) process: one line on stdout
fn child(args: &[String]) {
    let (arm, w, path, sha, bytes) = (
        args[0].parse::<usize>().unwrap(),
        args[1].parse::<usize>().unwrap(),
        &args[2],
        &args[3],
        args[4].parse::<usize>().unwrap(),
    );
    assert!(
        Kernels::detect().avx2,
        "AVX2 required: without it B is the shipped kernel"
    );
    let (plan, allowed, phys) = plan_and_cpus(w);
    let z = std::fs::read(path).unwrap();
    let mut d = decode_once(&z, w, arm_kernels(arm), bytes);
    if synthetic() {
        (d.wall_ns, d.cpu) = (1_000_000_000, (1_000_000, 0));
    }
    // untimed from here
    let end = threads();
    assert_eq!(
        d.threads_spawned.0,
        w + 3,
        "w workers, the scanner, the feeder and main"
    );
    assert_eq!(d.out.len(), bytes, "{path}: decoded length");
    assert_eq!(&sha256(&d.out), sha, "{path}: decoded SHA-256");
    let blocks = check_selections(arm, &d.selections);
    let sel: Vec<String> = d.selections.iter().map(u64::to_string).collect();
    println!(
        "CHILD\t{}\t{}\t{}\t{}\t1\t{blocks}\t{}\t{plan:?}\t{allowed}\t{phys}\t{}\t{}\t{}\t{}",
        d.wall_ns,
        d.cpu.0,
        d.cpu.1,
        d.out.len(),
        sel.join(","),
        d.threads_spawned.0,
        d.threads_spawned.1,
        end.0,
        end.1,
    );
}

/// Every thread's allowed CPUs within `alloc` (`name|allowed|...;...`)
fn assert_confined(snapshot: &str, alloc: &[usize], what: &str) {
    for t in snapshot.split(';').filter(|t| !t.is_empty()) {
        let allowed = t.split('|').nth(1).unwrap_or("");
        let cpus = cpu_list(allowed);
        assert!(
            !cpus.is_empty() && cpus.iter().all(|c| alloc.contains(c)),
            "{what}: thread {t} outside the allocation {alloc:?}"
        );
    }
}

/// The harness self-test: every figure written is synthetic, none measured
fn synthetic() -> bool {
    std::env::var("P74C_SYNTHETIC_TIMES").is_ok_and(|v| v == "1")
}

fn trial(args: &[String]) {
    assert!(
        synthetic()
            || !std::env::var("P74C_APPROVED_RUNNER")
                .unwrap_or_default()
                .is_empty(),
        "the timed trial runs only from bench/p74c_decoder_run.sh"
    );
    if let Ok(quota) = std::fs::read_to_string("/sys/fs/cgroup/cpu.max") {
        assert!(
            quota.starts_with("max "),
            "a CPU-quota-limited cgroup: {quota}"
        );
    }
    assert!(Kernels::detect().avx2, "AVX2 required");
    let corpora = manifest(&args[0]);
    let workers = list(&args[1]);
    let alloc = list(&args[2]);
    let mine = cpu_list(&status_field(
        &std::fs::read_to_string("/proc/self/status").unwrap(),
        "Cpus_allowed_list:",
    ));
    assert_eq!(
        mine, alloc,
        "the trial's own affinity is not the allocation"
    );
    let exe = std::env::current_exe().unwrap();
    let cells: Vec<(usize, usize)> = (0..corpora.len())
        .flat_map(|c| workers.iter().map(move |&w| (c, w)))
        .collect();
    let (orders, rounds) = schedule(cells.len());
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let run = |round: i64,
               cell_pos: usize,
               cell: usize,
               position: usize,
               arm: usize,
               out: &mut std::io::StdoutLock| {
        let (c, w) = cells[cell];
        let corpus = &corpora[c];
        let t0 = Instant::now();
        // (reaped by `wait_child`'s wait4, for the child's rusage)
        #[allow(clippy::zombie_processes)]
        let mut ch = std::process::Command::new(&exe)
            .args([
                "--child",
                &arm.to_string(),
                &w.to_string(),
                &corpus.path,
                &corpus.decoded,
            ])
            .arg(corpus.bytes.to_string())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn a child");
        let mut text = String::new();
        std::io::Read::read_to_string(ch.stdout.as_mut().unwrap(), &mut text).unwrap();
        let (status, mut ru) = wait_child(ch.id());
        let mut life_ns = t0.elapsed().as_nanos();
        if synthetic() {
            (life_ns, ru.utime, ru.stime) = (0, [0, 0], [0, 0]);
        }
        assert_eq!(status, 0, "{}: child failed ({status})", corpus.name);
        let line = text
            .lines()
            .find(|l| l.starts_with("CHILD\t"))
            .expect("the child's row");
        let f: Vec<&str> = line.split('\t').collect();
        assert_confined(f[12], &alloc, &corpus.name);
        assert_confined(f[14], &alloc, &corpus.name);
        writeln!(
            out,
            "ROW\t{round}\t{cell_pos}\t{c}\t{}\t{w}\t{arm}\t{position}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{life_ns}",
            corpus.name,
            f[1..].join("\t"),
            ru.user_us(),
            ru.sys_us(),
            ru.maxrss,
            ru.minflt,
            ru.majflt,
            ru.nvcsw,
            ru.nivcsw,
            u8::from(round < 0),
        )
        .unwrap();
        out.flush().unwrap();
    };
    // warm-up: every cell, each arm once (round -1; discarded)
    for cell in 0..cells.len() {
        for arm in 0..3 {
            run(-1, cell, cell, arm, arm, &mut out);
        }
    }
    for (round, order) in rounds.iter().enumerate() {
        for (cell_pos, &cell) in order.iter().enumerate() {
            for (position, &arm) in orders[cell][round].iter().enumerate() {
                run(round as i64, cell_pos, cell, position, arm, &mut out);
            }
        }
    }
}

/// `--check` decodes many times in one process. The previous decode's feeder ran in
/// `std::thread::scope`, which returns when the closure has finished, not when the OS
/// thread has exited, so it can still be listed in /proc/self/task (amendment 1), and the
/// next decode's snapshot would count it. Wait, bounded, until only main is listed.
/// Untimed, `--check` only: a timed child makes one decode in a fresh process.
fn settle_threads() {
    let t0 = Instant::now();
    loop {
        let n = std::fs::read_dir("/proc/self/task").map_or(0, |d| d.count());
        if n == 1 {
            return;
        }
        assert!(
            t0.elapsed() < std::time::Duration::from_secs(5),
            "{n} threads listed 5 s after the last decode was joined"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn check(args: &[String]) {
    assert!(
        Kernels::detect().avx2,
        "AVX2 required: without it B is the shipped kernel"
    );
    let workers = list(&args[1]);
    for c in manifest(&args[0]) {
        let z = std::fs::read(&c.path).unwrap();
        assert_eq!(sha256(&z), c.compressed, "{}: compressed SHA-256", c.name);
        let mut blocks = 0u64;
        let mut sc = Scanner::new(SliceSource(&z));
        while let Event::Block { .. } = sc.next(Vec::new()).unwrap() {
            blocks += 1;
        }
        let mut row = format!("CHECK\t{}\t{}\t{blocks}", c.name, c.role);
        for arm in [0, 1] {
            let k = arm_kernels(arm);
            for &w in &workers {
                plan_and_cpus(w);
                settle_threads();
                let d = decode_once(&z, w, k, c.bytes);
                assert_eq!(d.out.len(), c.bytes, "{} arm {} w {w}", c.name, ARMS[arm]);
                assert_eq!(
                    sha256(&d.out),
                    c.decoded,
                    "{} arm {} w {w}",
                    c.name,
                    ARMS[arm]
                );
                assert_eq!(check_selections(arm, &d.selections), blocks);
                if arm == 1 {
                    row += &format!(
                        "\tw{w}:bypass={},combined={}",
                        d.selections[4], d.selections[5]
                    );
                }
                let threads = d.threads_spawned.0;
                assert_eq!(
                    threads,
                    w + 3,
                    "w workers, the scanner, the feeder and main"
                );
                row += &format!("\t{}w{w}:threads={threads}", ARMS[arm]);
            }
            let s = mibz2::decode_with::<mibz2::bwt::Segmented>(&z, k).unwrap();
            assert_eq!(
                sha256(&s),
                c.decoded,
                "{} arm {} sequential",
                c.name,
                ARMS[arm]
            );
        }
        println!("{row}\tok");
    }
}

/// The frozen policy, spelled independently of the kernel: per block, bypassed or not,
/// and where the guard switches (the consumed symbols, the trigger included)
fn audit(args: &[String]) {
    for c in manifest(&args[0]) {
        let z = std::fs::read(&c.path).unwrap();
        let mut sc = Scanner::new(SliceSource(&z));
        let (mut blocks, mut bypassed, mut to_end, mut handoffs) = (0, 0, 0, 0);
        while let Event::Block { symbols: b, .. } = sc.next(Vec::new()).unwrap() {
            let s = b.syms.len();
            let bypass = s < 16_384 || 2 * s >= b.max_len;
            let (mut explicit, mut high, mut f) = (0usize, 0usize, 0usize);
            let mut switch = None;
            for (i, &x) in b.syms.iter().enumerate() {
                if x == b.eob {
                    break;
                }
                explicit += (x > 1) as usize;
                if x >= 33 {
                    high += 1;
                    f += 1;
                    if !bypass && switch.is_none() && f > 123 && 32 * (f - 123) > i + 1 {
                        switch = Some(i + 1);
                    }
                }
            }
            println!(
                "AUDIT\t{}\t{blocks}\t{s}\t{}\t{}\t{explicit}\t{high}\t{}",
                c.name,
                b.max_len,
                u8::from(bypass),
                switch.unwrap_or(0)
            );
            blocks += 1;
            if bypass {
                bypassed += 1;
            } else if switch.is_some() {
                handoffs += 1;
            } else {
                to_end += 1;
            }
        }
        println!(
            "AUDITSUM\t{}\t{blocks}\t{bypassed}\t{to_end}\t{handoffs}",
            c.name
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !cfg!(all(target_os = "linux", target_pointer_width = "64")) {
        eprintln!("p74c_decoder: Linux (64-bit) only");
        std::process::exit(2);
    }
    match args.first().map(String::as_str) {
        Some("--prepare") => prepare(Path::new(&args[1]), Path::new(&args[2])),
        Some("--windows") => windows(&args[1], &args[2..]),
        Some("--schedule") => {
            let cells = manifest(&args[1]).len() * list(&args[2]).len();
            let (orders, rounds) = schedule(cells);
            for (r, cell) in rounds.iter().enumerate() {
                let o: Vec<String> = cell
                    .iter()
                    .map(|&c| {
                        format!(
                            "{c}:{}",
                            orders[c][r]
                                .iter()
                                .map(|&a| ARMS[a])
                                .collect::<Vec<_>>()
                                .join("")
                        )
                    })
                    .collect();
                println!("{r}\t{}", o.join(" "));
            }
        }
        Some("--check") => check(&args[1..]),
        Some("--audit") => audit(&args[1..]),
        Some("--trial") => trial(&args[1..]),
        Some("--child") => child(&args[1..]),
        _ => {
            eprintln!("usage: see the header of examples/p74c_decoder.rs");
            std::process::exit(2);
        }
    }
}
