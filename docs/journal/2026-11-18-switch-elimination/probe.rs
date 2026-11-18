use mibz2::{
    Kernels,
    bwt::{Cells, InverseBwt, Plain},
    stream::{Event, Scanner, SliceSource},
    switch_elimination::{Switches, coincidence::Coincidence, forest::Forest},
};
use std::{env, fs, time::Instant};

fn census(data: &[u8], chains: bool) {
    let kernels = Kernels::detect();
    let mut scan = Scanner::new(SliceSource(data));
    let (mut symbols, mut column, mut expected, mut out) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut plain, mut network) = (Plain::default(), Switches::default());
    let mut runs = mibz2::runs::Runs::default();
    let mut ordinal = 0;
    let mut chain = Coincidence::default();
    if chains {
        println!(
            "ordinal,front,n,r,pass,rows,bands,common,removed,next_rows,next_bands,chains,joined,children,budget,roots,opened,literal_bytes,copies,copied_bytes"
        );
    } else {
        println!(
            "ordinal,front,pass,vertices,bands,rows,left_vertices,left_bands,left_rows,loop_vertices,joins,budget,roots,opened,literals,copies,copied_bytes"
        );
    }
    while let Event::Block { symbols: block, .. } = scan.next(symbols).unwrap() {
        let mut counts = [0u32; 256];
        mibz2::mtf::mtf(&block, &mut column, &mut counts, kernels).unwrap();
        plain.invert(&column, &counts, block.orig_ptr, &mut expected);
        let front = mibz2::runs::takes_runs(&block);
        if front {
            let n = mibz2::runs::front(&block, &mut runs, kernels).unwrap();
            let ok = if chains {
                chain.invert_runs(&runs, n, block.orig_ptr, &mut out, kernels)
            } else {
                network.invert_runs(&runs, n, block.orig_ptr, &mut out, kernels)
            };
            assert!(ok);
        } else if chains {
            chain.invert(&column, &counts, block.orig_ptr, &mut out);
        } else {
            network.invert(&column, &counts, block.orig_ptr, &mut out);
        }
        assert_eq!(out, expected, "block {ordinal}");
        if chains {
            let a = chain.stats();
            let n = column.len();
            let r = 1 + column.windows(2).filter(|x| x[0] != x[1]).count();
            if chain.rounds.is_empty() {
                println!(
                    "{ordinal},{},{n},{r},0,{n},{r},0,0,{n},{r},0,0,0,{},{},{},{},{},{}",
                    u8::from(front),
                    u8::from(chain.budget_stop),
                    a.roots,
                    a.opened,
                    a.literal_bytes,
                    a.copies,
                    a.copied
                );
            }
            for (i, s) in chain.rounds.iter().enumerate() {
                println!(
                    "{ordinal},{},{n},{r},{i},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    u8::from(front),
                    s.rows,
                    s.bands,
                    s.common,
                    s.removed,
                    s.next_rows,
                    s.next_bands,
                    s.chains,
                    s.joined,
                    s.children,
                    u8::from(chain.budget_stop),
                    a.roots,
                    a.opened,
                    a.literal_bytes,
                    a.copies,
                    a.copied
                );
            }
        } else {
            let a = network.output_stats();
            for (i, p) in network.passes.iter().enumerate() {
                println!(
                    "{ordinal},{},{i},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    u8::from(front),
                    p.before_v,
                    p.before_e,
                    p.before_m,
                    p.after_v,
                    p.after_e,
                    p.after_m,
                    p.loop_vertices,
                    p.splices,
                    u8::from(p.budget),
                    a.roots,
                    a.opened,
                    a.literals,
                    a.hits,
                    a.copied
                );
            }
        }
        symbols = block.syms;
        ordinal += 1;
    }
}

fn decode(arm: &str, data: &[u8]) -> Vec<u8> {
    let k = Kernels::detect();
    match arm {
        "forest" => mibz2::decode_with::<Forest>(data, k).unwrap(),
        "chains" => mibz2::decode_with::<Coincidence>(data, k).unwrap(),
        "switches" => mibz2::decode_with::<Switches>(data, k).unwrap(),
        "cells" => mibz2::decode_with::<Cells>(data, k).unwrap(),
        _ => panic!("unknown decoder {arm}"),
    }
}

fn price(data: &[u8], laps: usize) {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() else {
        panic!("empty stream")
    };
    let mut runs = mibz2::runs::Runs::default();
    let n = mibz2::runs::front(&symbols, &mut runs, k).unwrap();
    let (mut cells, mut switch, mut chains, mut forest) = (
        Cells::default(),
        Switches::default(),
        Coincidence::default(),
        Forest::default(),
    );
    let (mut expected, mut out) = (Vec::new(), Vec::new());
    assert!(cells.invert_runs(&runs, n, symbols.orig_ptr, &mut expected, k));
    println!("lap,arm,total_seconds,build_seconds,emit_seconds");
    for lap in 0..=laps {
        for j in 0..4 {
            let start = Instant::now();
            let (arm, build, emit) = if (j + lap) % 4 == 0 {
                assert!(cells.invert_runs(&runs, n, symbols.orig_ptr, &mut out, k));
                ("cells", 0.0, 0.0)
            } else if (j + lap) % 4 == 1 {
                assert!(chains.invert_runs(&runs, n, symbols.orig_ptr, &mut out, k));
                ("chains", chains.build_seconds, chains.output_seconds)
            } else if (j + lap) % 4 == 2 {
                assert!(forest.invert_runs(&runs, n, symbols.orig_ptr, &mut out, k));
                ("forest", forest.build_seconds, forest.output_seconds)
            } else {
                assert!(switch.invert_runs(&runs, n, symbols.orig_ptr, &mut out, k));
                ("switches", switch.build_seconds, switch.output_seconds)
            };
            let elapsed = start.elapsed().as_secs_f64();
            assert_eq!(expected, out);
            println!("{lap},{arm},{elapsed:.9},{build:.9},{emit:.9}");
        }
    }
}

fn forest_census(data: &[u8]) {
    let k = Kernels::detect();
    let mut scan = Scanner::new(SliceSource(data));
    let (mut syms, mut col, mut oracle, mut out) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut plain, mut forest, mut records) = (
        Plain::default(),
        Forest::default(),
        mibz2::runs::Runs::default(),
    );
    let mut ordinal = 0;
    println!(
        "ordinal,n,front,layer,v,e,rows,chords,retained,slabs,pair_tests,peak_pairs,bands,recipes,children_read,budget,roots,copied_bytes,zero_gain_mass"
    );
    while let Event::Block { symbols: b, .. } = scan.next(syms).unwrap() {
        let mut hist = [0; 256];
        mibz2::mtf::mtf(&b, &mut col, &mut hist, k).unwrap();
        plain.invert(&col, &hist, b.orig_ptr, &mut oracle);
        let front = mibz2::runs::takes_runs(&b);
        if front {
            let n = mibz2::runs::front(&b, &mut records, k).unwrap();
            assert!(forest.invert_runs(&records, n, b.orig_ptr, &mut out, k));
        } else {
            forest.invert(&col, &hist, b.orig_ptr, &mut out);
        }
        assert_eq!(oracle, out, "ordinal {ordinal}");
        let n = col.len();
        if forest.layers.is_empty() {
            println!(
                "{ordinal},{n},{},-1,0,0,{n},0,{n},0,0,0,0,0,0,{},{},{},0",
                u8::from(front),
                u8::from(forest.budget_stop),
                forest.stats().roots,
                forest.stats().copied
            );
        }
        for (i, s) in forest.layers.iter().enumerate() {
            println!(
                "{ordinal},{n},{},{i},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                u8::from(front),
                s.vertices,
                s.arcs,
                s.rows,
                s.chords,
                s.retained,
                s.slabs,
                s.pair_tests,
                s.peak_pairs,
                s.bands,
                s.rules,
                forest.children_read(),
                u8::from(forest.budget_stop),
                forest.stats().roots,
                forest.stats().copied,
                s.zero_gain_mass
            );
        }
        syms = b.syms;
        ordinal += 1;
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let data = fs::read(&args[2]).unwrap();
    match args[1].as_str() {
        "census" | "chains" => census(&data, args[1] == "chains"),
        "forest" => forest_census(&data),
        "dump" => {
            let mut scan = Scanner::new(SliceSource(&data));
            let Event::Block { symbols, .. } = scan.next(Vec::new()).unwrap() else {
                panic!("empty")
            };
            let mut column = Vec::new();
            mibz2::mtf::mtf(&symbols, &mut column, &mut [0; 256], Kernels::detect()).unwrap();
            fs::write(&args[3], column).unwrap();
            println!("{}", symbols.orig_ptr);
        }
        "price" => price(&data, args[3].parse().unwrap()),
        "write" => fs::write(&args[4], decode(&args[3], &data)).unwrap(),
        "stream" => {
            let expected = decode("cells", &data);
            println!("lap,arm,seconds,bytes");
            for lap in 0..=args[3].parse::<usize>().unwrap() {
                let arms: Vec<_> = args
                    .get(4)
                    .map_or("cells,chains,switches,forest", |s| s.as_str())
                    .split(',')
                    .collect();
                for j in 0..arms.len() {
                    let arm = arms[(j + lap) % arms.len()];
                    let start = Instant::now();
                    let actual = decode(arm, &data);
                    let t = start.elapsed().as_secs_f64();
                    assert_eq!(actual, expected);
                    println!("{lap},{arm},{t:.9},{}", actual.len());
                }
            }
        }
        _ => panic!("use census, price, write or stream"),
    }
}
