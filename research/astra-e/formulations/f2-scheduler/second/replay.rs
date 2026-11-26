use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[path="refill/candidate.rs"] mod refill;
#[path="batch/candidate.rs"] mod fixed;
#[path="scalar/candidate.rs"] mod scalar;
#[repr(C)]
struct Timespec {
    s: i64,
    ns: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, ts: *mut Timespec) -> i32;
}
#[inline(never)]
fn thread_cpu_ns() -> u64 {
    let mut ts = Timespec { s: 0, ns: 0 };
    // Linux/x86-64 CLOCK_THREAD_CPUTIME_ID; an exclusive stack timespec is writable.
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    ts.s as u64 * 1_000_000_000 + ts.ns as u64
}
fn main() {
    let input = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&input));
    let mut r = runs::Runs::default();
    let(mut r8,mut b8,mut serial)=(refill::Candidate::default(),fixed::Candidate::default(),scalar::Candidate::default());
    let (mut reference, mut want, mut out) = (Segmented::default(), Vec::new(), Vec::new());
    let k = Kernels::detect();
    let mut ordinal = 0;
    println!("stream,round,arm,discover_ns,complete_second_ns");
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let selected = ordinal % 24 == 0;
        ordinal += 1;
        // Compare the same admitted input, excluding the density-refused control.
        if !selected || r.records().len() * 20 >= 3 * n || r.records().len() * 64 < n {
            continue;
        }
        let p = symbols.orig_ptr;
        assert!(reference.invert_runs(&r, n, p, &mut want, k));
        let state0=r8.preload(&r,n,p);let state1=b8.preload(&r,n,p);let state2=serial.preload(&r,n,p);
        assert_eq!(state0,state1);assert_eq!(state0,state2);
        for round in 0..97 {
            for offset in 0..3 {
                let arm=(round+offset)%3;
                let v=match arm {0=>r8.replay(state0,n,&mut out),1=>b8.replay(state1,n,&mut out),_=>serial.replay(state2,n,&mut out)};
                assert_eq!(out,want);
                if round!=0{println!("{index},{round},{},{},{}",["refill","batch","scalar"][arm],v[0],v[1]);}
            }
        }
    }
}
