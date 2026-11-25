//! (research only, `ideal-*`) Byte-identity, store deduplication, work-removal counters and
//! activity-ledger checks for the four ideal-parts oracles:
//! - C (`ideal-construction`, target T2)
//! - M (`ideal-mtf`, target T3)
//! - R (`ideal-rle1`, target T6)
//! - D (`ideal-delivery`, reader mode)
//!
//! Tested at 1, 4 and 8 workers on the `ab gen corpora` set (`common::corpora(1)`, level 9)
//! and multi-stream repeated inputs; plus an env-driven (`MIBZ2_WIKIDATA`) test on
//! `wikidata.bz2` and `wikidata x4` (`bench/wikidata.py` from `bench-data`).
#![cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
    feature = "ideal-delivery",
))]

mod common;

use common::{compress, corpora};
use mibz2::{BzReader, Kernels};
use std::io::Read;
use std::sync::Mutex;

/// Oracle modes are process-global; serialise tests that switch them.
static LOCK: Mutex<()> = Mutex::new(());

const WORKERS: [usize; 3] = [1, 4, 8];

#[cfg(feature = "ideal-delivery")]
fn decode_workers(z: &'static [u8], workers: usize) -> Vec<u8> {
    let mut r = BzReader::parallel_workers(
        z,
        workers,
        mibz2::parallel::SLOTS_PER_THREAD * workers,
        Kernels::detect(),
        true,
    );
    let mut out = Vec::new();
    r.read_to_end(&mut out).expect("decode");
    drop(r);
    out
}

fn decode_workers_sha256(z: &'static [u8], workers: usize) -> (u64, [u8; 32]) {
    let mut r = BzReader::parallel_workers(
        z,
        workers,
        mibz2::parallel::SLOTS_PER_THREAD * workers,
        Kernels::detect(),
        true,
    );
    let mut buf = vec![0u8; 256 * 1024];
    let mut h = Sha256::new();
    let mut len = 0u64;
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                len += n as u64;
                h.update(&buf[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("decode failed: {e}"),
        }
    }
    drop(r);
    (len, h.finish())
}

struct CorpusInput {
    name: String,
    z: &'static [u8],
    len: u64,
    sha: [u8; 32],
    blocks: u64,
}

fn count_blocks(z: &[u8]) -> u64 {
    let mut n = 0u64;
    for r in mibz2::scan(z) {
        r.expect("scan");
        n += 1;
    }
    n
}

fn ab_corpora_inputs() -> Vec<CorpusInput> {
    let sets = corpora(1);
    let mut inputs = Vec::new();
    for (name, data) in &sets {
        let z: &'static [u8] = compress(data, 9).leak();
        let blocks = count_blocks(z);
        inputs.push(CorpusInput {
            name: (*name).to_string(),
            z,
            len: data.len() as u64,
            sha: sha256(data),
            blocks,
        });
    }
    // Repeated streams (3x runs): admitted by OpenCells, testing cross-stream indexing and store sharing.
    let runs_data = &sets.iter().find(|(n, _)| *n == "runs").unwrap().1;
    let one = compress(runs_data, 9);
    let mut multi = Vec::with_capacity(one.len() * 3);
    let mut multi_data = Vec::with_capacity(runs_data.len() * 3);
    for _ in 0..3 {
        multi.extend_from_slice(&one);
        multi_data.extend_from_slice(runs_data);
    }
    let multi_z: &'static [u8] = multi.leak();
    let blocks = count_blocks(multi_z);
    inputs.push(CorpusInput {
        name: "runs x3".to_string(),
        z: multi_z,
        len: multi_data.len() as u64,
        sha: sha256(&multi_data),
        blocks,
    });
    inputs
}

fn hex(d: [u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finish()
}

/// Reset every compiled ideal oracle to Real so tests never bleed state into each other.
fn reset_all_modes() {
    #[cfg(feature = "ideal-construction")]
    mibz2::ideal::construction::set_mode(mibz2::ideal::Mode::Real);
    #[cfg(feature = "ideal-mtf")]
    mibz2::ideal::mtf::set_mode(mibz2::ideal::Mode::Real);
    #[cfg(feature = "ideal-rle1")]
    mibz2::ideal::rle1::set_mode(mibz2::ideal::Mode::Real);
    #[cfg(feature = "ideal-delivery")]
    mibz2::ideal::delivery::set_mode(mibz2::ideal::delivery::Mode::Real);
}

#[cfg(feature = "ideal-construction")]
#[test]
fn ideal_construction_byte_identical_and_skips_work() {
    use mibz2::ideal::{construction as ic, Mode};
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_all_modes();

    for CorpusInput {
        name,
        z,
        len: want_len,
        sha: want_sha,
        blocks: total_blocks,
    } in ab_corpora_inputs()
    {
        ic::clear_store();
        let _ = ic::take_constructed();
        let _ = ic::take_replayed();
        let _ = ic::take_refused_replayed();

        ic::set_mode(Mode::Record);
        let (rec_len, rec_sha) = decode_workers_sha256(z, 4);
        assert_eq!((rec_len, rec_sha), (want_len, want_sha), "{name} record");
        let rec_constructed = ic::take_constructed();
        assert_eq!(ic::take_replayed(), 0, "{name} record replayed");
        let (blocks, distinct, bytes, conflicts, _missing) = ic::store_size();
        assert_eq!(conflicts, 0, "{name} conflicts");
        if rec_constructed > 0 {
            assert!(blocks >= 1 && distinct >= 1 && bytes > 0, "{name}");
        }
        if name == "runs x3" {
            assert_eq!(blocks, total_blocks, "{name} indexed");
            assert_eq!(blocks, 3 * distinct, "{name} deduplicates repeated streams");
        }

        for w in WORKERS {
            let _ = ic::take_constructed();
            let _ = ic::take_replayed();
            let _ = ic::take_refused_replayed();
            ic::set_mode(Mode::Replay);
            let (got_len, got_sha) = decode_workers_sha256(z, w);
            assert_eq!(
                (got_len, got_sha),
                (want_len, want_sha),
                "{name} C replay w={w}: got {} want {}",
                hex(got_sha),
                hex(want_sha)
            );
            assert_eq!(
                ic::take_constructed(),
                0,
                "{name} C replay w={w}: construct_capped must not run"
            );
            assert_eq!(
                ic::take_replayed() + ic::take_refused_replayed(),
                rec_constructed,
                "{name} C replay w={w}: every Open block must replay"
            );
        }
        ic::set_mode(Mode::Real);
    }
    ic::clear_store();
}

#[cfg(feature = "ideal-mtf")]
#[test]
fn ideal_mtf_byte_identical_and_skips_work() {
    use mibz2::ideal::{mtf as im, Mode};
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_all_modes();

    for CorpusInput {
        name,
        z,
        len: want_len,
        sha: want_sha,
        blocks: total_blocks,
    } in ab_corpora_inputs()
    {
        im::clear_store();
        let _ = im::take_executed();
        let _ = im::take_replayed();

        im::set_mode(Mode::Record);
        let (rec_len, rec_sha) = decode_workers_sha256(z, 4);
        assert_eq!((rec_len, rec_sha), (want_len, want_sha), "{name} record");
        assert_eq!(im::take_executed(), total_blocks, "{name} record executed");
        assert_eq!(im::take_replayed(), 0, "{name} record replayed");
        let (blocks, distinct, bytes, conflicts, missing) = im::store_size();
        assert_eq!((blocks, conflicts, missing), (total_blocks, 0, 0), "{name}");
        assert!(distinct >= 1 && distinct <= blocks && bytes > 0, "{name}");
        if name == "runs x3" {
            assert_eq!(blocks, 3 * distinct, "{name} deduplicates repeated streams");
        }

        for w in WORKERS {
            let _ = im::take_executed();
            let _ = im::take_replayed();
            im::set_mode(Mode::Replay);
            let (got_len, got_sha) = decode_workers_sha256(z, w);
            assert_eq!(
                (got_len, got_sha),
                (want_len, want_sha),
                "{name} M replay w={w}: got {} want {}",
                hex(got_sha),
                hex(want_sha)
            );
            assert_eq!(
                im::take_executed(),
                0,
                "{name} M replay w={w}: MTF/RLE2 must not run"
            );
            assert_eq!(
                im::take_replayed(),
                total_blocks,
                "{name} M replay w={w}: every block must replay"
            );
        }
        im::set_mode(Mode::Real);
    }
    im::clear_store();
}

#[cfg(feature = "ideal-rle1")]
#[test]
fn ideal_rle1_byte_identical_and_skips_work() {
    use mibz2::ideal::{rle1 as ir, Mode};
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_all_modes();

    for CorpusInput {
        name,
        z,
        len: want_len,
        sha: want_sha,
        blocks: total_blocks,
    } in ab_corpora_inputs()
    {
        ir::clear_store();
        let _ = ir::take_executed();
        let _ = ir::take_replayed();

        ir::set_mode(Mode::Record);
        let (rec_len, rec_sha) = decode_workers_sha256(z, 4);
        assert_eq!((rec_len, rec_sha), (want_len, want_sha), "{name} record");
        assert_eq!(ir::take_executed(), total_blocks, "{name} record executed");
        assert_eq!(ir::take_replayed(), 0, "{name} record replayed");
        let (blocks, distinct, bytes, conflicts, missing) = ir::store_size();
        assert_eq!((blocks, conflicts, missing), (total_blocks, 0, 0), "{name}");
        assert!(distinct >= 1 && distinct <= blocks && bytes > 0, "{name}");
        if name == "runs x3" {
            assert_eq!(blocks, 3 * distinct, "{name} deduplicates repeated streams");
        }

        for w in WORKERS {
            let _ = ir::take_executed();
            let _ = ir::take_replayed();
            ir::set_mode(Mode::Replay);
            let (got_len, got_sha) = decode_workers_sha256(z, w);
            assert_eq!(
                (got_len, got_sha),
                (want_len, want_sha),
                "{name} R replay w={w}: got {} want {}",
                hex(got_sha),
                hex(want_sha)
            );
            assert_eq!(
                ir::take_executed(),
                0,
                "{name} R replay w={w}: RLE1 and block CRC must not run"
            );
            assert_eq!(
                ir::take_replayed(),
                total_blocks,
                "{name} R replay w={w}: every block must replay"
            );
        }
        ir::set_mode(Mode::Real);
    }
    ir::clear_store();
}

#[cfg(feature = "ideal-delivery")]
#[test]
fn ideal_delivery_discards_in_order_without_copy() {
    use mibz2::ideal::delivery as id;
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_all_modes();

    for CorpusInput {
        name,
        z,
        len: want_len,
        sha: want_sha,
        blocks: total_blocks,
    } in ab_corpora_inputs()
    {
        for w in WORKERS {
            // Real mode: copies every decoded byte to caller.
            let _ = id::take_stats();
            id::set_mode(id::Mode::Real);
            let (got_len, got_sha) = decode_workers_sha256(z, w);
            assert_eq!(
                (got_len, got_sha),
                (want_len, want_sha),
                "{name} D real w={w}"
            );
            let st = id::take_stats();
            assert_eq!(st.copied_bytes, want_len, "{name} D real copied w={w}");
            assert_eq!(st.discarded_blocks, 0, "{name} D real discarded w={w}");

            // Discard mode: takes every block in order, zero bytes copied to caller.
            let _ = id::take_stats();
            id::set_mode(id::Mode::Discard);
            let out = decode_workers(z, w);
            assert!(out.is_empty(), "{name} D discard returns no bytes w={w}");
            let st = id::take_stats();
            assert_eq!(st.copied_bytes, 0, "{name} D discard zero copy w={w}");
            assert_eq!(
                st.discarded_blocks, total_blocks,
                "{name} D discard blocks w={w}"
            );
            assert_eq!(st.discarded_bytes, want_len, "{name} D discard bytes w={w}");

            // Check mode: exact same discard path, hashes taken blocks in order -> same SHA-256.
            let _ = id::take_stats();
            id::clear_check();
            id::set_mode(id::Mode::Check);
            let out = decode_workers(z, w);
            assert!(out.is_empty(), "{name} D check returns no bytes w={w}");
            let check_sha = id::take_check_sha256();
            let st = id::take_stats();
            assert_eq!(st.copied_bytes, 0, "{name} D check zero copy w={w}");
            assert_eq!(st.discarded_blocks, total_blocks);
            assert_eq!(st.discarded_bytes, want_len);
            assert_eq!(
                check_sha,
                want_sha,
                "{name} D check w={w}: got {} want {}",
                hex(check_sha),
                hex(want_sha)
            );
        }
        id::set_mode(id::Mode::Real);
    }

    // Empty stream (0 blocks): Discard finishes cleanly with 0 blocks, 0 bytes.
    let empty_z: &'static [u8] = compress(&[], 9).leak();
    let _ = id::take_stats();
    id::set_mode(id::Mode::Discard);
    let out = decode_workers(empty_z, 4);
    assert!(out.is_empty());
    let st = id::take_stats();
    assert_eq!(st, id::Stats::default());
    id::set_mode(id::Mode::Real);
}

#[cfg(all(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
    feature = "ideal-delivery",
))]
#[test]
fn all_four_oracles_compose_byte_identically() {
    use mibz2::ideal::{construction as ic, delivery as id, mtf as im, rle1 as ir, Mode};
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_all_modes();

    for CorpusInput {
        name,
        z,
        len: want_len,
        sha: want_sha,
        blocks: total_blocks,
    } in ab_corpora_inputs()
    {
        ic::clear_store();
        im::clear_store();
        ir::clear_store();
        let _ = ic::take_constructed();
        let _ = im::take_executed();
        let _ = ir::take_executed();

        // Record all three oracles in a single pass.
        ic::set_mode(Mode::Record);
        im::set_mode(Mode::Record);
        ir::set_mode(Mode::Record);
        id::set_mode(id::Mode::Real);
        let (rec_len, rec_sha) = decode_workers_sha256(z, 4);
        assert_eq!(
            (rec_len, rec_sha),
            (want_len, want_sha),
            "{name} combined record"
        );
        let rec_open = ic::take_constructed();

        // Replay C + M + R together (D Real -> check output bytes directly).
        for w in WORKERS {
            let _ = ic::take_constructed();
            let _ = ic::take_replayed();
            let _ = ic::take_refused_replayed();
            let _ = im::take_executed();
            let _ = im::take_replayed();
            let _ = ir::take_executed();
            let _ = ir::take_replayed();
            ic::set_mode(Mode::Replay);
            im::set_mode(Mode::Replay);
            ir::set_mode(Mode::Replay);
            id::set_mode(id::Mode::Real);

            let (got_len, got_sha) = decode_workers_sha256(z, w);
            assert_eq!(
                (got_len, got_sha),
                (want_len, want_sha),
                "{name} C+M+R w={w}"
            );
            assert_eq!(
                ic::take_constructed(),
                0,
                "{name} C+M+R C constructed w={w}"
            );
            assert_eq!(
                ic::take_replayed() + ic::take_refused_replayed(),
                rec_open,
                "{name} C+M+R C replayed w={w}"
            );
            assert_eq!(im::take_executed(), 0, "{name} C+M+R M executed w={w}");
            assert_eq!(
                im::take_replayed(),
                total_blocks,
                "{name} C+M+R M replayed w={w}"
            );
            assert_eq!(ir::take_executed(), 0, "{name} C+M+R R executed w={w}");
            assert_eq!(
                ir::take_replayed(),
                total_blocks,
                "{name} C+M+R R replayed w={w}"
            );

            // Now with D Check (all four ideal together).
            let _ = id::take_stats();
            id::clear_check();
            id::set_mode(id::Mode::Check);
            let out = decode_workers(z, w);
            assert!(out.is_empty());
            let st = id::take_stats();
            assert_eq!(st.copied_bytes, 0);
            assert_eq!(st.discarded_blocks, total_blocks);
            assert_eq!(st.discarded_bytes, want_len);
            assert_eq!(id::take_check_sha256(), want_sha, "{name} C+M+R+D w={w}");
        }
        // Early cancellation during combined Replay must end cleanly without hanging.
        ic::set_mode(Mode::Replay);
        im::set_mode(Mode::Replay);
        ir::set_mode(Mode::Replay);
        id::set_mode(id::Mode::Real);
        let mut r = BzReader::parallel_workers(z, 4, 8, Kernels::detect(), true);
        let mut buf = [0u8; 1000];
        let _ = r.read(&mut buf).expect("partial read");
        drop(r);
        println!(
            "AB_CORPUS {name:10} blocks={total_blocks:3} open={rec_open:3} decoded_bytes={want_len:8} sha256={}",
            hex(want_sha)
        );
        reset_all_modes();
    }
    ic::clear_store();
    im::clear_store();
    ir::clear_store();
}

/// Wikidata and wikidata x4 verification + store sizes + activity ledger check.
/// Run with:
/// `MIBZ2_WIKIDATA=/path/to/wikidata.bz2 cargo test --release --features "ideal-construction ideal-mtf ideal-rle1 ideal-delivery trace" --test ideal_oracles wikidata -- --ignored --nocapture`
#[cfg(all(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
    feature = "ideal-delivery",
))]
#[test]
#[ignore]
fn wikidata_and_x4_oracles_verified() {
    use mibz2::ideal::{construction as ic, delivery as id, mtf as im, rle1 as ir, Mode};
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_all_modes();

    let path = std::env::var("MIBZ2_WIKIDATA")
        .expect("set MIBZ2_WIKIDATA to the wikidata.bz2 built by bench/wikidata.py");
    let one_bytes = std::fs::read(&path).expect("read wikidata.bz2");
    let mut x4_bytes = Vec::with_capacity(one_bytes.len() * 4);
    for _ in 0..4 {
        x4_bytes.extend_from_slice(&one_bytes);
    }
    let one: &'static [u8] = one_bytes.leak();
    let x4: &'static [u8] = x4_bytes.leak();

    for (label, z) in [("wikidata", one), ("wikidata x4", x4)] {
        let total_blocks = count_blocks(z);
        reset_all_modes();
        let (want_len, want_sha) = decode_workers_sha256(z, 4);
        println!(
            "\n=== {label}: blocks={total_blocks} decoded_bytes={want_len} sha256={} ===",
            hex(want_sha)
        );

        ic::clear_store();
        im::clear_store();
        ir::clear_store();
        let _ = ic::take_constructed();
        let _ = im::take_executed();
        let _ = ir::take_executed();

        ic::set_mode(Mode::Record);
        im::set_mode(Mode::Record);
        ir::set_mode(Mode::Record);
        let (rec_len, rec_sha) = decode_workers_sha256(z, 4);
        assert_eq!((rec_len, rec_sha), (want_len, want_sha));
        let rec_open = ic::take_constructed();
        reset_all_modes();

        let cs = ic::store_size();
        let ms = im::store_size();
        let rs = ir::store_size();
        println!(
            "STORE {label} C: blocks={} distinct={} bytes={} ({:.2} MiB) conflicts={} missing={} open_blocks={rec_open}",
            cs.0, cs.1, cs.2, cs.2 as f64 / (1024.0 * 1024.0), cs.3, cs.4
        );
        println!(
            "STORE {label} M: blocks={} distinct={} bytes={} ({:.2} MiB) conflicts={} missing={}",
            ms.0,
            ms.1,
            ms.2,
            ms.2 as f64 / (1024.0 * 1024.0),
            ms.3,
            ms.4
        );
        println!(
            "STORE {label} R: blocks={} distinct={} bytes={} ({:.2} MiB) conflicts={} missing={}",
            rs.0,
            rs.1,
            rs.2,
            rs.2 as f64 / (1024.0 * 1024.0),
            rs.3,
            rs.4
        );
        assert_eq!(cs.3, 0);
        assert_eq!((ms.3, ms.4), (0, 0));
        assert_eq!((rs.3, rs.4), (0, 0));
        if label == "wikidata x4" {
            assert_eq!(ms.0, 4 * ms.1, "M deduplicates x4");
            assert_eq!(rs.0, 4 * rs.1, "R deduplicates x4");
            assert_eq!(rec_open, 4 * cs.1, "C deduplicates x4");
            assert_eq!(cs.0 - cs.4, rec_open, "C indexed entries match open_blocks");
        }

        for w in WORKERS {
            // C alone
            reset_all_modes();
            let _ = ic::take_constructed();
            let _ = ic::take_replayed();
            let _ = ic::take_refused_replayed();
            ic::set_mode(Mode::Replay);
            let (len, sha) = decode_workers_sha256(z, w);
            assert_eq!((len, sha), (want_len, want_sha), "{label} C w={w}");
            let (c_exec, c_rep, c_ref) = (
                ic::take_constructed(),
                ic::take_replayed(),
                ic::take_refused_replayed(),
            );
            assert_eq!(c_exec, 0);
            assert_eq!(c_rep + c_ref, rec_open);
            println!(
                "CHECK {label} w={w} C: sha={} ok constructed={c_exec} replayed={c_rep}",
                hex(sha)
            );

            // M alone
            reset_all_modes();
            let _ = im::take_executed();
            let _ = im::take_replayed();
            im::set_mode(Mode::Replay);
            let (len, sha) = decode_workers_sha256(z, w);
            assert_eq!((len, sha), (want_len, want_sha), "{label} M w={w}");
            let (m_exec, m_rep) = (im::take_executed(), im::take_replayed());
            assert_eq!(m_exec, 0);
            assert_eq!(m_rep, total_blocks);
            println!(
                "CHECK {label} w={w} M: sha={} ok executed={m_exec} replayed={m_rep}",
                hex(sha)
            );

            // R alone
            reset_all_modes();
            let _ = ir::take_executed();
            let _ = ir::take_replayed();
            ir::set_mode(Mode::Replay);
            let (len, sha) = decode_workers_sha256(z, w);
            assert_eq!((len, sha), (want_len, want_sha), "{label} R w={w}");
            let (r_exec, r_rep) = (ir::take_executed(), ir::take_replayed());
            assert_eq!(r_exec, 0);
            assert_eq!(r_rep, total_blocks);
            println!(
                "CHECK {label} w={w} R: sha={} ok executed={r_exec} replayed={r_rep}",
                hex(sha)
            );

            // D Discard & Check
            reset_all_modes();
            let _ = id::take_stats();
            id::set_mode(id::Mode::Discard);
            let out = decode_workers(z, w);
            assert!(out.is_empty());
            let st_d = id::take_stats();
            assert_eq!(st_d.copied_bytes, 0);
            assert_eq!(st_d.discarded_blocks, total_blocks);
            assert_eq!(st_d.discarded_bytes, want_len);

            id::clear_check();
            id::set_mode(id::Mode::Check);
            let out = decode_workers(z, w);
            assert!(out.is_empty());
            let d_sha = id::take_check_sha256();
            let st_c = id::take_stats();
            assert_eq!(st_c.copied_bytes, 0);
            assert_eq!(d_sha, want_sha, "{label} D w={w}");
            println!(
                "CHECK {label} w={w} D: sha={} ok copied={} discarded_blocks={} discarded_bytes={}",
                hex(d_sha),
                st_d.copied_bytes,
                st_d.discarded_blocks,
                st_d.discarded_bytes
            );

            // All four combined (C + M + R + D Check)
            reset_all_modes();
            let _ = ic::take_constructed();
            let _ = im::take_executed();
            let _ = ir::take_executed();
            let _ = id::take_stats();
            id::clear_check();
            ic::set_mode(Mode::Replay);
            im::set_mode(Mode::Replay);
            ir::set_mode(Mode::Replay);
            id::set_mode(id::Mode::Check);
            let out = decode_workers(z, w);
            assert!(out.is_empty());
            let all_sha = id::take_check_sha256();
            let st_all = id::take_stats();
            assert_eq!(all_sha, want_sha, "{label} C+M+R+D w={w}");
            assert_eq!(ic::take_constructed(), 0);
            assert_eq!(im::take_executed(), 0);
            assert_eq!(ir::take_executed(), 0);
            assert_eq!(st_all.copied_bytes, 0);
            println!("CHECK {label} w={w} C+M+R+D: sha={} ok", hex(all_sha));
        }

        // Activity ledger timings on wikidata (1 worker, fixed 256 KiB read buffer as in examples/four_arm.rs)
        #[cfg(feature = "trace")]
        if label == "wikidata" {
            use mibz2::trace::{self, Kind};
            let mut read_buf = vec![0u8; 256 * 1024];
            let mut measure = |arm: &str, setup: &dyn Fn()| {
                reset_all_modes();
                setup();
                let _ = trace::take();
                trace::set_level(1);
                let mut r = BzReader::parallel_workers(
                    z,
                    1,
                    mibz2::parallel::SLOTS_PER_THREAD,
                    Kernels::detect(),
                    true,
                );
                loop {
                    match r.read(&mut read_buf) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(e) => panic!("ledger decode failed: {e}"),
                    }
                }
                drop(r);
                trace::set_level(0);
                let traces = trace::take();
                let (mut mtf_ns, mut inv_ns, mut rle_ns, mut deliv_ns) = (0u64, 0u64, 0u64, 0u64);
                for tt in &traces {
                    let mut dec_s = None;
                    let mut inv_s = None;
                    let mut inv_e = None;
                    let mut taken_t = None;
                    for mk in &tt.marks {
                        match mk.kind {
                            Kind::DecodeStart => dec_s = Some(mk.ns),
                            Kind::InvertStart => {
                                if let Some(t) = dec_s.take() {
                                    mtf_ns += mk.ns.saturating_sub(t);
                                }
                                inv_s = Some(mk.ns);
                            }
                            Kind::InvertEnd => {
                                if let Some(t) = inv_s.take() {
                                    inv_ns += mk.ns.saturating_sub(t);
                                }
                                inv_e = Some(mk.ns);
                            }
                            Kind::DecodeEnd => {
                                if let Some(t) = inv_e.take() {
                                    rle_ns += mk.ns.saturating_sub(t);
                                }
                            }
                            Kind::Taken => taken_t = Some(mk.ns),
                            Kind::ReadWait => {
                                if let Some(t) = taken_t.take() {
                                    deliv_ns += mk.ns.saturating_sub(t);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                println!(
                    "LEDGER {arm:8}: M(DecS->InvS)={:7.2} ms  C+E(InvS->InvE)={:7.2} ms  R(InvE->DecE)={:7.2} ms  D(Taken->ReadW)={:7.2} ms",
                    mtf_ns as f64 / 1e6,
                    inv_ns as f64 / 1e6,
                    rle_ns as f64 / 1e6,
                    deliv_ns as f64 / 1e6,
                );
            };
            measure("Real", &|| {});
            measure("C=Replay", &|| ic::set_mode(Mode::Replay));
            measure("M=Replay", &|| im::set_mode(Mode::Replay));
            measure("R=Replay", &|| ir::set_mode(Mode::Replay));
            measure("D=Disc", &|| id::set_mode(id::Mode::Discard));
            measure("C+M+R+D", &|| {
                ic::set_mode(Mode::Replay);
                im::set_mode(Mode::Replay);
                ir::set_mode(Mode::Replay);
                id::set_mode(id::Mode::Discard);
            });
            reset_all_modes();
        }
    }

    // Also print ab gen corpora SHA-256 summary table
    reset_all_modes();
    println!("\n=== ab gen corpora (level 9) SHA-256 check at w=1,4,8 ===");
    for CorpusInput {
        name,
        z,
        len: want_len,
        sha: want_sha,
        blocks: total_blocks,
    } in ab_corpora_inputs()
    {
        reset_all_modes();
        ic::clear_store();
        im::clear_store();
        ir::clear_store();
        ic::set_mode(Mode::Record);
        im::set_mode(Mode::Record);
        ir::set_mode(Mode::Record);
        assert_eq!(decode_workers_sha256(z, 4), (want_len, want_sha));
        reset_all_modes();
        for w in WORKERS {
            ic::set_mode(Mode::Replay);
            assert_eq!(decode_workers_sha256(z, w), (want_len, want_sha));
            reset_all_modes();
            im::set_mode(Mode::Replay);
            assert_eq!(decode_workers_sha256(z, w), (want_len, want_sha));
            reset_all_modes();
            ir::set_mode(Mode::Replay);
            assert_eq!(decode_workers_sha256(z, w), (want_len, want_sha));
            reset_all_modes();
            id::clear_check();
            id::set_mode(id::Mode::Check);
            assert!(decode_workers(z, w).is_empty());
            assert_eq!(id::take_check_sha256(), want_sha);
            reset_all_modes();
        }
        println!(
            "AB_CORPUS {name:10} blocks={total_blocks:3} decoded_bytes={want_len:8} sha256={} (C,M,R,D ok at w=1,4,8)",
            hex(want_sha)
        );
    }

    ic::clear_store();
    im::clear_store();
    ir::clear_store();
    reset_all_modes();
}

/// Minimal std-only SHA-256 (FIPS 180-4) for byte-identity checks in tests
struct Sha256 {
    state: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total_bytes: u64,
}

impl Sha256 {
    const H0: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    const fn new() -> Self {
        Self {
            state: Self::H0,
            buf: [0; 64],
            buf_len: 0,
            total_bytes: 0,
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for (&ki, &wi) in Self::K.iter().zip(&w) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(ki)
                .wrapping_add(wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, delta) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(delta);
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total_bytes += data.len() as u64;
        if self.buf_len > 0 {
            let take = (64 - self.buf_len).min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        while data.len() >= 64 {
            let block: &[u8; 64] = data[..64].try_into().unwrap();
            self.compress(block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    fn finish(mut self) -> [u8; 32] {
        let bits = self.total_bytes.wrapping_mul(8);
        self.buf[self.buf_len] = 0x80;
        self.buf_len += 1;
        if self.buf_len > 56 {
            self.buf[self.buf_len..].fill(0);
            let block = self.buf;
            self.compress(&block);
            self.buf_len = 0;
        }
        self.buf[self.buf_len..56].fill(0);
        self.buf[56..64].copy_from_slice(&bits.to_be_bytes());
        let block = self.buf;
        self.compress(&block);
        let mut out = [0u8; 32];
        for (i, &word) in self.state.iter().enumerate() {
            out[4 * i..4 * i + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}
