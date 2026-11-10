//! A parallel `BzReader` whose source's `read` panics: the reader's feeder thread unwinds,
//! and its guard cancels the decode, so the reader's read fails (an `io::Error` saying the
//! input's reader panicked) instead of waiting for ever for input that will never come.
//! At once, after a few bytes, half way and with blocks in flight, on each parallel reader
//! (production's workers, the plain teams, a pipelined team, the service core: explicitly,
//! and as `BzReader::parallel`'s default at 10 threads on its target, by its own dispatch),
//! all under a watchdog.

mod common;
use common::*;
use mibz2::parallel::{with_machine, Machine, Plan, PIPELINED_LEVERS};
use mibz2::{BzReader, Kernels};
use std::io::Read;
use std::time::Duration;

/// `f` on another thread, failing the test if it takes a minute (a hang)
fn watchdog<T: Send + 'static>(what: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(60))
        .unwrap_or_else(|_| panic!("{what}: hung (a minute)"))
}

/// A source whose `read` panics once `ok` bytes have been read
struct Panicky {
    data: Vec<u8>,
    at: usize,
    ok: usize,
}

impl Read for Panicky {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.at >= self.ok {
            panic!("a source's read panicking at byte {}", self.at);
        }
        let n = buf
            .len()
            .min(self.ok - self.at)
            .min(self.data.len() - self.at);
        buf[..n].copy_from_slice(&self.data[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

#[test]
fn a_panicking_source() {
    let corp = corpora(1);
    let z = compress(&corp[0].1, 9);
    // (more than the feeder's 4 MiB chunk, so a panic at 5 MB comes after its first write,
    // with blocks in flight)
    let big = compress(&corp[1].1.repeat(3), 1);
    assert!(big.len() > 5_000_000);
    let cases = [(&z, 0), (&z, 10), (&z, z.len() / 2), (&big, 5_000_000)];
    for (z, ok) in cases {
        for reader in [
            "workers",
            "teams",
            "pipelined",
            "service",
            "service default",
        ] {
            let z = z.clone();
            watchdog(&format!("panic at {ok}, {reader}"), move || {
                let src = Panicky { data: z, at: 0, ok };
                let k = Kernels::detect();
                let mut r = match reader {
                    "workers" => BzReader::parallel(src, 3),
                    "teams" => {
                        BzReader::parallel_planned(src, Plan::Teams { teams: 2, k: 2 }, 8, k)
                    }
                    "pipelined" => {
                        BzReader::parallel_pipelined_levers(src, 1, 3, 4, k, PIPELINED_LEVERS)
                    }
                    "service" => BzReader::parallel_service(src, 2, 3, 2, 8, k),
                    _ => {
                        let r =
                            with_machine(Machine::service_target(), || BzReader::parallel(src, 10));
                        let dec = r.parallel_decoder().unwrap();
                        let on_target = cfg!(all(target_os = "linux", target_arch = "x86_64"))
                            && k.avx2
                            && k.clmul;
                        assert_eq!(dec.is_service(), on_target, "{reader}");
                        r
                    }
                };
                let mut out = Vec::new();
                let e = r.read_to_end(&mut out).unwrap_err();
                assert!(e.to_string().contains("panicked"), "{reader}: {e}");
                drop(r);
            });
        }
    }
}
