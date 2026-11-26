pub use mibz2::{bwt, runs, Kernels};
use mibz2::{trace, BzReader};
use std::io::{Cursor, Read};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let compressed = std::fs::read(&args[1]).unwrap().repeat(4);
    let workers: usize = args[2].parse().unwrap();
    trace::set_level(1);
    let mut reader = BzReader::parallel_workers_with::<edge::Candidate>(
        Cursor::new(compressed), workers, 4 * workers, Kernels::detect(),
    );
    let mut buf = vec![0; 4 << 20];
    let mut total = 0u64;
    loop {
        let n = reader.read(&mut buf).unwrap();
        if n == 0 { break; }
        total += n as u64;
    }
    drop(reader);
    assert_eq!(total, 838800784);
    println!("thread,kind,place,ns,len");
    for thread in trace::take() {
        for m in thread.marks {
            use trace::Kind::*;
            if matches!(m.kind, SlotGot | Issued | JobGot | DecodeStart | MtfEnd |
                                DecodeEnd | Ready | Taken | ThreadStart | ThreadEnd) {
                println!("{},{:?},{},{},{}",thread.name,m.kind,m.place,m.ns,m.len);
            }
        }
    }
}
