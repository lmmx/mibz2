use mibz2::bwt::{Cells, ForcedCells, InverseBwt, LazyCells, Plain, Segmented};
use mibz2::extrema::Extrema;
use mibz2::forest::{Forest, Stats};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::valleys::Valleys;
use mibz2::{Kernels, StreamDecoder};
use std::{env, fs, time::Instant};

fn stream<I: InverseBwt + 'static>(bytes: &[u8], workers: usize) -> Vec<u8> {
    let k = Kernels::detect();
    if workers == 0 {
        return mibz2::decode_with::<I>(bytes, k).unwrap();
    }
    let dec = StreamDecoder::workers_with::<I>(workers, 4 * workers, k);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for chunk in bytes.chunks(4 << 20) {
                dec.write(chunk.to_vec()).unwrap();
            }
            dec.close();
        });
        let mut buf = vec![0; 4 << 20];
        let mut out = Vec::new();
        loop {
            let count = dec.read(&mut buf).unwrap();
            if count == 0 {
                break;
            }
            out.extend_from_slice(&buf[..count]);
        }
        out
    })
}

fn run(arm: &str, data: &[u8], workers: usize) -> Vec<u8> {
    match arm {
        "direct" => {
            assert_eq!(workers, 0);
            mibz2::extrema::direct(data, Kernels::detect()).unwrap()
        }
        "cells" => stream::<Cells>(data, workers),
        "forced" => stream::<ForcedCells>(data, workers),
        "lazy" => stream::<LazyCells>(data, workers),
        "extrema" => stream::<Extrema>(data, workers),
        "valley" => stream::<Valleys<false>>(data, workers),
        "recursive" => stream::<Valleys<true>>(data, workers),
        "segmented" => stream::<Segmented>(data, workers),
        "1" => stream::<Forest<1>>(data, workers),
        "2" => stream::<Forest<2>>(data, workers),
        "3" => stream::<Forest<3>>(data, workers),
        "4" => stream::<Forest<4>>(data, workers),
        "6" => stream::<Forest<6>>(data, workers),
        "8" => stream::<Forest<8>>(data, workers),
        _ => panic!("unknown arm {arm}"),
    }
}

fn sample(k: usize, l: &[u8], p: u32, out: &mut Vec<u8>) -> Stats {
    match k {
        1 => Forest::<1>::column(l, p, out),
        2 => Forest::<2>::column(l, p, out),
        3 => Forest::<3>::column(l, p, out),
        4 => Forest::<4>::column(l, p, out),
        6 => Forest::<6>::column(l, p, out),
        8 => Forest::<8>::column(l, p, out),
        _ => panic!("unknown context length"),
    }
}

fn census(data: &[u8], orders: &str) {
    let orders: Vec<usize> = orders.split(',').map(|s| s.parse().unwrap()).collect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let k = Kernels::detect();
    let (mut ll, mut oracle, mut trial) = (Vec::new(), Vec::new(), Vec::new());
    let mut counts = [0; 256];
    let mut plain = Plain::default();
    println!("block,k,admitted,n,runs,states,slices,cycles,capped,max_depth,path_bytes,cut_rows,cut_slices,pieces,inherited,towers,period,returns");
    while let Event::Block { index, symbols, .. } = sc.next(syms).unwrap() {
        mibz2::mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
        plain.invert(&ll, &counts, symbols.orig_ptr, &mut oracle);
        for &order in &orders {
            let s = sample(order, &ll, symbols.orig_ptr, &mut trial);
            assert_eq!(trial, oracle, "block {index}");
            println!(
                "{index},{order},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                u8::from(mibz2::runs::takes_runs(&symbols)),
                s.n,
                s.runs,
                s.states,
                s.slices,
                s.cycles,
                s.capped,
                s.max_depth,
                s.path_bytes,
                s.cut_rows,
                s.cut_slices,
                s.pieces,
                s.inherited,
                s.towers,
                s.period,
                s.returns
            );
        }
        syms = symbols.syms;
    }
}

fn valleys(data: &[u8]) {
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let mut l = Vec::new();
    let (mut oracle, mut probe) = (Vec::new(), Vec::new());
    let mut counts = [0; 256];
    let mut reference = Plain::default();
    let mut valley = Valleys::<true>::default();
    println!(
        "block,admitted,n,runs,fixed,up,down,minima,marked_minima,period,max_excursion,levels"
    );
    while let Event::Block { index, symbols, .. } = sc.next(syms).unwrap() {
        mibz2::mtf::mtf(&symbols, &mut l, &mut counts, Kernels::detect()).unwrap();
        reference.invert(&l, &counts, symbols.orig_ptr, &mut oracle);
        let s = valley.column(&l, symbols.orig_ptr, &mut probe);
        assert_eq!(probe, oracle, "block {index}");
        let levels = s
            .levels
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join("/");
        println!(
            "{index},{},{},{},{},{},{},{},{},{},{},{levels}",
            u8::from(mibz2::runs::takes_runs(&symbols)),
            s.n,
            s.runs,
            s.fixed,
            s.up,
            s.down,
            s.minima,
            s.marked_minima,
            s.period,
            s.max_excursion
        );
        syms = symbols.syms;
    }
}

fn extrema(data: &[u8]) {
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let (mut l, mut oracle, mut probe) = (Vec::new(), Vec::new(), Vec::new());
    let mut counts = [0; 256];
    let mut reference = Plain::default();
    println!(
        "block,admitted,n,period,rules,used,level,rows,atoms,ports,seeds,minima,leaves,new_rules"
    );
    while let Event::Block { index, symbols, .. } = sc.next(syms).unwrap() {
        mibz2::mtf::mtf(&symbols, &mut l, &mut counts, Kernels::detect()).unwrap();
        reference.invert(&l, &counts, symbols.orig_ptr, &mut oracle);
        let s = Extrema::column(&l, symbols.orig_ptr, &mut probe);
        assert_eq!(probe, oracle, "block {index}");
        let active = u8::from(mibz2::runs::takes_runs(&symbols));
        if s.rounds.is_empty() {
            println!(
                "{index},{active},{},{},{},{},0,{},0,0,0,0,0,0",
                l.len(),
                s.period,
                s.rules,
                s.used,
                l.len()
            );
        }
        for (i, t) in s.rounds.iter().enumerate() {
            println!(
                "{index},{active},{},{},{},{},{i},{},{},{},{},{},{},{}",
                l.len(),
                s.period,
                s.rules,
                s.used,
                t.rows,
                t.atoms,
                t.ports,
                t.seeds,
                t.minima,
                t.leaves,
                t.rules
            );
        }
        syms = symbols.syms;
    }
}

fn compare_small() {
    let mut plain = Plain::default();
    let mut once = Valleys::<false>::default();
    let mut recurse = Valleys::<true>::default();
    let (mut expected, mut got) = (Vec::new(), Vec::new());
    let mut marks = 0;
    for (alphabet, max_len) in [(2usize, 8), (3, 6)] {
        for n in 1..=max_len {
            for mut v in 0..alphabet.pow(n) {
                let mut l = vec![0; n as usize];
                let mut counts = [0; 256];
                for a in &mut l {
                    *a = (v % alphabet) as u8;
                    counts[*a as usize] += 1;
                    v /= alphabet;
                }
                for p in 0..n {
                    plain.invert(&l, &counts, p, &mut expected);
                    once.column(&l, p, &mut got);
                    assert_eq!(got, expected);
                    recurse.column(&l, p, &mut got);
                    assert_eq!(got, expected);
                    Forest::<4>::column(&l, p, &mut got);
                    assert_eq!(got, expected);
                    marks += 1;
                }
            }
        }
    }
    println!("{marks} primary choices, both rank sweeps and context forest matched Plain");
}

fn corruption(data: &[u8]) {
    use mibz2::ErrorKind;
    let mut block = data.to_vec();
    block[10] ^= 1;
    assert!(matches!(
        mibz2::decode_with::<Extrema>(&block, Kernels::detect())
            .unwrap_err()
            .kind,
        ErrorKind::BlockCrc { .. }
    ));
    let info = mibz2::scan(data).next().unwrap().unwrap();
    let bit = info.end_bit as usize + 48;
    let mut stream = data.to_vec();
    stream[bit / 8] ^= 1 << (7 - bit % 8);
    assert!(matches!(
        mibz2::decode_with::<Extrema>(&stream, Kernels::detect())
            .unwrap_err()
            .kind,
        ErrorKind::StreamCrc { .. }
    ));
    assert!(matches!(
        mibz2::extrema::direct(&block, Kernels::detect())
            .unwrap_err()
            .kind,
        ErrorKind::BlockCrc { .. }
    ));
    assert!(matches!(
        mibz2::extrema::direct(&stream, Kernels::detect())
            .unwrap_err()
            .kind,
        ErrorKind::StreamCrc { .. }
    ));
    println!("block and stream CRC mutations rejected");
}
fn main() {
    let args: Vec<_> = env::args().collect();
    if args[1] == "small" {
        compare_small();
        return;
    }
    let bytes = fs::read(&args[2]).unwrap();
    match args[1].as_str() {
        "corrupt" => corruption(&bytes),
        "extrema" => extrema(&bytes),
        "valleys" => valleys(&bytes),
        "census" => census(&bytes, &args[3]),
        "write" => fs::write(&args[4], run(&args[3], &bytes, 0)).unwrap(),
        "bench" => {
            let reps: usize = args[3].parse().unwrap();
            let workers = args[4].parse().unwrap();
            let arms: Vec<_> = args[5].split(',').collect();
            let reference = mibz2::decode_with::<Cells>(&bytes, Kernels::detect()).unwrap();
            println!("round,workers,arm,seconds,output");
            for round in 0..=reps {
                for j in 0..arms.len() {
                    let arm = arms[(round + j) % arms.len()];
                    let start = Instant::now();
                    let out = run(arm, &bytes, workers);
                    let elapsed = start.elapsed().as_secs_f64();
                    assert_eq!(out, reference);
                    println!("{round},{workers},{arm},{elapsed:.6},{}", out.len());
                }
            }
        }
        _ => panic!("census INPUT ORDERS | write INPUT ARM OUTPUT | bench INPUT REPS WORKERS ARMS"),
    }
}
