use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[allow(dead_code)]
mod compact;
#[allow(dead_code)]
mod cut;
#[repr(C)]
struct Ts {
    s: i64,
    ns: i64,
}
unsafe extern "C" {
    fn clock_gettime(clock: i32, ts: *mut Ts) -> i32;
}
#[inline(never)]
fn now() -> u64 {
    let mut ts = Ts { s: 0, ns: 0 };
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    (ts.s * 1_000_000_000 + ts.ns) as u64
}
fn cap<T>(v: &Vec<T>) -> usize {
    v.capacity() * std::mem::size_of::<T>()
}
fn main() {
    let input = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan = Scanner::new(SliceSource(&input));
    let (mut fast, mut first) = (
        compact::Dispatch::<2, 2>::default(),
        compact::Dispatch::<2, 2>::default(),
    );
    let (mut one, mut eight) = (cut::Subject::default(), cut::Subject::default());
    let mut r = runs::Runs::default();
    let k = Kernels::detect();
    let mut refb = Segmented::default();
    let mut want = Vec::new();
    let mut out = Vec::new();
    let mut ordinal = 0;
    let mut blocks = 0;
    println!("stream,round,arm,cpu_ns");
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let select = ordinal % 24 == 0;
        ordinal += 1;
        if !select || r.records().len() * 20 >= 3 * n || r.records().len() * 64 < n {
            continue;
        }
        blocks += 1;
        assert!(refb.invert_runs(&r, n, symbols.orig_ptr, &mut want, k));
        let p = symbols.orig_ptr;
        let n = n as u32;
        for round in 0..=40 {
            for j in 0..4 {
                let a = (round + j) % 4;
                let ns = match a {
                    0 => {
                        let t = fast.complete(&r, n, p, &mut out);
                        assert_eq!(out, want);
                        t
                    }
                    1 => first.control(&r, n, p).unwrap().iter().sum(),
                    2 => one.measure(&r, n, p, 1, 8).unwrap().iter().sum(),
                    _ => eight.measure(&r, n, p, 8, 8).unwrap().iter().sum(),
                };
                if round != 0 {
                    println!(
                        "{index},{round},{},{ns}",
                        ["complete", "first", "cut1", "cut8"][a]
                    );
                }
            }
        }
    }
    assert_eq!(blocks, 9);
}
