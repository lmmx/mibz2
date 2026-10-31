//! Preparation/checking by default. Timing requires explicit --measure-approved.
#[path = "../tests/common/mod.rs"]
mod common;
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item, Symbols, BLOCK_MAGIC};
use mibz2::mtf::{self, p74_c, p74_c::Arm};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::{hint::black_box, path::Path, time::Instant};

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

struct Oracle {
    ll: Vec<u8>,
    counts: [u32; 256],
    cps: Vec<[u32; 256]>,
}
fn apply(b: &Symbols, o: &mut Oracle, cp: bool, arm: Arm, k: Kernels) -> usize {
    let r = if cp {
        p74_c::mtf_checkpoints(
            black_box(b),
            &mut o.ll,
            &mut o.counts,
            &mut o.cps,
            black_box(k),
            black_box(arm),
        )
    } else {
        p74_c::mtf(
            black_box(b),
            &mut o.ll,
            &mut o.counts,
            black_box(k),
            black_box(arm),
        )
    };
    black_box(r.unwrap())
}
fn pass(blocks: &[Symbols], refs: &[Oracle], cp: bool, arm: Arm, timed: bool) -> u128 {
    let mut o = Oracle {
        ll: Vec::new(),
        counts: [0; 256],
        cps: Vec::new(),
    };
    // Untimed reservation: all arms get equivalent reusable capacity.
    let max = blocks.iter().map(|b| b.max_len).max().unwrap_or(0);
    o.ll.reserve(max + 32);
    o.cps.reserve(max / mtf::CHECKPOINT + 2);
    let k = Kernels::detect();
    let mut ns = 0;
    for (b, want) in blocks.iter().zip(refs) {
        let n = if timed {
            let start = Instant::now();
            let n = apply(b, &mut o, cp, arm, k);
            ns += start.elapsed().as_nanos();
            n
        } else {
            apply(b, &mut o, cp, arm, k)
        };
        assert_eq!(n, want.ll.len());
        assert_eq!(o.ll, want.ll);
        assert_eq!(o.counts, want.counts);
        if cp {
            assert_eq!(o.cps, want.cps);
        }
        black_box((&o.ll, &o.counts, &o.cps));
    }
    ns
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--prepare") => {
            let dir = Path::new(&args[1]);
            std::fs::create_dir_all(dir).unwrap();
            for (name, data) in common::corpora(4) {
                if ["random", "records", "text"].contains(&name) {
                    std::fs::write(dir.join(format!("{name}.bz2")), common::compress(&data, 9))
                        .unwrap();
                }
            }
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
            return;
        }
        Some("--windows") => {
            windows(&args[1], &args[2..]);
            return;
        }
        _ => {}
    }
    let timed = args.first().map(String::as_str) == Some("--measure-approved");
    assert!(timed || args.first().map(String::as_str)==Some("--check"),"use --prepare DIR, --windows OUT WINDOWS..., --check FILES..., or --measure-approved FILES...");
    if timed {
        assert!(
            !std::env::var("P74_APPROVED_RUNNER")
                .unwrap_or_default()
                .is_empty(),
            "explicit runner approval required"
        );
        if let Ok(quota) = std::fs::read_to_string("/sys/fs/cgroup/cpu.max") {
            assert!(quota.starts_with("max "), "quota-limited hosts excluded");
        }
    }
    assert!(
        Kernels::detect().avx2,
        "AVX2 required to validate distinct research arms"
    );
    for (file, path) in args[1..].iter().enumerate() {
        let z = std::fs::read(path).unwrap();
        let decoded = common::oracle(&z).unwrap();
        assert_eq!(mibz2::decode(&z).unwrap(), decoded);
        let mut sc = Scanner::new(SliceSource(&z));
        let mut blocks = Vec::new();
        let mut refs = Vec::new();
        let mut independent = Vec::new();
        while let Event::Block { symbols: b, .. } = sc.next(Vec::new()).unwrap() {
            let mut o = Oracle {
                ll: Vec::new(),
                counts: [0; 256],
                cps: Vec::new(),
            };
            mtf::mtf_checkpoints(&b, &mut o.ll, &mut o.counts, &mut o.cps, Kernels::detect())
                .unwrap();
            independent.extend(common::reference(&o.ll, b.orig_ptr).unwrap());
            blocks.push(b);
            refs.push(o);
        }
        assert_eq!(independent, decoded);
        let bytes: usize = refs.iter().map(|r| r.ll.len()).sum();
        println!(
            "META\t{file}\t{}\t{}\t{bytes}\t{}\t{}",
            blocks.len(),
            blocks.iter().map(|b| b.syms.len()).sum::<usize>(),
            decoded.len(),
            refs.iter().map(|r| r.cps.len()).sum::<usize>()
        );
        for cp in [false, true] {
            // Both check-only and measured invocation validate/warm every arm twice.
            for _ in 0..2 {
                for arm in [Arm::T0, Arm::T1, Arm::T2] {
                    pass(&blocks, &refs, cp, arm, false);
                }
            }
            if !timed {
                continue;
            }
            // All six orders twice, shuffled once with a registered fixed seed.
            let mut orders = [
                [0, 1, 2],
                [0, 2, 1],
                [1, 0, 2],
                [1, 2, 0],
                [2, 0, 1],
                [2, 1, 0],
            ]
            .repeat(2);
            let mut rng = common::Rng(0x74c0_2026_1031_0002);
            for i in (1..orders.len()).rev() {
                orders.swap(i, rng.below(i as u64 + 1) as usize);
            }
            for (round, order) in orders.iter().enumerate() {
                for (position, &a) in order.iter().enumerate() {
                    let arm = [Arm::T0, Arm::T1, Arm::T2][a];
                    let mut ns = 0;
                    for _ in 0..2 {
                        ns += pass(&blocks, &refs, cp, arm, true);
                    }
                    println!(
                        "ROW\t{file}\t{}\t{round}\t{position}\t{a}\t{ns}\t{}",
                        usize::from(cp),
                        bytes * 2
                    );
                }
            }
        }
    }
}
