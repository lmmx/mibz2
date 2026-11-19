use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[allow(dead_code)]
mod compact;
#[allow(dead_code)]
mod cut;
#[allow(dead_code)]
mod known;
#[allow(dead_code)]
mod refill;
#[repr(C)]
struct Ts {
    i: i64,
    n: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, ts: *mut Ts) -> i32;
}
#[inline(never)]
fn now() -> u64 {
    let mut ts = Ts { i: 0, n: 0 };
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    (ts.i * 1_000_000_000 + ts.n) as u64
}
fn cap<T>(v: &Vec<T>) -> usize {
    v.capacity() * std::mem::size_of::<T>()
}

fn small() {
    let mut subject = cut::Subject::default();
    let mut total = 0usize;
    let mut refuse = 0usize;
    for n in 1..=7 {
        for mut code in 0..3usize.pow(n as u32) {
            let mut ll = Vec::new();
            let mut freq = [0; 256];
            for _ in 0..n {
                let b = 33 + (code % 3) as u8;
                code /= 3;
                ll.push(b);
                freq[b as usize] += 1;
            }
            let caps: Vec<_> = ll.iter().map(|&b| (u16::from(b) << 8) + 1).collect();
            let mut psi: Vec<_> = (0..n).collect();
            psi.sort_by_key(|&i| ll[i]);
            for p in 0..n {
                let mut x = p;
                let want: Vec<_> = (0..n)
                    .map(|_| {
                        x = psi[x];
                        ll[x]
                    })
                    .collect();
                for &(threshold, lanes) in &[(1, 1), (2, 8), (4, 8), (8, 8)] {
                    let took = subject
                        .measure_raw(&caps, &freq, n as u32, p as u32, threshold, lanes)
                        .is_some();
                    assert_eq!(took, want.contains(&34));
                    if took {
                        subject.verify(&ll, &want);
                        total += 1;
                    } else {
                        refuse += 1;
                    }
                }
            }
        }
    }
    println!("all small marks: {total} native byte/arc checks, {refuse} disjoint refusals");
}
fn rss(arm: &str, data: &[u8]) {
    let mut scan = Scanner::new(SliceSource(data));
    let mut r = runs::Runs::default();
    let k = Kernels::detect();
    let mut a = refill::Candidate::default();
    let mut b = known::Candidate::default();
    let mut c = compact::Dispatch::<2, 2>::default();
    let mut cut = cut::Subject::default();
    let mut checked = 0;
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 || r.records().len() * 20 >= 3 * n || r.records().len() * 64 < n {
            continue;
        }
        let p = symbols.orig_ptr;
        let used = match arm {
            "refill" => {
                a.control(&r, n as u32, p).unwrap();
                a.stage_memory()
            }
            "known" => {
                b.control(&r, n as u32, p).unwrap();
                b.stage_memory()
            }
            "compact" => {
                c.control(&r, n as u32, p).unwrap();
                c.stage_memory()
            }
            _ => {
                let threshold = if arm == "cut1x8" { 1 } else { 8 };
                let lanes = if arm == "cut8x1" { 1 } else { 8 };
                cut.measure(&r, n as u32, p, threshold, lanes).unwrap();
                cut.memory().iter().sum()
            }
        };
        let status = std::fs::read_to_string("/proc/self/status").unwrap();
        let hwm = status
            .lines()
            .find(|s| s.starts_with("VmHWM:"))
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap();
        let resident = status
            .lines()
            .find(|s| s.starts_with("VmRSS:"))
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap();
        println!("{index},{n},{arm},{used},{resident},{hwm}");
        checked += 1;
    }
    assert_eq!(checked, 228);
}
fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap();
    if mode == "small" {
        small();
        return;
    }
    let data = std::fs::read(args.next().unwrap()).unwrap();
    if mode == "rss" {
        rss(&args.next().unwrap(), &data);
        return;
    }
    let mut scan = Scanner::new(SliceSource(&data));
    let mut r = runs::Runs::default();
    let k = Kernels::detect();
    let mut ref_bwt = Segmented::default();
    let mut expected = Vec::new();
    let mut r0 = refill::Candidate::default();
    let mut r1 = known::Candidate::default();
    let mut r2 = compact::Dispatch::<2, 2>::default();
    let mut one = cut::Subject::default();
    let mut eight = cut::Subject::default();
    let mut scalar = cut::Subject::default();
    let mut ordinal = 0;
    let mut admitted = 0;
    let mut checked = 0;
    const ARMS: [&str; 6] = ["refill", "known", "compact", "cut1x8", "cut8x8", "cut8x1"];
    if mode == "phase" {
        println!("stream,round,arm,prepare_mark_ns,discovery_ns,root_merge_ns,publish_ns,psi_build_ns,group_stage_ns,total_ns");
    }
    if mode == "geometry" {
        println!("stream,rows,arm,original_runs,psi_cells,seed_groups,source_gaps,cold_rows,terminal_spans,staged_bytes,hot_dict,shared_capacity,index_capacity,endpoint_capacity,stage_capacity,lane_capacity,total_capacity");
    }
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let selected = ordinal % 24 == 0;
        ordinal += 1;
        if r.records().len() * 20 >= 3 * n || r.records().len() * 64 < n {
            continue;
        }
        if mode != "geometry" && !selected {
            continue;
        }
        let p = symbols.orig_ptr;
        admitted += 1;
        checked += n;
        assert!(ref_bwt.invert_runs(&r, n, p, &mut expected, k));
        let mut ll = Vec::with_capacity(n);
        for &v in r.records() {
            ll.resize(ll.len() + (v & 255) as usize, (v >> 8) as u8);
        }
        let rounds = if mode == "phase" { 36 } else { 0 };
        for round in 0..=rounds {
            for offset in 0..6 {
                let a = (round + offset) % 6;
                let times = match a {
                    0 => r0.control(&r, n as u32, p),
                    1 => r1.control(&r, n as u32, p),
                    2 => r2.control(&r, n as u32, p),
                    3 => one.measure(&r, n as u32, p, 1, 8),
                    4 => eight.measure(&r, n as u32, p, 8, 8),
                    _ => scalar.measure(&r, n as u32, p, 8, 1),
                }
                .unwrap();
                if round == 0 && a >= 3 {
                    let sub = match a {
                        3 => &one,
                        4 => &eight,
                        _ => &scalar,
                    };
                    sub.verify(&ll, &expected);
                    if mode == "geometry" {
                        let g = sub.geometry();
                        let mem = sub.memory();
                        print!("{index},{n},{}", ARMS[a]);
                        for item in g.into_iter().chain(mem) {
                            print!(",{item}");
                        }
                        println!(",{}", mem.iter().sum::<usize>());
                    }
                }
                if mode == "geometry" && a < 3 {
                    let size = match a {
                        0 => r0.stage_memory(),
                        1 => r1.stage_memory(),
                        _ => r2.stage_memory(),
                    };
                    println!(
                        "{index},{n},{},0,0,0,0,0,0,0,0,{size},0,0,0,0,{size}",
                        ARMS[a]
                    );
                }
                if mode == "phase" && round > 0 {
                    print!("{index},{round},{}", ARMS[a]);
                    for t in times {
                        print!(",{t}");
                    }
                    println!(",{}", times.iter().sum::<u64>());
                }
            }
        }
    }
    assert_eq!(admitted, if mode == "geometry" { 228 } else { 9 });
    eprintln!("{admitted} admissions, {checked} raw bytes checked for each cold arm");
}
