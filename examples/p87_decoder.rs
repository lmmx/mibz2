//! P87's four-core diagnostic (docs/journal/2026-11-01-p87-four-core-protocol.md): one
//! executable, both arms, every measurement its own child process. Checking by default;
//! timing only with `--trial`, under the runner's approval. Ported from the unrolled walk's
//! decoder trial harness (`examples/walk_decoder.rs`, research/walk-unroll-decoder
//! `f5c492d`), itself the P74-C trial's: its allocation, plan, thread-inventory and
//! affinity checks, children, schedule and output are kept; the arms differ, the windows
//! and preparation modes are dropped (the corpora are the walk trial's frozen files), and
//! each child's placement is new.
//!
//! - `--schedule MANIFEST WORKERS`: the registered order (no timing).
//! - `--check MANIFEST WORKERS`: every corpus decoded by arms A and B and the shipped path
//!   at every w through the trial's decode, and sequentially, each output's SHA-256 the
//!   expected one (no timing).
//! - `--trial MANIFEST WORKERS ALLOC`: the timed trial (needs `P87_APPROVED_RUNNER`).
//! - `--child ARM W PATH SHA256 BYTES PLACEMENT`: one measurement (the trial's children).
//!
//! `P87_SYNTHETIC_TIMES=1` (the runner's harness self-test, before any timing): `--trial`
//! and `--child` run every step, but every time and CPU figure they write is a fixed
//! synthetic value, never a measurement; no runner approval is needed for it.
//!
//! **The arms differ only in where the inverse BWT's assembly writes** (`src/p87.rs`): A
//! and A′ into `rle` (the shipped buffer), B into `ll` once the build has read it, both
//! through `mibz2::p87::decode_block`, one compiled function; the decoder is
//! `StreamDecoder::with_inverse::<Segmented>(w, 2w, Kernels::detect())`, as
//! `Plan::Workers(w)` builds it, in every arm. The same binary, kernels, feeder, reader,
//! scanner and workers' code. PLACEMENT (`0`: `ll` allocated first, `1`: `rle` first) is
//! the round's parity, the same for every arm in a round.
//!
//! A MANIFEST is TSV: name, role, path, compressed SHA-256, decoded SHA-256, decoded bytes.
//! WORKERS and ALLOC are comma-separated (`1,4`; CPUs `1,2,3,4`). The CHILD and ROW lines
//! keep the P74-C harness's fields (its selection fields written as `0`), with the
//! placement appended to each ROW.
#[path = "../tests/common/mod.rs"]
mod common;
use mibz2::bwt::Segmented;
use mibz2::p87::{self, Arm};
use mibz2::parallel::Plan;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{Kernels, StreamDecoder};
use std::io::Write as _;
use std::path::Path;
use std::time::Instant;

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
    record_of(dir, &status)
}

/// The record of a thread whose status has been read
fn record_of(dir: &Path, status: &str) -> String {
    let stat = std::fs::read_to_string(dir.join("stat")).unwrap_or_default();
    // fields after the command: state is field 3, processor (psr) field 39
    let psr = stat
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().nth(36).map(str::to_string))
        .unwrap_or_default();
    format!(
        "{}|{}|{}|{}|{}",
        status_field(status, "Name:"),
        status_field(status, "Cpus_allowed_list:"),
        psr,
        status_field(status, "voluntary_ctxt_switches:"),
        status_field(status, "nonvoluntary_ctxt_switches:"),
    )
}

/// A thread listed in /proc/self/task, told apart three ways (the lifecycle fix): alive,
/// its status readable with a CPU list: its record (`name|allowed|psr|vcs|nvcs`, the same
/// two reads as before); exited, its directory gone (a joined thread released while the
/// snapshot was read): `EXITED:<tid>`; unresolved, its directory still there but no
/// readable CPU list after about 10 ms of retries: `UNRESOLVED:<tid>`
fn listed_record(dir: &Path) -> String {
    let tid = dir
        .file_name()
        .map(|t| t.to_string_lossy().into_owned())
        .unwrap_or_default();
    for _ in 0..100 {
        if let Ok(status) = std::fs::read_to_string(dir.join("status")) {
            if !status_field(&status, "Cpus_allowed_list:").is_empty() {
                return record_of(dir, &status);
            }
        }
        if !dir.exists() {
            return format!("EXITED:{tid}");
        }
        std::thread::sleep(std::time::Duration::from_micros(100));
    }
    format!("UNRESOLVED:{tid}")
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
            v.push(listed_record(&e.path()));
        }
    }
    (v.len(), v.join(";"))
}

// ---------------------------------------------------------------- one decode

const ARMS: [&str; 3] = ["A", "B", "A2"];

/// The arm's assembly destination (`mibz2::p87`): A and A′ into `rle`, B into `ll`
fn p87_arm(arm: usize) -> Arm {
    match arm {
        1 => Arm::IntoLl,
        0 | 2 => Arm::IntoRle,
        _ => panic!("arm {arm}"),
    }
}

/// The decoder, the same in every arm (`Plan::Workers(w)`'s); the arm is the process's
/// (`p87::set`, before this)
fn decoder(_arm: usize, w: usize) -> StreamDecoder {
    StreamDecoder::with_inverse::<Segmented>(w, 2 * w, Kernels::detect())
}

struct Decoded {
    out: Vec<u8>,
    wall_ns: u128,
    cpu: (i64, i64),
    threads_spawned: (usize, String),
}

/// `z` through the arm's parallel decoder (`decoder`), fed and read as
/// `decode_parallel` does (4 MiB chunks from a feeder thread, 4 MiB reads): the wall and
/// process CPU from just before the decoder is built to the read that returns 0
fn decode_once(z: &[u8], w: usize, arm: usize, expected_len: usize) -> Decoded {
    let mut out = Vec::with_capacity(expected_len);
    let mut buf = vec![0u8; 4 << 20];
    let feeder = std::sync::Mutex::new(String::new());
    let ru0 = rusage_self();
    let t0 = Instant::now();
    let dec = decoder(arm, w);
    // every decoder thread spawned and blocked (no input yet: none can have ended): the
    // workers, the scanner and main, their allowed CPUs; the same cost in every arm
    // (a thread's name can still be the parent's until it starts)
    let mut spawned = threads();
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .name("p87-feeder".into())
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
        threads_spawned: spawned,
    }
}

/// The checks every child and `--check` make: the plan the default policy would take
/// here is `Workers(w)` (so production's decoder here is `with_inverse::<Segmented>`)
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
/// seed 0x8720261101_0001: (orders[cell][round], rounds[round] = cell order)
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
    let mut rng = common::Rng(0x87_2026_1101_0001);
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
    let (arm, w, path, sha, bytes, placement) = (
        args[0].parse::<usize>().unwrap(),
        args[1].parse::<usize>().unwrap(),
        &args[2],
        &args[3],
        args[4].parse::<usize>().unwrap(),
        args[5].parse::<usize>().unwrap(),
    );
    assert!(placement <= 1, "placement {placement}");
    p87::set(p87_arm(arm), placement == 1);
    let (plan, allowed, phys) = plan_and_cpus(w);
    let z = std::fs::read(path).unwrap();
    let mut d = decode_once(&z, w, arm, bytes);
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
    // (blocks and selections: the P74-C fields, 0 here: no selector)
    println!(
        "CHILD\t{}\t{}\t{}\t{}\t1\t0\t0\t{plan:?}\t{allowed}\t{phys}\t{}\t{}\t{}\t{}",
        d.wall_ns,
        d.cpu.0,
        d.cpu.1,
        d.out.len(),
        d.threads_spawned.0,
        d.threads_spawned.1,
        end.0,
        end.1,
    );
}

/// Every thread's allowed CPUs within `alloc` (`name|allowed|...;...`). An `EXITED:`
/// entry is accepted only where `allow_exited` (the end snapshot, taken after the joins);
/// an `UNRESOLVED:` entry, an empty CPU list or a CPU outside `alloc` always fails
fn assert_confined(snapshot: &str, alloc: &[usize], what: &str, allow_exited: bool) {
    for t in snapshot.split(';').filter(|t| !t.is_empty()) {
        if t.starts_with("EXITED:") {
            assert!(
                allow_exited,
                "{what}: thread {t} exited in a snapshot that must list only live threads"
            );
            continue;
        }
        assert!(
            !t.starts_with("UNRESOLVED:"),
            "{what}: thread {t}: its status could not be resolved"
        );
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
    std::env::var("P87_SYNTHETIC_TIMES").is_ok_and(|v| v == "1")
}

fn trial(args: &[String]) {
    assert!(
        synthetic()
            || !std::env::var("P87_APPROVED_RUNNER")
                .unwrap_or_default()
                .is_empty(),
        "the timed trial runs only from bench/p87_run.sh"
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
        // the round's parity: every arm of a round alike, each arm half its rounds each way
        let placement = usize::from(round >= 0 && round % 2 == 1);
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
            .arg(placement.to_string())
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
        // the spawn snapshot (every decoder thread alive): live threads only
        assert_confined(f[12], &alloc, &corpus.name, false);
        // the end snapshot, after the joins: a released thread is exited, not a violation
        assert_confined(f[14], &alloc, &corpus.name, true);
        writeln!(
            out,
            "ROW\t{round}\t{cell_pos}\t{c}\t{}\t{w}\t{arm}\t{position}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{life_ns}\t{placement}",
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
        for arm in [0, 1, 3] {
            // (3: the shipped path, unset; A and B through `p87::decode_block`)
            p87::set(if arm == 3 { Arm::Shipped } else { p87_arm(arm) }, false);
            for &w in &workers {
                plan_and_cpus(w);
                settle_threads();
                let d = decode_once(&z, w, arm, c.bytes);
                let name = if arm == 3 { "shipped" } else { ARMS[arm] };
                assert_eq!(d.out.len(), c.bytes, "{} arm {name} w {w}", c.name);
                assert_eq!(sha256(&d.out), c.decoded, "{} arm {name} w {w}", c.name);
                let threads = d.threads_spawned.0;
                assert_eq!(
                    threads,
                    w + 3,
                    "w workers, the scanner, the feeder and main"
                );
                row += &format!("\t{name}w{w}:threads={threads}");
            }
            let s = mibz2::decode_with::<Segmented>(&z, Kernels::detect()).unwrap();
            let name = if arm == 3 { "shipped" } else { ARMS[arm] };
            assert_eq!(sha256(&s), c.decoded, "{} arm {name} sequential", c.name);
        }
        println!("{row}\tok");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !cfg!(all(target_os = "linux", target_pointer_width = "64")) {
        eprintln!("p87_decoder: Linux (64-bit) only");
        std::process::exit(2);
    }
    match args.first().map(String::as_str) {
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
        Some("--trial") => trial(&args[1..]),
        Some("--child") => child(&args[1..]),
        _ => {
            eprintln!("usage: see the header of examples/p87_decoder.rs");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod lifecycle_tests {
    //! The lifecycle fix's three cases, and the race it handles reproduced
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("lifecycle-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn a_vanished_directory_is_exited() {
        let d = tmp("gone").join("4242");
        assert_eq!(listed_record(&d), "EXITED:4242");
    }

    #[test]
    fn a_directory_without_a_readable_cpu_list_is_unresolved() {
        let d = tmp("noread").join("4243");
        std::fs::create_dir_all(&d).unwrap();
        assert_eq!(listed_record(&d), "UNRESOLVED:4243");
        std::fs::write(d.join("status"), "Name:\tx\nCpus_allowed_list:\t\n").unwrap();
        assert_eq!(listed_record(&d), "UNRESOLVED:4243", "an empty CPU list");
        std::fs::write(d.join("status"), "Name:\tx\nCpus_allowed_list:\t0-1\n").unwrap();
        assert!(
            listed_record(&d).starts_with("x|0-1|"),
            "readable: its record"
        );
    }

    #[test]
    fn the_checks_three_ways() {
        let alloc = [0, 2];
        assert_confined("main|0,2|0|1|0;EXITED:7", &alloc, "end", true);
        for (snap, allow) in [
            ("main|0,2|0|1|0;EXITED:7", false),
            ("main|0,2|0|1|0;UNRESOLVED:7", true),
            ("main|0,2|0|1|0;w||||", true),
            ("main|0-3|0|1|0", true),
        ] {
            let r = std::panic::catch_unwind(|| assert_confined(snap, &alloc, "t", allow));
            assert!(r.is_err(), "{snap} (allow_exited {allow}) must fail");
        }
    }

    /// Threads joined and the process's tasks listed at once, many times: every listed task
    /// is alive with a CPU list or exited; never unresolved, never an empty record
    #[test]
    fn joined_threads_are_never_unresolved() {
        let mut exited = 0;
        for _ in 0..3000 {
            let hs: Vec<_> = (0..3).map(|_| std::thread::spawn(|| ())).collect();
            for h in hs {
                h.join().unwrap();
            }
            let (_, snap) = threads();
            for t in snap.split(';') {
                assert!(!t.starts_with("UNRESOLVED:"), "{t}");
                assert!(
                    t.starts_with("EXITED:") || !t.split('|').nth(1).unwrap_or("").is_empty(),
                    "{t}"
                );
                exited += usize::from(t.starts_with("EXITED:"));
            }
        }
        eprintln!(
            "lifecycle: {exited} records exited across 3000 snapshots taken right after joins"
        );
    }
}
