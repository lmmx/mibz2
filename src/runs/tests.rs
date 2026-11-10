//! The records' path against the column's: `front` (both kernels) against
//! `mtf::mtf_scalar`, results and errors; the records against the column's runs; `fill`
//! (both kernels) against the column's LF table, word for word; the backward walks (plain,
//! and the room walk at several lane and segment counts) against `bwt::walk` on the
//! column's T-vector; every broken tiling refused; and `decode_block` by records against
//! the column's kernels.

use super::*;
use crate::bwt::{self, Plain, Segmented};
use crate::{decode_block, mtf, Scratch};

/// The kernels to check: the scalar ones, and this CPU's where it has more
fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    v
}

/// A column's runs cut at CAP, and each byte's count of them: the plainest scan
fn runs_of_column(ll: &[u8]) -> (Vec<u16>, [u32; 256]) {
    let (mut runs, mut per) = (Vec::new(), [0u32; 256]);
    let mut i = 0;
    while i < ll.len() {
        let c = ll[i];
        let mut j = i;
        while j < ll.len() && ll[j] == c {
            j += 1;
        }
        let mut left = j - i;
        while left > 0 {
            let len = left.min(CAP as usize);
            runs.push((c as u16) << 8 | len as u16);
            per[c as usize] += 1;
            left -= len;
        }
        i = j;
    }
    (runs, per)
}

/// xorshift64*
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 11) % n
    }
}

/// The run symbols (RUNA 0, RUNB 1) that code a run of `len` rows (bijective base 2)
fn run_syms(mut len: u32, out: &mut Vec<u16>) {
    while len > 0 {
        if len & 1 == 1 {
            out.push(0);
            len = (len - 1) / 2;
        } else {
            out.push(1);
            len = (len - 2) / 2;
        }
    }
}

/// A made-up block (the Pathfinder's generator, `examples/run_front.rs`, with two shapes
/// added): `in_use` bytes, symbols that are run symbols with probability `run_pct`%, literals
/// near the front or anywhere (indices past 31 to the list's end), runs coded to exact
/// adversarial lengths around the record's cap (126-128, 254-256, ...) and long ones, run
/// symbols 21 and 22 long (`RunTooLong`'s edge), and an end-of-block symbol at the end, in
/// the middle as well, or nowhere
fn synthetic(rng: &mut Rng, syms_max: u64) -> Symbols {
    let in_use = match rng.below(4) {
        0 => 1 + rng.below(4) as usize,
        1 => 256,
        _ => 1 + rng.below(256) as usize,
    };
    let mut bytes: [u8; 256] = std::array::from_fn(|i| i as u8);
    for i in (1..256).rev() {
        bytes.swap(i, rng.below(i as u64 + 1) as usize);
    }
    let mut seq_to_unseq = [0u8; 256];
    seq_to_unseq[..in_use].copy_from_slice(&bytes[..in_use]);
    let eob = in_use as u16 + 1;
    let run_pct = [5, 30, 60, 90, 97][rng.below(5) as usize];
    let wide = rng.below(3) == 0;
    let shaped = rng.below(3) == 0;
    let len = match rng.below(4) {
        0 => rng.below(40) as usize,
        _ => rng.below(syms_max) as usize,
    };
    let mut syms = Vec::with_capacity(len + 64);
    while syms.len() < len {
        if shaped && rng.below(4) == 0 {
            // a run of an exact length: about the cap, a multiple of it, or long
            let l = match rng.below(100) {
                0..=24 => 125 + rng.below(5) as u32,
                25..=49 => 253 + rng.below(5) as u32,
                50..=74 => CAP * (1 + rng.below(8) as u32) + rng.below(3) as u32,
                75..=97 => 1 + rng.below(2000) as u32,
                // (Miri: runs of at most a few thousand rows, or it takes hours)
                _ if cfg!(miri) => 1 + rng.below(200) as u32,
                98 => (1 << 20) + rng.below(1 << 20) as u32,
                // 21 or 22 run symbols: 2^21 - 1 .. 2^23 - 2 rows
                _ => (1 << 21) - 1 + rng.below(1 << 22) as u32,
            };
            run_syms(l, &mut syms);
        } else if in_use == 1 || rng.below(100) < run_pct {
            syms.push(rng.below(2) as u16);
        } else {
            // an index in 1..in_use: any, or mostly the list's first few
            let nn = if wide || rng.below(16) == 0 {
                1 + rng.below(in_use as u64 - 1)
            } else {
                1 + rng.below((in_use as u64 - 1).min(6))
            };
            syms.push(nn as u16 + 1);
        }
    }
    match rng.below(8) {
        0 => {}
        1 if !syms.is_empty() => {
            let at = rng.below(syms.len() as u64) as usize;
            syms[at] = eob;
            syms.push(eob);
        }
        _ => syms.push(eob),
    }
    Symbols {
        crc: 0,
        orig_ptr: rng.next() as u32 & 0xff_ffff,
        seq_to_unseq,
        syms,
        eob,
        max_len: 1 << 23,
    }
}

/// What the checks have seen
#[derive(Default, Debug)]
struct Seen {
    ok: usize,
    too_long: usize,
    run_too_long: usize,
    rows: usize,
    records: usize,
    /// records cut from a run over CAP, and literals past the list's first 32
    cut: usize,
    deep: usize,
    tables: usize,
    walks: usize,
}

/// The column's LF table, by its definition: `L[i] | LF(i) << 8`, LF(i) = C(L[i]) + the
/// copies of L[i] before row i
fn lf_of(ll: &[u8]) -> Vec<u32> {
    let counts = crate::run_fed::counts_of(ll);
    let mut next = [0u32; 256];
    let mut cum = 0;
    for c in 0..256 {
        next[c] = cum;
        cum += counts[c];
    }
    ll.iter()
        .map(|&c| {
            let e = c as u32 | next[c as usize] << 8;
            next[c as usize] += 1;
            e
        })
        .collect()
}

/// The backward walks of `p` from `orig` against `bwt::walk` of the column's T-vector
fn check_walks(p: &[u32], tt: &bwt::TVector, orig: u32, seg: &mut bwt::SegScratch, what: &str) {
    let mut want = Vec::new();
    bwt::walk(tt, orig, &mut want);
    let mut got = vec![9u8; 3];
    bwt::walk_back(&bwt::TVector(p.to_vec()), orig, &mut got);
    assert!(got == want, "{what}: the plain backward walk");
    for (lanes, segments) in [(1, 2), (4, 8), (32, 64)] {
        let mut t = bwt::TVector(p.to_vec());
        let mut got = vec![9u8; 3];
        match lanes {
            1 => bwt::walk_back_room_with::<1>(&mut t, orig, &mut got, seg, segments),
            4 => bwt::walk_back_room_with::<4>(&mut t, orig, &mut got, seg, segments),
            _ => bwt::walk_back_room_with::<32>(&mut t, orig, &mut got, seg, segments),
        }
        assert!(got == want, "{what}: the backward room walk, {lanes} lanes");
        // the starts' flags cleared again: the table as it was
        assert!(t.0 == p, "{what}: the table after the room walk");
    }
}

/// One block's symbols through both fronts and, where it decodes, both fills and the
/// backward walks, everything compared with the column's path
fn check_block(b: &Symbols, r: &mut Runs, seen: &mut Seen, what: &str) {
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let want = mtf::mtf_scalar(b, &mut ll, &mut counts);
    match want {
        Ok(_) => seen.ok += 1,
        Err(ErrorKind::BlockTooLong) => seen.too_long += 1,
        Err(ErrorKind::RunTooLong) => seen.run_too_long += 1,
        Err(ref e) => panic!("{what}: mtf_scalar's {e:?}"),
    }
    let (runs, per) = match want {
        Ok(_) => runs_of_column(&ll),
        Err(_) => (Vec::new(), [0; 256]),
    };
    for k in kernels() {
        let got = front(b, r, k);
        assert!(got == want, "{what}, {k:?}: {got:?}, mtf {want:?}");
        let Ok(n) = want else { continue };
        assert!(r.runs == runs, "{what}, {k:?}: the records");
        assert!(r.counts == counts, "{what}, {k:?}: the row counts");
        assert!(r.per == per, "{what}, {k:?}: the record counts");
        let want_p = lf_of(&ll);
        for kf in kernels() {
            let mut tt = bwt::TVector(vec![7; 5]);
            assert!(
                fill(r, n, &mut tt, kf),
                "{what}, {k:?}, fill {kf:?}: refused"
            );
            assert!(tt.0 == want_p, "{what}, {k:?}, fill {kf:?}: the table");
        }
        seen.tables += 1;
    }
    if let Ok(n) = want {
        // the walks, once a block (they don't depend on the front's kernel)
        if n > 0 && (cfg!(miri) || n < 1 << 16) {
            let mut tt = bwt::TVector::new();
            bwt::build(&ll, &counts, &mut tt);
            let orig = b.orig_ptr % n as u32;
            check_walks(
                &lf_of(&ll),
                &tt,
                orig,
                &mut bwt::SegScratch::default(),
                what,
            );
            seen.walks += 1;
        }
        seen.rows += n;
        seen.records += runs.len();
        seen.cut += runs.windows(2).filter(|w| w[0] >> 8 == w[1] >> 8).count();
        seen.deep += b.syms.iter().filter(|&&s| s > 32 && s != b.eob).count();
    }
}

/// The equivalence on made-up blocks: `front`'s result (length or error) is
/// `mtf_scalar`'s, its records the column's runs cut at 127, its counts the column's, and
/// the table `bwt::build`'s, by each kernel, on every block and at lengths around its own
#[test]
fn miri_runs_synthetic_blocks() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut r = Runs::default();
    let mut seen = Seen::default();
    let cases = if cfg!(miri) { 6 } else { 12_000 };
    let syms_max = if cfg!(miri) { 60 } else { 3000 };
    for i in 0..cases {
        let mut b = synthetic(&mut rng, syms_max);
        check_block(&b, &mut r, &mut seen, &format!("synthetic {i}"));
        // and at lengths around its own, and one nothing reaches
        let mut lens = vec![rng.below(1 << 12) as usize];
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        if let Ok(n) = mtf::mtf_scalar(&b, &mut ll, &mut counts) {
            lens.extend([n, n + 1]);
            lens.extend(n.checked_sub(1));
        }
        for max in lens {
            b.max_len = max;
            check_block(&b, &mut r, &mut seen, &format!("synthetic {i} at {max}"));
        }
    }
    eprintln!("{seen:?}");
    if !cfg!(miri) {
        // every shape was reached: blocks, both errors, cut runs, deep literals
        assert!(seen.ok > 1000 && seen.too_long > 100 && seen.run_too_long > 100);
        assert!(seen.cut > 1000 && seen.deep > 1000, "{seen:?}");
    }
}

/// Each case of the AVX2 front's hand-off and record-writing, on a block built to reach it:
/// a run of exactly 126, 127, 128, 254 and 255 rows, between literals and at the block's
/// start and end; literals of indices 31, 32, 33 and 255; 21 and 22 run symbols together
#[test]
fn miri_runs_front_edges() {
    let mut s2u = [0u8; 256];
    for (i, x) in s2u.iter_mut().enumerate() {
        *x = (255 - i) as u8;
    }
    let mut r = Runs::default();
    let mut seen = Seen::default();
    let lens = [1, 2, 126, 127, 128, 254, 255, 256, 381, 1000];
    for (i, &first) in lens.iter().enumerate() {
        let mut syms = Vec::new();
        run_syms(first, &mut syms);
        for (j, &l) in lens.iter().enumerate() {
            let idx = [1u16, 31, 32, 33, 200, 255][(i + j) % 6];
            syms.push(idx + 1);
            run_syms(l - 1, &mut syms);
        }
        syms.push(257);
        let b = Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: s2u,
            syms,
            eob: 257,
            max_len: 900_000,
        };
        check_block(&b, &mut r, &mut seen, &format!("edges {i}"));
    }
    // 21 run symbols (2^21 - 1 rows and more) and 22 (RunTooLong), at the start and later
    // (21 run symbols are 2^21 - 1 rows at least: too many for Miri, which checks 22's error)
    let ones = if cfg!(miri) {
        &[][..]
    } else {
        &[(21, false), (21, true)][..]
    };
    for &(count, later) in ones.iter().chain(&[(22, false), (22, true)]) {
        let mut syms = Vec::new();
        if later {
            syms.extend([5, 0, 7]);
        }
        syms.extend(std::iter::repeat(0).take(count));
        syms.extend([3, 257]);
        let b = Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: s2u,
            syms,
            eob: 257,
            max_len: 1 << 23,
        };
        check_block(&b, &mut r, &mut seen, &format!("{count} run symbols"));
    }
    assert!(
        seen.run_too_long == 2 && seen.ok == lens.len() + ones.len(),
        "{seen:?}"
    );
}

/// The column's own records filled by each kernel, for a short column: the table, or None
/// if every kernel refused (and left the table empty); kernels that disagree fail
fn table_of(ll: &[u8], r: &mut Runs, brk: impl Fn(&mut Runs) -> usize) -> Option<Vec<u32>> {
    let mut got = Vec::new();
    for k in kernels() {
        let (runs, per) = runs_of_column(ll);
        r.runs = runs;
        r.per = per;
        r.counts = crate::run_fed::counts_of(ll);
        let n = brk(r);
        let mut tt = bwt::TVector(vec![3; 4]);
        let ok = fill(r, n, &mut tt, k);
        assert!(ok || tt.0.is_empty(), "{k:?}: a refused table kept");
        got.push(ok.then_some(tt.0));
    }
    assert!(got.windows(2).all(|w| w[0] == w[1]), "the kernels disagree");
    got.pop().unwrap()
}

/// Every tiling broken by one record or count is refused by both kernels, the table left
/// empty; the empty block's table is empty
#[test]
fn miri_runs_broken_tilings_refused() {
    let mut r = Runs::default();
    assert_eq!(table_of(&[], &mut r, |_| 0), Some(vec![]));
    let ll: Vec<u8> = b"aaabbbbbbcab"
        .iter()
        .chain(std::iter::repeat(&b'z').take(300))
        .chain(b"qa")
        .copied()
        .collect();
    let n = ll.len();
    assert_eq!(table_of(&ll, &mut r, |_| n), Some(lf_of(&ll)));
    // a record a row longer or shorter, of length 0, of another byte; a count off by one,
    // or moved to another byte; the wrong n either way; a record dropped or added (of
    // length 0, or 1)
    type Break<'a> = Box<dyn Fn(&mut Runs) -> usize + 'a>;
    let breaks: Vec<Break> = vec![
        Box::new(|r| {
            r.runs[1] += 1;
            n
        }),
        Box::new(|r| {
            r.runs[1] -= 1;
            n
        }),
        Box::new(|r| {
            r.runs[2] &= 0xff00;
            n
        }),
        Box::new(|r| {
            r.runs[2] += 1 << 8;
            n
        }),
        Box::new(|r| {
            r.counts[b'a' as usize] += 1;
            n
        }),
        Box::new(|r| {
            r.counts[b'a' as usize] -= 1;
            r.counts[b'b' as usize] += 1;
            n
        }),
        Box::new(|_| n + 1),
        Box::new(|_| n - 1),
        Box::new(|r| {
            r.runs.pop();
            n
        }),
        Box::new(|r| {
            r.runs.push((b'a' as u16) << 8);
            n
        }),
        Box::new(|r| {
            r.runs.push((b'a' as u16) << 8 | 1);
            n
        }),
    ];
    for (i, brk) in breaks.iter().enumerate() {
        assert!(table_of(&ll, &mut r, brk).is_none(), "break {i} accepted");
    }
}

/// `decode_block` by records (`Segmented`, which reads runs) gives what the column's kernel
/// (`Plain`, which doesn't) gives, on blocks of few symbols and of many, errors included
#[test]
fn miri_runs_decode_block_equals_the_column() {
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    let (mut sr, mut sp) = (Scratch::<Segmented>::default(), Scratch::<Plain>::default());
    let cases = if cfg!(miri) { 12 } else { 3000 };
    let mut by_runs = 0;
    for i in 0..cases {
        let mut b = synthetic(&mut rng, if cfg!(miri) { 120 } else { 3000 });
        // a bound nothing reaches, and the selector's edge on either side
        b.max_len = match rng.below(3) {
            0 => 1 << 20,
            1 => 2 * b.syms.len(),
            _ => (2 * b.syms.len()).saturating_sub(1),
        };
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        if let Ok(n) = mtf::mtf_scalar(&b, &mut ll, &mut counts) {
            if n > 0 {
                b.orig_ptr = rng.below(n as u64 + 1) as u32;
                // the CRC of what the column decodes to, so the block checks
                let mut out = Vec::new();
                let mut tt = bwt::TVector::new();
                bwt::build(&ll, &counts, &mut tt);
                if (b.orig_ptr as usize) < n {
                    bwt::walk(&tt, b.orig_ptr, &mut out);
                    let mut text = Vec::new();
                    if crate::rle::unrle(&out, &mut text, Kernels::scalar()).is_ok() {
                        b.crc = crate::crc::crc32(&text, Kernels::scalar());
                    }
                }
            }
        }
        by_runs += takes_runs(&b) as usize;
        for k in kernels() {
            let (mut a, mut c) = (vec![1u8], vec![1u8]);
            let ra = decode_block(&b, k, &mut sr, &mut a);
            let rc = decode_block(&b, k, &mut sp, &mut c);
            assert!(ra == rc, "case {i}, {k:?}: {ra:?} against {rc:?}");
            assert!(a == c, "case {i}, {k:?}: the bytes");
        }
    }
    assert!(by_runs > cases / 4, "{by_runs} of {cases} by records");
}

/// Built with AVX2, BMI1, BMI2, LZCNT and POPCNT enabled (as Miri's AVX2 pass is), the
/// tests above check the AVX2 front and fill, not only the scalar ones
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi1",
    target_feature = "bmi2",
    target_feature = "lzcnt",
    target_feature = "popcnt"
))]
#[test]
fn miri_runs_avx2_kernels_selected() {
    let k = Kernels::detect();
    assert!(k.avx2 && front_fast(k));
    assert_eq!(kernels().len(), 2);
}
