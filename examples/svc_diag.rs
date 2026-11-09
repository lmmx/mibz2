//! The service core's starvation diagnostic, run (feature `svc-diag`; research, untimed:
//! nothing it prints is timing evidence). Decodes a .bz2 with the service core at the plan
//! given (by default svc2: two teams of three, two service workers, a window of 8 and the
//! admission threshold 2), checks its bytes against the default decoder's, checks its log
//! complete (`diag::check`), and writes the log as TSV for bench/svc_diag.py.
//!
//! Usage: svc_diag IN.bz2 OUT.tsv [TEAMS K SERVICES SLOTS [ADMISSION]]

use mibz2::{diag, Kernels, StreamDecoder};

/// The decoder's bytes, fed 4 MB at a time (as `BzReader` feeds it) from another thread
fn decode(dec: &StreamDecoder, z: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(4 << 20) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 1 << 20];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => panic!("the decode failed: {e:?}"),
            }
        }
    });
    out
}

/// FNV-1a, 64 bits: the output's identity in the report (not a security digest)
fn fnv(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325, |h, &x| {
        (h ^ u64::from(x)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let z = std::fs::read(&a[0]).expect("the input");
    let n = |i: usize, d: usize| a.get(i).map_or(d, |v| v.parse().expect("a number"));
    let (teams, k, services, slots) = (n(2, 2), n(3, 3), n(4, 2), n(5, 8));
    let admission = a.get(6).map(|v| v.parse::<usize>().expect("a number"));
    let k_ = Kernels::detect();
    let want = decode(
        &StreamDecoder::with_inverse::<mibz2::bwt::Segmented>(4, 16, k_),
        &z,
    );
    let dec = match admission {
        None => StreamDecoder::service(teams, k, services, slots, k_),
        Some(l) => StreamDecoder::service_admission(teams, k, services, slots, l, k_),
    };
    let id = dec.diag_id();
    let got = decode(&dec, &z);
    drop(dec);
    let logs = diag::take(id);
    let identical = got == want;
    let summary = diag::check(&logs);
    std::fs::write(&a[1], diag::tsv(&logs)).expect("the log written");
    println!(
        "plan={teams}x{k}+{services} slots={slots} admission={} bytes={} fnv={:016x} \
         identical={identical} check={}",
        admission.map_or("teams".into(), |l| l.to_string()),
        got.len(),
        fnv(&got),
        match &summary {
            Ok(s) => format!("ok blocks={} events={}", s.blocks, s.events),
            Err(e) => format!("FAILED {e}"),
        }
    );
    if !identical || summary.is_err() {
        std::process::exit(1);
    }
}
