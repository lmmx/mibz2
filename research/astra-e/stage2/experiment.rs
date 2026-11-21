//! Paired single-worker stage experiment for Astra E. Run through
//! research/astra-e/stage/run.sh to verify the pinned compressed corpus digest.
//! The clock encloses invert_runs on ready MTF records. All stream setup,
//! equality checks, RLE1, CRC, output delivery and logging are outside it.
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
use std::{env, fs, time::Instant};

#[allow(dead_code)]
#[path = "../../../docs/journal/2026-11-18-open-return/open_return.rs"]
mod open_return;
#[allow(dead_code)]
#[path = "../lookahead/candidate.rs"]
mod lookahead;
#[allow(dead_code)]
#[path = "../dictionary/candidate.rs"]
mod dictionary;
#[allow(dead_code)]
#[path = "../open/compact/candidate.rs"]
mod compact_f;
#[allow(dead_code)]
#[path = "../short-words/candidate.rs"]
mod short_words;
const ORDINALS: [u64; 11] = [1, 19, 61, 108, 150, 194, 240, 282, 329, 371, 407];

struct Block {
    ordinal: u64,
    n: usize,
    primary: u32,
    runs: runs::Runs,
    reference: Vec<u8>,
}

fn sample(data: &[u8], kernels: Kernels) -> Vec<Block> {
    let mut scanner = Scanner::new(SliceSource(data));
    let mut lazy = bwt::LazyCells::default();
    let mut eligible = Vec::new();
    let mut blocks = Vec::new();
    while let Event::Block { index, symbols, .. } = scanner.next(Vec::new()).unwrap() {
        if !runs::takes_runs(&symbols) {
            continue;
        }
        let mut runs = runs::Runs::default();
        let n = runs::front(&symbols, &mut runs, kernels).unwrap();
        if n < 65_536 || runs.records().len() * 20 >= n * 3 {
            continue;
        }
        eligible.push(index);
        if ORDINALS.contains(&index) {
            let mut reference = Vec::new();
            assert!(bwt::InverseBwt::invert_runs(
                &mut lazy,
                &runs,
                n,
                symbols.orig_ptr,
                &mut reference,
                kernels
            ));
            blocks.push(Block {
                ordinal: index,
                n,
                primary: symbols.orig_ptr,
                runs,
                reference,
            });
        }
    }
    // The frozen first block plus the midpoint of every decile of large eligible
    // blocks, in scanner order. Detect a changed stream or eligibility rule.
    assert_eq!(eligible.len(), 228);
    assert_eq!(eligible[0], ORDINALS[0]);
    for j in 0..10 {
        assert_eq!(eligible[(2 * j + 1) * eligible.len() / 20], ORDINALS[j + 1]);
    }
    assert_eq!(blocks.len(), ORDINALS.len());
    blocks
}

type Stage = Box<dyn FnMut(&Block, &mut Vec<u8>) -> bool>;
struct Arm {
    name: &'static str,
    run: Stage,
    out: Vec<u8>,
}

fn arm<K: bwt::InverseBwt + 'static>(name: &'static str, kernels: Kernels) -> Arm {
    let mut kernel = K::default();
    Arm {
        name,
        run: Box::new(move |b, out| kernel.invert_runs(&b.runs, b.n, b.primary, out, kernels)),
        out: Vec::new(),
    }
}

// Stage harness for the target Linux x86-64 ABI. Read only the current thread's
// CPU time. Both CPU clock calls surround the wall clock, outside its interval.
#[cfg(not(all(target_os = "linux", target_pointer_width = "64")))]
compile_error!("The thread CPU clock in this stage harness requires 64-bit Linux.");
fn cpu_ns() -> u64 {
    use std::ffi::c_long;
    #[repr(C)]
    struct Timespec { secs: c_long, nanos: c_long }
    unsafe extern "C" { fn clock_gettime(id: i32, out: *mut Timespec) -> i32; }
    let mut stamp = Timespec { secs: 0, nanos: 0 };
    // SAFETY: stamp is a live, correctly aligned timespec. The Linux caller
    // receives two longs from the current thread clock and checks its return.
    assert_eq!(unsafe { clock_gettime(3, &mut stamp) }, 0);
    stamp.secs as u64 * 1_000_000_000 + stamp.nanos as u64
}

fn main() {
    let path = env::args().nth(1).expect("astra_e FILE.bz2 [ROUNDS]");
    let rounds: usize = env::args().nth(2).map_or(11, |r| r.parse().unwrap());
    let kernels = Kernels::detect();
    let blocks = sample(&fs::read(path).unwrap(), kernels);
    let mut arms = [
        arm::<bwt::LazyCells>("lazy", kernels),
        arm::<bwt::Cells>("eager", kernels),
        arm::<open_return::Open<16, true, true>>("opend16", kernels),
        arm::<lookahead::Candidate>("lookahead", kernels),
        arm::<dictionary::Candidate>("dictionary", kernels),
        arm::<compact_f::Candidate>("compact_f", kernels),
        arm::<short_words::Candidate>("short_words", kernels),
    ];
    if env::args().nth(3).as_deref() == Some("probe") {
        let mut probe = arm::<short_words::ShortWords<true>>("probe", kernels);
        for block in &blocks {
            assert!((probe.run)(block, &mut probe.out) && probe.out == block.reference);
        }
        return;
    }
    let count = arms.len();
    println!("round,position,ordinal,rows,primary,records,arm,elapsed_ns,thread_cpu_ns");
    for pass in 0..=rounds {
        for (bi, b) in blocks.iter().enumerate() {
            for position in 0..count {
                let i = (pass + bi + position) % count;
                let arm = &mut arms[i];
                let cpu_begin = (pass > 0).then(cpu_ns);
                let begin = (pass > 0).then(Instant::now);
                let completed = (arm.run)(b, &mut arm.out);
                let elapsed = begin.map(|start| start.elapsed().as_nanos());
                let cpu = cpu_begin.map(|start| cpu_ns() - start);
                assert!(
                    completed && arm.out == b.reference,
                    "arm {} block {}",
                    arm.name,
                    b.ordinal
                );
                if let Some(elapsed) = elapsed {
                    println!(
                        "{},{},{},{},{},{},{},{},{}",
                        pass - 1,
                        position,
                        b.ordinal,
                        b.n,
                        b.primary,
                        b.runs.records().len(),
                        arm.name,
                        elapsed, cpu.unwrap()
                    );
                }
            }
        }
    }
}
