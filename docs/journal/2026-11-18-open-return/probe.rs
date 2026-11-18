use mibz2::bwt::{Cells, InverseBwt, LazyCells, Plain};
use mibz2::open_return::{
    switches::{Switch, SwitchKernel},
    Kernel, Open,
};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{Kernels, StreamDecoder};
use std::{env, fs, time::Instant};

fn stream<I: InverseBwt + 'static>(bytes: &[u8], workers: usize) -> Vec<u8> {
    if workers == 0 {
        return mibz2::decode_with::<I>(bytes, Kernels::detect()).unwrap();
    }
    let decoder = StreamDecoder::workers_with::<I>(workers, 4 * workers, Kernels::detect());
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for chunk in bytes.chunks(4 << 20) {
                decoder.write(chunk.to_vec()).unwrap();
            }
            decoder.close();
        });
        let mut buf = vec![0; 4 << 20];
        let mut out = Vec::new();
        loop {
            let n = decoder.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
        }
        out
    })
}

fn decode(arm: &str, data: &[u8], workers: usize) -> Vec<u8> {
    match arm {
        "cells" => stream::<Cells>(data, workers),
        "lazy" => stream::<LazyCells>(data, workers),
        "qi8" => stream::<Open<8, true, true>>(data, workers),
        "qi24" => stream::<Open<24, true, true>>(data, workers),
        "qi32" => stream::<Open<32, true, true>>(data, workers),
        "ranked" => stream::<Open<16, true, true, true>>(data, workers),
        "q16" => stream::<Open<16, false, true>>(data, workers),
        "qi16" => stream::<Open<16, true, true>>(data, workers),
        "i2" => stream::<Open<2, true>>(data, workers),
        "i4" => stream::<Open<4, true>>(data, workers),
        "i8" => stream::<Open<8, true>>(data, workers),
        "i16" => stream::<Open<16, true>>(data, workers),
        "i32" => stream::<Open<32, true>>(data, workers),
        "s8" => stream::<Switch<8>>(data, workers),
        "s16" => stream::<Switch<16>>(data, workers),
        "s32" => stream::<Switch<32>>(data, workers),
        "s64" => stream::<Switch<64>>(data, workers),
        "8" => stream::<Open<8>>(data, workers),
        "16" => stream::<Open<16>>(data, workers),
        "32" => stream::<Open<32>>(data, workers),
        "64" => stream::<Open<64>>(data, workers),
        _ => panic!("unknown arm {arm}"),
    }
}

fn census(data: &[u8], limits: &[u32], mode: &str) {
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let (mut l, mut oracle, mut out) = (Vec::new(), Vec::new(), Vec::new());
    let mut counts = [0; 256];
    let mut reference = Plain::default();
    let mut kernel = Kernel::default();
    let mut switches = SwitchKernel::default();
    let mut ordinal = 0;
    println!("ordinal,block,admitted,limit,n,r,retained,thick,bands,words,copies,traced,thick_area,unit_work,longest,table_bytes,thin_traced,thin_words,thin_area,open_nodes,steps,rules");
    while let Event::Block { index, symbols, .. } = sc.next(syms).unwrap() {
        mibz2::mtf::mtf(&symbols, &mut l, &mut counts, Kernels::detect()).unwrap();
        reference.invert(&l, &counts, symbols.orig_ptr, &mut oracle);
        for &limit in limits {
            let (s, work, words, area) = if mode == "switches" {
                let q = switches.column(&l, symbols.orig_ptr, limit, &mut out);
                (q.common, q.thin_traced, q.thin_words, q.thin_area)
            } else if mode == "induced" {
                (
                    kernel.column::<true>(&l, symbols.orig_ptr, limit, &mut out),
                    0,
                    0,
                    0,
                )
            } else {
                (
                    kernel.column::<false>(&l, symbols.orig_ptr, limit, &mut out),
                    0,
                    0,
                    0,
                )
            };
            assert_eq!(out, oracle, "block {ordinal}");
            println!("{ordinal},{index},{},{limit},{},{},{},{},{},{},{},{},{},{},{},{},{work},{words},{area},{},{},{}",
                u8::from(mibz2::runs::takes_runs(&symbols)), s.n, s.runs, s.retained, s.thick,
                s.bands, s.words, s.copies, s.traced, s.thick_area, s.unit_work, s.longest, s.table_bytes, s.open_nodes, s.steps, s.rules);
        }
        ordinal += 1;
        syms = symbols.syms;
    }
}

fn small() {
    let mut reference = Plain::default();
    let mut kernel = Kernel::default();
    let (mut baseline, mut actual) = (Vec::new(), Vec::new());
    let mut switches = SwitchKernel::default();
    let mut cases = 0;
    for (base, last) in [(2u32, 10u32), (3, 7)] {
        for n in 1..=last {
            for mut num in 0..base.pow(n) {
                let l: Vec<_> = (0..n)
                    .map(|_| {
                        let v = (num % base) as u8;
                        num /= base;
                        v
                    })
                    .collect();
                let mut counts = [0; 256];
                for &c in &l {
                    counts[usize::from(c)] += 1;
                }
                for p in 0..n {
                    reference.invert(&l, &counts, p, &mut baseline);
                    for limit in [0, 1, 2, 4, 16] {
                        kernel.column::<false>(&l, p, limit, &mut actual);
                        assert_eq!(baseline, actual);
                        kernel.column::<true>(&l, p, limit, &mut actual);
                        assert_eq!(baseline, actual, "{l:?} p={p} limit={limit}");
                        switches.column(&l, p, limit, &mut actual);
                        assert_eq!(baseline, actual, "switch {l:?} p={p} limit={limit}");
                        cases += 1;
                    }
                }
            }
        }
    }
    println!("{cases} marked column/limit comparisons");
}

fn price(data: &[u8], rounds: usize) {
    let k = Kernels::detect();
    let mut scanner = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let mut ordinal = 0;
    let (mut cells, mut lazy) = (Cells::default(), LazyCells::default());
    let (mut direct, mut endpoint, mut switches) = (
        Kernel::default(),
        Kernel::default(),
        SwitchKernel::default(),
    );
    let mut runs = mibz2::runs::Runs::default();
    let (mut reference, mut out) = (Vec::new(), Vec::new());
    println!("block,round,arm,build_seconds,write_seconds");
    while let Event::Block { symbols, .. } = scanner.next(syms).unwrap() {
        if mibz2::runs::takes_runs(&symbols) {
            let n = mibz2::runs::front(&symbols, &mut runs, k).unwrap();
            let p = symbols.orig_ptr;
            assert!(cells.invert_runs(&runs, n, p, &mut reference, k));
            for lap in 0..=rounds {
                let arms = [
                    "cells", "lazy", "16", "i4", "i8", "i16", "s16", "q16", "qi16",
                ];
                for j in 0..arms.len() {
                    let arm = arms[(j + lap + ordinal) % arms.len()];
                    let (build, write) = match arm {
                        "q16" => direct.price::<false, true>(&runs, n, p, 16, &mut out),
                        "qi16" => endpoint.price::<true, true>(&runs, n, p, 16, &mut out),
                        "16" => direct.price::<false, false>(&runs, n, p, 16, &mut out),
                        "i4" => endpoint.price::<true, false>(&runs, n, p, 4, &mut out),
                        "i8" => endpoint.price::<true, false>(&runs, n, p, 8, &mut out),
                        "i16" => endpoint.price::<true, false>(&runs, n, p, 16, &mut out),
                        "s16" => switches.price(&runs, n, p, 16, &mut out),
                        _ => {
                            let start = Instant::now();
                            if arm == "cells" {
                                assert!(cells.invert_runs(&runs, n, p, &mut out, k));
                            } else {
                                assert!(lazy.invert_runs(&runs, n, p, &mut out, k));
                            }
                            (start.elapsed().as_secs_f64(), 0.0)
                        }
                    };
                    assert_eq!(reference, out);
                    println!("{ordinal},{lap},{arm},{build:.9},{write:.9}");
                }
            }
        }
        ordinal += 1;
        syms = symbols.syms;
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args[1] == "small" {
        small();
        return;
    }
    let data = fs::read(&args[2]).unwrap();
    match args[1].as_str() {
        "census" | "switches" | "induced" => census(
            &data,
            &args[3]
                .split(',')
                .map(|s| s.parse().unwrap())
                .collect::<Vec<_>>(),
            &args[1],
        ),
        "dump" => {
            let mut scanner = Scanner::new(SliceSource(&data));
            let Event::Block { symbols, .. } = scanner.next(Vec::new()).unwrap() else {
                panic!("empty")
            };
            let mut l = Vec::new();
            mibz2::mtf::mtf(&symbols, &mut l, &mut [0; 256], Kernels::detect()).unwrap();
            println!("{}", symbols.orig_ptr);
            fs::write(&args[3], l).unwrap();
        }
        "price" => price(&data, args[3].parse().unwrap()),
        "write" => fs::write(&args[4], decode(&args[3], &data, 0)).unwrap(),
        "bench" => {
            let arms: Vec<_> = args[3].split(',').collect();
            let rounds: usize = args[4].parse().unwrap();
            let workers: usize = args[5].parse().unwrap();
            let repeats: usize = args[6].parse().unwrap();
            let expected = stream::<Cells>(&data, 0);
            println!("round,arm,workers,repeats,seconds,bytes");
            for lap in 0..=rounds {
                for j in 0..arms.len() {
                    let arm = arms[(j + lap) % arms.len()];
                    let mut seconds = 0.0;
                    for _ in 0..repeats {
                        let start = Instant::now();
                        let output = decode(arm, &data, workers);
                        seconds += start.elapsed().as_secs_f64();
                        assert_eq!(output, expected);
                    }
                    println!(
                        "{lap},{arm},{workers},{repeats},{:.9},{}",
                        seconds / repeats as f64,
                        expected.len()
                    );
                }
            }
        }
        _ => panic!("use small, census, write, or bench"),
    }
}
