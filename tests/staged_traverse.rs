//! The staged cooperative walk (research, `coop_research::set_staged_traverse`) end to end:
//! with it selected, every team decoder (the plain teams of 2, 3 and 4, `bwt::Cooperative`,
//! and the pipelined teams with park and small, the plan the default runs on the ten-core
//! topology) gives libbzip2's bytes on the crate's corpora and the pipeline's multi-block
//! inputs. In its own test binary, since the selector is global: the other test binaries
//! keep the default, unstaged walk.

mod common;
use common::*;
use mibz2::parallel::{Levers, Plan};
use mibz2::{Kernels, StreamDecoder};
use std::io::Read;

fn decode(dec: StreamDecoder, z: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    std::thread::scope(|s| {
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
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => panic!("{e}"),
            }
        }
    });
    out
}

#[test]
fn staged_teams_give_libbzip2s_bytes() {
    mibz2::coop_research::set_staged_traverse(true);
    assert!(mibz2::coop_research::staged_traverse());
    let mut rng = Rng(0x5747_4745_4400_0001);
    let mut inputs: Vec<(String, Vec<u8>)> = corpora(1)
        .into_iter()
        .map(|(n, d)| (n.to_string(), compress(&d, 9)))
        .collect();
    inputs.extend(pipeline_corpus(&mut rng));
    let k = Kernels::detect();
    let both = Levers {
        park: true,
        small: true,
        spin2k: false,
    };
    for (name, z) in &inputs {
        let want = oracle(z).unwrap();
        for team in [2, 3, 4] {
            let plan = Plan::Teams { teams: 2, k: team };
            let got = decode(StreamDecoder::planned(plan, 4, k), z);
            assert!(got == want, "{name}: teams of {team}, staged");
        }
        for team in [2, 3] {
            let got = decode(StreamDecoder::pipelined_levers(3, team, 12, k, both), z);
            assert!(
                got == want,
                "{name}: pipelined teams of {team}, park and small, staged"
            );
        }
        let mut r =
            mibz2::BzReader::parallel_pipelined(std::io::Cursor::new(z.clone()), 2, 3, 6, k);
        let mut got = Vec::new();
        r.read_to_end(&mut got).unwrap();
        assert!(got == want, "{name}: pipelined reader, staged");
    }
}
