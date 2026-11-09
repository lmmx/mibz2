//! The general plan policy's plans (research, opt-in: `parallel::general`) cannot change
//! output or errors: every distinct plan and mode rule A gives on its grid (1-64 cores, an
//! L3 of 2-128 MiB, threads 1 to 2 × cores), forced through `general::decoder`, decodes the
//! test streams byte for byte as the default decoder does and as libbzip2 does, and fails on
//! corrupted, truncated and empty input with the same error, after the same bytes.

mod common;
use common::*;
use mibz2::parallel::general::{decoder, rule_a, Mode, Topology};
use mibz2::parallel::{Plan, SLOTS_PER_THREAD};
use mibz2::{Error, Kernels, StreamDecoder};
use std::collections::BTreeSet;

/// A decoder's output, fed 100,003 bytes at a time from another thread: the bytes read,
/// and the error that ended it if any
fn run(dec: StreamDecoder, z: &[u8]) -> (Vec<u8>, Option<Error>) {
    let mut out = Vec::new();
    let err = std::thread::scope(|s| {
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
                Ok(0) => break None,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => {
                    dec.cancel();
                    break Some(e);
                }
            }
        }
    });
    (out, err)
}

/// Every distinct (plan, mode) rule A gives on the journal's grid
fn rule_a_plans() -> BTreeSet<(usize, usize, bool)> {
    let mut set = BTreeSet::new();
    for phys in 6..=64usize {
        for m in 2..=128u64 {
            let t = Topology::synthetic(Some(phys), Some(m << 20), true, false);
            for n in 6..=2 * phys {
                let d = rule_a(&t, n);
                if let Plan::Teams { teams, k } = d.plan {
                    set.insert((teams, k, matches!(d.mode, Mode::Pipelined(_))));
                }
            }
        }
    }
    set
}

fn inputs() -> Vec<(String, Vec<u8>)> {
    let mut rng = Rng(0x6e6e_7261_6c41_0001);
    let s = samples(&mut rng);
    let mut v = Vec::new();
    // every sample, each at its own level, as one file of many streams (levels 1-9 mixed)
    let mut mixed = Vec::new();
    for (i, (_, data)) in s.iter().enumerate() {
        mixed.extend(compress(data, 1 + (i as u32 * 4) % 9));
    }
    v.push(("mixed levels".to_string(), mixed));
    // the text at levels 1, 3, 6 and 9, and a level-3-then-9 concatenation
    let text = &s[3].1;
    for level in [1, 3, 6, 9] {
        v.push((format!("text level {level}"), compress(text, level)));
    }
    let mut lo_hi = compress(&s[4].1, 3);
    lo_hi.extend(compress(text, 9));
    v.push(("level 3 then 9".into(), lo_hi));
    // corrupted: a block's stored CRC, a byte of its data; truncated; empty; not bz2
    let z = compress(text, 9);
    let starts = block_starts(&z);
    assert!(starts.len() >= 3);
    for (i, &b) in starts.iter().enumerate().take(3) {
        let mut t = z.clone();
        let bit = b + 48 + 5;
        t[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        v.push((format!("block {i}'s CRC"), t));
        let mut t = z.clone();
        t[(b / 8) as usize + 4000] ^= 0x5a;
        v.push((format!("block {i}'s data"), t));
    }
    v.push(("truncated at half".into(), z[..z.len() / 2].to_vec()));
    v.push(("truncated by 3".into(), z[..z.len() - 3].to_vec()));
    v.push(("empty".into(), Vec::new()));
    v.push(("not bz2".into(), b"hello, world\n".repeat(100)));
    let mut tail = z.clone();
    tail.extend(b"trailing");
    v.push(("bytes after the stream".into(), tail));
    v
}

#[test]
fn rule_a_plans_change_no_output_or_error() {
    let plans = rule_a_plans();
    // (2 × 2 up to 26 × 2 plain, 2 × 3 up to 21 × 3 pipelined, and nothing else)
    assert!(plans
        .iter()
        .all(|&(t, k, p)| t >= 1 && (k == 2 && !p || k == 3 && p)));
    println!("{} distinct plans: {plans:?}", plans.len());
    let inputs = inputs();
    let k = Kernels::detect();
    let want: Vec<_> = inputs
        .iter()
        .map(|(_, z)| run(StreamDecoder::new(4, 4 * SLOTS_PER_THREAD, k), z))
        .collect();
    for ((name, z), (out, err)) in inputs.iter().zip(&want) {
        match oracle(z) {
            Ok(o) => assert!(
                err.is_none() && o == *out,
                "{name}: the default against libbzip2"
            ),
            Err(_) => assert!(err.is_some(), "{name}: libbzip2 refuses it"),
        }
    }
    for &(teams, kk, piped) in &plans {
        let plan = Plan::Teams { teams, k: kk };
        let mode = if piped {
            Mode::Pipelined(mibz2::parallel::PIPELINED_LEVERS)
        } else {
            Mode::Plain
        };
        let threads = plan.threads();
        for ((name, z), (out, err)) in inputs.iter().zip(&want) {
            let dec = decoder(plan, mode, threads, SLOTS_PER_THREAD * threads, k);
            assert_eq!(dec.is_pipelined(), piped);
            let (o, e) = run(dec, z);
            assert_eq!(e, *err, "{plan:?} {mode:?} on {name}: the error");
            assert!(o == *out, "{plan:?} {mode:?} on {name}: the bytes");
        }
    }
}
