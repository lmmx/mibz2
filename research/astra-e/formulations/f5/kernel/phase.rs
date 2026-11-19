use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
use std::io::Write;
use std::time::Instant;

#[allow(dead_code)]
mod direct;
#[allow(dead_code)]
mod old;

#[repr(C)]
struct Timespec {
    sec: i64,
    nano: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, ts: *mut Timespec) -> i32;
}
fn cpu() -> u64 {
    let mut ts = Timespec { sec: 0, nano: 0 };
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    ts.sec as u64 * 1_000_000_000 + ts.nano as u64
}

fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut r = runs::Runs::default();
    let k = Kernels::detect();
    let mut old = old::Kernel::default();
    let mut direct = direct::Kernel::default();
    let mut book = direct::Book::default();
    let (mut out, mut raw, mut old_out) = (Vec::new(), Vec::new(), Vec::new());
    let mut large = 0;
    let mut verified = 0;
    let mut phase_file =
        std::io::BufWriter::new(std::fs::File::create(std::env::args().nth(2).unwrap()).unwrap());
    writeln!(
        phase_file,
        "stream,round,arm,wall_ns,cpu_ns,construction_ns,book_ns,walk_ns,unrle_ns,crc_ns"
    )
    .unwrap();
    println!("stream,rows,primary,action_count,action_bytes,cache_bytes,cache_capacity,action_capacity,decoded_bytes");
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        if !runs::takes_runs(&symbols) {
            continue;
        }
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if n < 65_536 {
            continue;
        }
        let numbered = large;
        large += 1;
        if r.records().len() * 20 >= 3 * n {
            continue;
        }
        let p = symbols.orig_ptr;
        old.run::<true, true, false>(r.records(), n, p, 16, &mut raw);
        old_out.clear();
        mibz2::rle::unrle(&raw, &mut old_out, k).unwrap();
        assert_eq!(mibz2::crc::crc32(&old_out, k), symbols.crc);
        out.clear();
        direct
            .expanded(r.records(), n, p, &mut book, &mut out)
            .unwrap();
        assert_eq!(out, old_out);
        verified += n;
        let z = book.sizes();
        println!(
            "{index},{n},{p},{},{},{},{},{},{}",
            z[0],
            z[1],
            z[2],
            z[3],
            z[4],
            out.len()
        );
        if numbered % 24 != 0 {
            continue;
        }
        for round in 0..=20 {
            for offset in 0..2 {
                let mut phases = [0; 5];
                let arm = (round + offset) % 2;
                let w = Instant::now();
                let t = cpu();
                out.clear();
                if arm == 0 {
                    phases = direct
                        .observed(r.records(), n, p, &mut book, &mut out)
                        .unwrap();
                } else {
                    let t0 = cpu();
                    old.run::<true, true, false>(r.records(), n, p, 16, &mut raw);
                    let t1 = cpu();
                    mibz2::rle::unrle(&raw, &mut out, k).unwrap();
                    phases[2] = t1 - t0;
                    phases[3] = cpu() - t1;
                }
                let before_crc = cpu();
                let checksum = mibz2::crc::crc32(&out, k);
                let after = cpu();
                let elapsed = w.elapsed().as_nanos();
                phases[4] = after - before_crc;
                assert_eq!(checksum, symbols.crc);
                assert_eq!(out, old_out);
                if round > 0 {
                    writeln!(
                        phase_file,
                        "{index},{round},{},{elapsed},{},{},{},{},{},{}",
                        ["direct", "opend"][arm],
                        after - t,
                        phases[0],
                        phases[1],
                        phases[2],
                        phases[3],
                        phases[4]
                    )
                    .unwrap();
                }
            }
        }
    }
    assert_eq!(large, 233);
    assert_eq!(verified, 205197794);
}
