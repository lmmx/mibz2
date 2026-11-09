//! The service core's starvation diagnostic (feature `svc-diag`, src/diag.rs; research): a
//! small decode at the svc2 plan (two teams of three, two service workers, a window of 8,
//! the admission threshold 2) logs every block through every stage once, in order, with the
//! ownership identities at every hub event, and gives the same bytes as the default decoder
//! and the input; a panic injected in a finish or a team's walk still leaves every buffer
//! accounted for. Run with `cargo test --features svc-diag --test svc_diag`.

mod common;
use common::*;
use mibz2::diag::{self, Kind};
use mibz2::parallel::ServiceFault;
use mibz2::{Error, Kernels, StreamDecoder};

/// A decoder's output to its end or first error, fed 100,003 bytes at a time from another
/// thread, and the decoder's log once it is dropped
fn run(dec: StreamDecoder, z: &[u8]) -> (Vec<u8>, Result<(), Error>, Vec<diag::ThreadLog>) {
    let id = dec.diag_id();
    let mut out = Vec::new();
    let res = std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(100_003) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 1 << 16];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break Ok(()),
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => {
                    dec.cancel();
                    break Err(e);
                }
            }
        }
    });
    drop(dec);
    (out, res, diag::take(id))
}

/// svc2: two teams of three, two service workers, a window of 8
fn svc2() -> StreamDecoder {
    StreamDecoder::service(2, 3, 2, 8, Kernels::detect())
}

/// Several megabytes at level 1 (100 kB blocks): enough blocks to fill the window often
fn stream() -> (Vec<u8>, Vec<u8>) {
    let mut data = Vec::new();
    for (_, d) in corpora(1) {
        data.extend_from_slice(&d);
    }
    let mut rng = Rng(0xd1a6);
    data.extend(rng.bytes(300_000));
    (compress(&data, 1), data)
}

#[test]
fn a_svc2_decode_is_logged_complete_and_its_output_unchanged() {
    let (z, data) = stream();
    let (want, want_res, _) = run(
        StreamDecoder::with_inverse::<mibz2::bwt::Segmented>(2, 4, Kernels::detect()),
        &z,
    );
    want_res.unwrap();
    assert!(want == data);
    let (got, res, logs) = run(svc2(), &z);
    res.unwrap();
    assert!(got == want, "the bytes");
    let s = diag::check(&logs).unwrap_or_else(|e| panic!("the log: {e}"));
    let blocks = block_starts(&z).len() as u64;
    assert!(blocks >= 20, "{blocks} blocks");
    assert_eq!(
        (s.teams, s.services, s.lookahead, s.slots, s.blocks),
        (2, 2, 2, 8, blocks)
    );
    // every thread logged: the scanner, the feeder, two service workers, two teams, and
    // the reader (this one)
    let mut names: Vec<&str> = logs.iter().map(|l| l.name.as_str()).collect();
    names.sort_unstable();
    for n in [
        "mibz2-0",
        "mibz2-1",
        "mibz2-scan",
        "mibz2-svc-0",
        "mibz2-svc-1",
        "mibz2-svc-feed",
    ] {
        assert!(names.contains(&n), "{n} in {names:?}");
    }
    // the TSV has a row an event
    let t = diag::tsv(&logs);
    assert_eq!(t.lines().count(), s.events + 1);
    // and the window never held more than W blocks
    for (_, e) in diag::merged(&logs) {
        if let Some(w) = e.win {
            assert!(w.issued - w.taken <= 8, "{e:?}");
        }
    }
}

#[test]
fn the_same_decode_without_a_log_is_another_decoders_business() {
    // a decoder other than the service core's logs nothing
    let (z, _) = stream();
    let dec = StreamDecoder::with_inverse::<mibz2::bwt::Segmented>(2, 4, Kernels::detect());
    assert_eq!(dec.diag_id(), 0);
    let (_, res, logs) = run(dec, &z);
    res.unwrap();
    assert!(logs.is_empty());
}

/// A panic injected in a finish, in a team's walk and in a preparation: the decode fails as
/// it does without the log, and every front and back set is accounted for at every hub
/// event, with no retire from no task, and the hub empty at its end
#[test]
fn a_fault_leaves_every_buffer_accounted_for() {
    let (z, _) = stream();
    for fault in [
        ServiceFault::Finish { place: 5 },
        ServiceFault::Team {
            place: 7,
            thread: 0,
        },
        ServiceFault::Prepare { place: 3 },
    ] {
        let dec = StreamDecoder::service_with_fault(2, 3, 2, 8, Kernels::detect(), Some(fault));
        let (_, res, logs) = run(dec, &z);
        assert!(res.is_err(), "{fault:?}");
        let m = diag::merged(&logs);
        let cfg = m.iter().find(|(_, e)| e.kind == Kind::Config).unwrap().1;
        let f = cfg.hub.unwrap().free_front;
        let mut last = None;
        for (_, e) in &m {
            assert_ne!(e.kind, Kind::RetireUnowned, "{fault:?}");
            if let Some(h) = e.hub {
                assert_eq!(h.free_front + h.a + h.p + h.u, f, "{fault:?} {e:?}");
                assert_eq!(h.free_back + h.u + h.q + h.c, 4, "{fault:?} {e:?}");
                last = Some(h);
            }
        }
        let h = last.unwrap();
        assert_eq!((h.a, h.c, h.u, h.live), (0, 0, 0, 0), "{fault:?} {h:?}");
        let retired = m
            .iter()
            .filter(|(_, e)| {
                matches!(
                    e.kind,
                    Kind::PrepRetire | Kind::FinishRetire | Kind::WalkRetire
                )
            })
            .count();
        let taken = m.iter().filter(|(_, e)| e.kind == Kind::SvcPrepare).count();
        assert_eq!(retired, taken, "{fault:?}: a retire for every block taken");
    }
}
