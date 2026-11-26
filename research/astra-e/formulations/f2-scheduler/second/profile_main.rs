use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[path = "candidate.rs"]
mod batched;

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
    let mut one=batched::Candidate::default();
    let mut graph=std::io::BufWriter::new(std::fs::File::create(std::env::args().nth(2).unwrap()).unwrap());
    use std::io::Write;
    writeln!(graph,"stream,node,parent,ranks,selectors,dict_bytes").unwrap();
    let (mut reference, mut want, mut out) = (Segmented::default(), Vec::new(), Vec::new());
    let k = Kernels::detect();
    let mut ordinal = 0;
    println!("stream,round,column_shift_ns,second_discover_ns,second_merge_ns,second_table_ns,second_emit_ns,total_ns");
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
        for round in 0..22 {
            let v=one.profile(&r,n,p,&mut out,round,index as usize,&mut graph);
            assert_eq!(out,want);
            if round!=0 {println!("{index},{round},{},{},{},{},{},{}",v[0],v[1],v[2],v[3],v[4],v[5]);}
        }
    }
}
