use mibz2::stream::{Event, Scanner, SliceSource};
#[repr(C)]
struct Timespec {
    seconds: i64,
    nanos: i64,
}
unsafe extern "C" {
    fn clock_gettime(clock: i32, time: *mut Timespec) -> i32;
}
fn cpu() -> u64 {
    let mut t = Timespec {
        seconds: 0,
        nanos: 0,
    };
    assert_eq!(unsafe { clock_gettime(3, &mut t) }, 0);
    t.seconds as u64 * 1_000_000_000 + t.nanos as u64
}
fn lap<const NARROW: bool>(data: &[u8]) -> (usize, usize, u64) {
    let t = cpu();
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let (mut count, mut bytes) = (0, 0);
    while let Event::Block { symbols, .. } = sc.next_width::<NARROW>(syms).unwrap() {
        count += 1;
        bytes += symbols.syms.len() * 2;
        syms = std::hint::black_box(symbols.syms);
    }
    (count, bytes, cpu() - t)
}
fn main() {
    let data = std::fs::read(std::env::args_os().nth(1).unwrap()).unwrap();
    println!("round,position,packed,blocks,transport_bytes,cpu_ns");
    for r in 0..13 {
        for p in 0..2 {
            let packed = (r + p) % 2 == 1;
            let (n, b, ns) = if packed {
                lap::<true>(&data)
            } else {
                lap::<false>(&data)
            };
            println!("{r},{p},{packed},{n},{b},{ns}");
        }
    }
}
