//! Research only (`mtf-fold`): the parallel decoder with the MTF-into-construction fold on and
//! off, on the test corpora at several levels, whole, cut short and with a byte changed: one
//! output and one error (kind, block and bit) either way, the output libbzip2's, and the fold
//! taken on the blocks it is for. One test, as the fold's switch and counters are the
//! process's.
mod common;
use common::{compress, drive, oracle};
use mibz2::bwt::{ForcedOpenCells, OpenCells};
use mibz2::runs::fold;
use mibz2::{Kernels, StreamDecoder};

fn decode(data: &[u8], k: Kernels, forced: bool) -> (Vec<u8>, Result<(), mibz2::Error>) {
    let dec = if forced {
        StreamDecoder::with_inverse::<ForcedOpenCells>(4, 16, k)
    } else {
        StreamDecoder::with_inverse::<OpenCells>(4, 16, k)
    };
    drive(&dec, data, 1 << 20, 1 << 16)
}

#[test]
fn fold_on_and_off_one_output_and_one_error() {
    let mut kernels = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        kernels.push(Kernels::detect());
    }
    let mut inputs = Vec::new();
    for (name, data) in common::corpora(1) {
        for level in [1, 9] {
            let z = compress(&data, level);
            let mut spoiled = z.clone();
            let at = spoiled.len() / 3;
            spoiled[at] ^= 0x10;
            let cut = z[..z.len() * 2 / 3].to_vec();
            inputs.push((format!("{name} -{level}"), z));
            inputs.push((format!("{name} -{level}, a byte changed"), spoiled));
            inputs.push((format!("{name} -{level}, cut"), cut));
        }
    }
    fold::take_counts();
    let mut on_counts = fold::Counts::default();
    for (what, z) in &inputs {
        let want = oracle(z);
        for &k in &kernels {
            for forced in [false, true] {
                fold::set_enabled(true);
                let on = decode(z, k, forced);
                let c = fold::take_counts();
                on_counts.folded += c.folded;
                on_counts.bailed += c.bailed;
                on_counts.not_open += c.not_open;
                fold::set_enabled(false);
                let off = decode(z, k, forced);
                let c = fold::take_counts();
                assert!(
                    c == fold::Counts::default(),
                    "{what}: folded while off: {c:?}"
                );
                assert!(
                    on.1 == off.1,
                    "{what}, {k:?}: {:?} against {:?}",
                    on.1,
                    off.1
                );
                assert!(on.0 == off.0, "{what}, {k:?}: the bytes");
                if let (Ok(w), Ok(())) = (&want, &on.1) {
                    assert!(&on.0 == w, "{what}, {k:?}: not libbzip2's bytes");
                }
                if want.is_ok() {
                    assert!(on.1.is_ok(), "{what}, {k:?}: {:?}", on.1);
                }
            }
        }
    }
    fold::set_enabled(true);
    eprintln!("{on_counts:?}");
    assert!(on_counts.folded > 50, "{on_counts:?}");
    #[cfg(feature = "scan-oracle")]
    check_fs_replay(&kernels);
}

/// Research only (`mtf-fold` + `scan-oracle`): in `scan_oracle::Mode::Replay`, `b.syms` is empty
/// and workers borrow `&[u16]` via `scan_oracle::with_syms`. Verify that `runs::fold::wants` and
/// both fold fronts (scalar and AVX2) actually fold Open blocks in the FS arm (`folded > 0`,
/// `bailed == 0`), produce exact bytes, and match `F` block-for-block.
#[cfg(feature = "scan-oracle")]
fn check_fs_replay(kernels: &[Kernels]) {
    use mibz2::scan_oracle;
    // Build a multi-block compressible corpus that exercises Open blocks (>= MIN_SYMS symbols).
    let mut raw = Vec::new();
    for (name, data) in common::corpora(2) {
        if matches!(name, "text" | "pattern" | "runs") {
            raw.extend_from_slice(&data);
        }
    }
    let z = compress(&raw, 9);
    let want = oracle(&z).expect("libbzip2 oracle");

    scan_oracle::clear_store();
    scan_oracle::set_scanner_cpu(None);
    fold::set_enabled(false);
    scan_oracle::set_mode(scan_oracle::Mode::Record);
    let _ = scan_oracle::take_scanned();
    let _ = scan_oracle::take_replayed();
    let rec = decode(&z, Kernels::detect(), false);
    assert!(rec.1.is_ok(), "record pass failed: {:?}", rec.1);
    assert_eq!(rec.0, want, "record pass bytes mismatch");
    let scanned = scan_oracle::take_scanned();
    assert!(scanned > 0, "record pass scanned 0 blocks");
    let (indexed, _, _, conflicts, missing) = scan_oracle::store_size();
    assert_eq!(conflicts, 0, "scan oracle store conflicts");
    assert_eq!(missing, 0, "scan oracle store missing blocks");
    assert_eq!(indexed, scanned, "indexed != scanned");

    // Compare F (real scan + fold) against FS (zero-copy Replay scan + fold) on every kernel.
    for &k in kernels {
        scan_oracle::set_mode(scan_oracle::Mode::Scan);
        fold::set_enabled(true);
        let _ = fold::take_counts();
        let f_arm = decode(&z, k, false);
        assert!(f_arm.1.is_ok(), "F arm failed on {k:?}: {:?}", f_arm.1);
        assert_eq!(f_arm.0, want, "F arm bytes mismatch on {k:?}");
        let f_counts = fold::take_counts();
        assert!(
            f_counts.folded > 0,
            "F arm folded 0 blocks on {k:?}: {f_counts:?}"
        );
        assert_eq!(f_counts.bailed, 0, "F arm bailed on {k:?}: {f_counts:?}");

        scan_oracle::set_mode(scan_oracle::Mode::Replay);
        let _ = scan_oracle::take_scanned();
        let _ = scan_oracle::take_replayed();
        let _ = scan_oracle::take_lock_acquires();
        let _ = fold::take_counts();
        let fs_arm = decode(&z, k, false);
        assert!(fs_arm.1.is_ok(), "FS arm failed on {k:?}: {:?}", fs_arm.1);
        assert_eq!(fs_arm.0, want, "FS arm bytes mismatch on {k:?}");
        let fs_counts = fold::take_counts();
        let fs_scanned = scan_oracle::take_scanned();
        let fs_replayed = scan_oracle::take_replayed();
        let fs_locks = scan_oracle::take_lock_acquires();
        assert_eq!(fs_scanned, 0, "FS arm ran real scanner on {k:?}");
        assert_eq!(
            fs_replayed, indexed,
            "FS arm did not replay every block on {k:?}"
        );
        assert_eq!(fs_locks, 0, "FS arm acquired fallback lock on {k:?}");
        assert!(
            fs_counts.folded > 0,
            "FS arm folded 0 blocks on {k:?} (borrowed syms glue broken): {fs_counts:?}"
        );
        assert_eq!(fs_counts.bailed, 0, "FS arm bailed on {k:?}: {fs_counts:?}");
        assert_eq!(
            fs_counts, f_counts,
            "FS fold counts diverged from F on {k:?}: FS={fs_counts:?} F={f_counts:?}"
        );
    }
    scan_oracle::set_mode(scan_oracle::Mode::Scan);
    fold::set_enabled(true);
}
