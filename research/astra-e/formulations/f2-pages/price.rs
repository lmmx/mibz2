use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
use std::time::Instant;
#[allow(dead_code)]
mod baseline;
#[allow(dead_code)]
mod layout;

#[repr(C)]
struct Timespec { s: i64, ns: i64 }
unsafe extern "C" { fn clock_gettime(id: i32, ts: *mut Timespec) -> i32; }
#[inline(never)]
fn thread_cpu_ns() -> u64 {
    let mut ts = Timespec { s: 0, ns: 0 };
    // SAFETY: a valid aligned timespec for the current thread CPU clock on 64-bit Linux.
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    ts.s as u64 * 1_000_000_000 + ts.ns as u64
}

fn main() {
    let input = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mode = std::env::args().nth(2).unwrap();
    let mut scan = Scanner::new(SliceSource(&input));
    let (mut r, mut reference) = (runs::Runs::default(), Segmented::default());
    let (mut want, mut out) = (Vec::new(), Vec::new());
    let kernels = Kernels::detect();
    let mut base = baseline::Candidate::default();
    let mut p32 = layout::PageCandidate::<32>::default();
    let mut p40 = layout::PageCandidate::<40>::default();
    let mut p44 = layout::PageCandidate::<44>::default();
    let mut p64 = layout::PageCandidate::<64>::default();
    let mut ordinal = 0;
    let mut accepted = 0;
    if mode == "geometry" {
        println!("block,rows,capped_records,width,runs,boundary_starts,allocated_bytes");
        // Odd final pages, cuts on the page boundary, dense alternating starts, cap splits.
        for n in [1,31,32,39,40,43,44,63,64,65,127,128,900000] {
            for step in [1usize,2,31,32,39,40,43,44,63,64,127,505] {
                let mut caps = Vec::new();
                let mut left = n;
                let mut c = b'"';
                while left > 0 {
                    let take = left.min(step);
                    let mut remain = take;
                    while remain > 0 {
                        let part = remain.min(127);
                        caps.push((u16::from(c) << 8) | part as u16);
                        remain -= part;
                    }
                    c ^= 1;
                    left -= take;
                }
                p32.check_pages(&caps, n as u32);
                p40.check_pages(&caps, n as u32);
                p44.check_pages(&caps, n as u32);
                p64.check_pages(&caps, n as u32);
            }
        }
        // Start fresh workers so the dense synthetic data do not affect real capacity.
        p32 = Default::default();
        p40 = Default::default();
        p44 = Default::default();
        p64 = Default::default();
    } else {
        assert_eq!(mode, "price");
        println!("block,round,position,arm,prep_mark_ns,discover_ns,merge_ns,residual_output_ns,inverse_cpu_ns,wall_ns");
    }
    while let Event::Block { index, symbols, .. } = scan.next(Vec::new()).unwrap() {
        let n = runs::front(&symbols, &mut r, kernels).unwrap();
        if n < 65536 { continue; }
        let selected = ordinal % 24 == 0;
        ordinal += 1;
        if r.records().len() * 20 >= 3*n || r.records().len() * 64 < n { continue; }
        if mode == "price" && !selected { continue; }
        let p = symbols.orig_ptr;
        assert!(reference.invert_runs(&r, n, p, &mut want, kernels));
        accepted += 1;
        if mode == "geometry" {
            macro_rules! check { ($arm:expr, $width:expr) => {{
                let (count, cuts, bytes) = $arm.check_pages(r.records(),n as u32);
                assert!($arm.invert_runs(&r,n,p,&mut out,kernels));
                assert_eq!(out,want);
                println!("{index},{n},{},{},{count},{cuts},{bytes}", r.records().len(), $width);
            }}; }
            check!(p32,32); check!(p40,40); check!(p44,44); check!(p64,64);
            continue;
        }
        for pass in 0..=30usize {
            for pos in 0..5 {
                let rot = pass.saturating_sub(1) + accepted;
                let arm = (rot + if rot/5%2 == 0 {pos} else {4-pos}) % 5;
                let start = Instant::now();
                let t = match arm {
                    0 => base.measure(&r,n,p,&mut out,true),
                    1 => p32.measure(&r,n,p,&mut out,true),
                    2 => p40.measure(&r,n,p,&mut out,true),
                    3 => p44.measure(&r,n,p,&mut out,true),
                    _ => p64.measure(&r,n,p,&mut out,true),
                };
                let wall = start.elapsed().as_nanos();
                assert_eq!(out,want);
                if pass > 0 {
                    println!("{index},{pass},{pos},{},{},{},{},{},{},{wall}",
                        ["split64","packed32","packed40","packed44","wide64"][arm],
                        t[0], t[1], t[2], t[3], t.iter().sum::<u64>());
                }
            }
        }
    }
    assert_eq!(accepted, if mode=="geometry" {228} else {9});
}
