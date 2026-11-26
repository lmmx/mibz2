//! Paired single-worker stage experiment for Astra E. Run through
//! research/astra-e/stage/run.sh to verify the pinned compressed corpus digest.
//! The clock encloses invert_runs on ready MTF records. All stream setup,
//! equality checks, RLE1, CRC, output delivery and logging are outside it.
// Research harness: its code lives in research/ and is exempt from lint (#610); correctness
// is gated by the build, the tests and the scorer's byte checks
#![allow(clippy::all)]
// Two experimental kernels bind the same endpoint source to distinct parent types.
#![allow(clippy::duplicate_mod)]
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
use std::{env, fs, time::Instant};

#[rustfmt::skip]
#[allow(dead_code, clippy::too_many_arguments)]
#[path = "../research/astra-e/open/candidate.rs"]
mod conjugate;
#[allow(dead_code)]
#[path = "../research/astra-e/cells/hybrid/candidate.rs"]
mod hybrid;
#[allow(dead_code)]
#[path = "../research/astra-e/dense/candidate.rs"]
mod one;
#[allow(dead_code, clippy::too_many_arguments)]
#[path = "../docs/journal/2026-11-18-open-return/open_return.rs"]
mod open_return;
#[rustfmt::skip]
#[allow(dead_code, clippy::too_many_arguments)]
#[path = "../research/astra-e/bidirectional/candidate.rs"]
mod opposed;
#[allow(dead_code)]
#[path = "../research/astra-e/cells/packed.rs"]
mod packed;
#[allow(dead_code, clippy::int_plus_one)]
#[path = "../research/astra-e/dense/two_step.rs"]
mod two;

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

fn main() {
    let path = env::args().nth(1).expect("astra_e FILE.bz2 [ROUNDS]");
    let rounds: usize = env::args().nth(2).map_or(11, |r| r.parse().unwrap());
    let kernels = Kernels::detect();
    let blocks = sample(&fs::read(path).unwrap(), kernels);
    let mut arms = [
        arm::<bwt::LazyCells>("lazy", kernels),
        arm::<open_return::Open<16, true, true>>("opend16", kernels),
        arm::<open_return::Open<8, true, true>>("open8", kernels),
        arm::<open_return::Open<32, true, true>>("open32", kernels),
        arm::<one::Candidate>("page_trace", kernels),
        arm::<two::Candidate>("two_step", kernels),
        arm::<conjugate::Candidate>("conjugate", kernels),
        arm::<opposed::Candidate>("opposed", kernels),
        arm::<packed::Candidate>("packed_eager", kernels),
        arm::<hybrid::Candidate>("hybrid_eager", kernels),
        arm::<bwt::Cells>("eager", kernels),
    ];
    let count = arms.len();
    println!("round,position,ordinal,rows,primary,records,arm,elapsed_ns");
    for pass in 0..=rounds {
        for (bi, b) in blocks.iter().enumerate() {
            for position in 0..count {
                let i = (pass + bi + position) % count;
                let arm = &mut arms[i];
                let begin = (pass > 0).then(Instant::now);
                let completed = (arm.run)(b, &mut arm.out);
                let elapsed = begin.map(|start| start.elapsed().as_nanos());
                assert!(
                    completed && arm.out == b.reference,
                    "arm {} block {}",
                    arm.name,
                    b.ordinal
                );
                if let Some(elapsed) = elapsed {
                    println!(
                        "{},{},{},{},{},{},{},{}",
                        pass - 1,
                        position,
                        b.ordinal,
                        b.n,
                        b.primary,
                        b.runs.records().len(),
                        arm.name,
                        elapsed
                    );
                }
            }
        }
    }
}
