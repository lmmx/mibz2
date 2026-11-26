use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
mod sparse;
pub use sparse::Table;
use sparse::{oracle_write, paid, Plan};
#[allow(dead_code)]
mod original {
    include!(env!("EDGE_TABLE_SOURCE"));
}

#[inline(never)]
fn client(op: usize, arg: usize) {
    let request = [0x43540000 + op, arg, 0, 0, 0, 0];
    unsafe {
        std::arch::asm!("rol rdi,3","rol rdi,13","rol rdi,61","rol rdi,51","xchg rbx,rbx",
        in("rax") request.as_ptr(),inout("rdx") 0usize=>_,inout("rdi") 0usize=>_,options(nostack));
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() else {
        panic!("frozen block required")
    };
    let mut r = runs::Runs::default();
    let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
    let mut original = original::Kernel::default();
    let t = original.table(r.records(), n, symbols.orig_ptr);
    assert_eq!((t.edges.len(), t.mark), (89874, 35272));
    let or = t.oracle(7);
    let mut plan = Plan::default();
    let mut out = Vec::new();
    let mut gold = Vec::new();
    original::Kernel::stock(&t, &mut gold);
    let mut run = || match args[2].as_str() {
        "stock" => original::Kernel::stock(&t, &mut out),
        "oracle8s128" => oracle_write::<8>(&or, &t, &mut out),
        "paid8s128" => paid::<8, 7>(&mut plan, &t, &mut out),
        _ => panic!("unrecognized arm"),
    };
    run();
    client(4, 0);
    client(1, 0);
    run();
    client(3, c"emitter".as_ptr() as usize);
    client(5, 0);
    assert!(out == gold);
}
