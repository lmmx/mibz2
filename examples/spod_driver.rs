//! Linux timing driver for bench/spod/bench.py. The same source builds on master.
use mibz2::{parallel::Plan, Kernels, StreamDecoder};
use std::time::Instant;

fn ticks() -> u64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap();
    let fields: Vec<_> = stat
        .rsplit_once(") ")
        .unwrap()
        .1
        .split_whitespace()
        .collect();
    // The suffix starts at field 3. utime and stime are fields 14 and 15.
    fields[11].parse::<u64>().unwrap() + fields[12].parse::<u64>().unwrap()
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let threads: usize = args[3].parse().unwrap();
    let prefix: usize = args[4].parse().unwrap();
    let slots = 4 * threads;
    let k = Kernels::detect();
    let start = Instant::now();
    let dec = match args[2].as_str() {
        "workers" => StreamDecoder::planned(Plan::Workers(threads), slots, k),
        "teams" => StreamDecoder::planned(
            Plan::Teams {
                teams: threads / 2,
                k: 2,
            },
            slots,
            k,
        ),
        #[cfg(feature = "spod")]
        "spod" => StreamDecoder::spod_workers(threads, slots, k),
        _ => panic!("unknown arm"),
    };
    let mut buf = vec![0; 1 << 16];
    let mut delivered = 0;
    let mut tail = None;
    // The bounded scanner reads ahead naturally. Nothing waits for the marker.
    dec.write(data).unwrap();
    dec.close();
    loop {
        let len = dec.read(&mut buf).unwrap();
        if len == 0 {
            break;
        }
        delivered += len;
        if prefix != 0 && delivered == prefix {
            tail = Some((Instant::now(), ticks()));
        }
    }
    dec.join();
    let finish = Instant::now();
    let (tail_ns, tail_ticks) = tail.map_or((0, 0), |(at, cpu)| {
        ((finish - at).as_nanos(), ticks() - cpu)
    });
    assert!(prefix == 0 || tail.is_some());
    let wall_ns = (finish - start).as_nanos();
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let rss_kib = status
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    #[cfg(feature = "spod")]
    let hits = dec.spod_blocks();
    #[cfg(not(feature = "spod"))]
    let hits = 0;
    println!("{wall_ns} {tail_ns} {tail_ticks} {rss_kib} {hits} {delivered}");
}
