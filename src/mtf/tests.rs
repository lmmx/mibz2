//! C29's register-list checkpoint kernel, now `mtf_checkpoints`' AVX2 kernel, against
//! `mtf_checkpoints_mem` (the earlier memory-list kernel, kept as the reference) and the
//! scalar twin: every checkpoint and raw-store class, valid and
//! malformed, each kernel; and under Miri, its AVX2 stores. Ported from #270's research
//! (research/run-fed-leader, `parallel::run_team::tests`, Guardian D049 and D054;
//! docs/journal/2026-11-09-c29-register-checkpoints.md).

use super::*;

fn xorshift(mut x: u64) -> impl FnMut() -> u64 {
    move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    }
}

/// A column of runs (lengths from `lens`) over `sigma` bytes
fn runs_column(len: usize, sigma: u64, lens: &[usize], seed: u64) -> Vec<u8> {
    let mut r = xorshift(seed);
    let mut ll = Vec::new();
    while ll.len() < len {
        let c = (r() % sigma) as u8;
        let run = lens[(r() % lens.len() as u64) as usize];
        ll.extend(std::iter::repeat(c).take(run));
    }
    ll.truncate(len);
    ll
}

/// A column's block symbols (every byte in use, in order), bzip2's MTF and RLE2
pub(crate) fn symbols_of(ll: &[u8]) -> Symbols {
    let mut list: Vec<u8> = (0..=255).collect();
    let mut syms = Vec::new();
    let mut run = 0u32;
    let flush = |run: &mut u32, syms: &mut Vec<u16>| {
        if *run > 0 {
            let mut z = *run - 1;
            loop {
                syms.push((z & 1) as u16);
                if z < 2 {
                    break;
                }
                z = (z - 2) / 2;
            }
            *run = 0;
        }
    };
    for &c in ll {
        let j = list.iter().position(|&x| x == c).unwrap();
        if j == 0 {
            run += 1;
            continue;
        }
        flush(&mut run, &mut syms);
        list.remove(j);
        list.insert(0, c);
        syms.push(j as u16 + 1);
    }
    flush(&mut run, &mut syms);
    syms.push(257);
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq: std::array::from_fn(|i| i as u8),
        syms,
        eob: 257,
        max_len: 900_000,
    }
}

/// The scalar kernels, and the detected ones where they differ
fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    v
}

/// The input classes a symbol stream reaches in `mtf`'s semantics (`classify`), each one
/// a branch of `mtf_checkpoints_reg_avx2`: a run split at a checkpoint, a run ending at
/// one, a byte written at one, a block ending at one; a run of 64 or fewer by the two
/// 32-byte stores, or by the byte store at the block's bound (n + 64 past `max_len`), or a
/// longer run; and each error and where it fires
#[derive(Debug, Default)]
struct Reach {
    run_split: u64,
    run_to_checkpoint: u64,
    byte_at_checkpoint: u64,
    end_at_checkpoint: u64,
    run_vector: u64,
    run_bytes_at_bound: u64,
    run_long: u64,
    ok: u64,
    too_long_in_run: u64,
    too_long_at_byte: u64,
    run_too_long: u64,
    no_eob: u64,
}

impl Reach {
    fn each(&self) -> [(&'static str, u64); 12] {
        [
            ("run split at a checkpoint", self.run_split),
            ("run ending at a checkpoint", self.run_to_checkpoint),
            ("byte at a checkpoint", self.byte_at_checkpoint),
            ("block ending at a checkpoint", self.end_at_checkpoint),
            ("run by the vector stores", self.run_vector),
            (
                "run of <= 64 by bytes at the bound",
                self.run_bytes_at_bound,
            ),
            ("run of > 64", self.run_long),
            ("Ok", self.ok),
            ("BlockTooLong in a run", self.too_long_in_run),
            ("BlockTooLong at a byte", self.too_long_at_byte),
            ("RunTooLong", self.run_too_long),
            ("no end-of-block symbol", self.no_eob),
        ]
    }
}

/// `b`'s classes in `mtf`'s semantics (an independent model of the input, not of the
/// kernel): the length so far, each run and byte as it lands, the bound checked
fn classify(b: &Symbols, r: &mut Reach) {
    // (MAX_RUN_WEIGHT, restated)
    const WEIGHT: u64 = 2 * 1024 * 1024;
    let (cp, max) = (CHECKPOINT, b.max_len);
    let (mut n, mut run, mut w) = (0usize, 0u64, 1u64);
    for &s in &b.syms {
        if s <= 1 {
            if w >= WEIGHT {
                r.run_too_long += 1;
                return;
            }
            run += (s as u64 + 1) * w;
            w <<= 1;
            continue;
        }
        if run != 0 {
            let len = run as usize;
            if n + len > max {
                r.too_long_in_run += 1;
                return;
            }
            let end = n + len;
            r.run_split += ((n / cp + 1) * cp < end) as u64;
            r.run_to_checkpoint += (end % cp == 0) as u64;
            if len > 64 {
                r.run_long += 1;
            } else if n + 64 <= max {
                r.run_vector += 1;
            } else {
                r.run_bytes_at_bound += 1;
            }
            (n, run, w) = (end, 0, 1);
        }
        if s == b.eob {
            r.ok += 1;
            r.end_at_checkpoint += (n > 0 && n % cp == 0) as u64;
            return;
        }
        if n >= max {
            r.too_long_at_byte += 1;
            return;
        }
        r.byte_at_checkpoint += (n > 0 && n % cp == 0) as u64;
        n += 1;
    }
    r.no_eob += 1;
}

/// `mtf_checkpoints_reg` against `mtf_checkpoints` on `b`, each kernel: the result, and
/// the column and counts (on errors too: both are `mtf`'s on every input), the
/// checkpoints where it succeeds, from buffers holding another block's
fn same_checkpoints(b: &Symbols, r: &mut Reach) {
    classify(b, r);
    for k in kernels() {
        let (mut ll, mut c, mut p) = (Vec::new(), [0; 256], Vec::new());
        let want = mtf_checkpoints_mem(b, &mut ll, &mut c, &mut p, k);
        let (mut ll2, mut c2, mut p2) = (vec![1, 2, 3], [9; 256], vec![[3; 256]; 2]);
        let got = mtf_checkpoints(b, &mut ll2, &mut c2, &mut p2, k);
        let what = format!("{} symbols, max_len {}, {k:?}", b.syms.len(), b.max_len);
        assert_eq!(got, want, "{what}: the result");
        assert!(ll == ll2, "{what}: the column");
        assert!(c == c2, "{what}: the counts");
        if want.is_ok() {
            assert!(p == p2, "{what}: the checkpoints");
        }
        // and the scalar twin, the definition
        let (mut ll3, mut c3, mut p3) = (vec![5; 7], [1; 256], Vec::new());
        let scalar = mtf_checkpoints_scalar(b, &mut ll3, &mut c3, &mut p3);
        assert_eq!(got, scalar, "{what}: the scalar result");
        assert!(
            ll3 == ll2 && c3 == c2,
            "{what}: the scalar column and counts"
        );
        if scalar.is_ok() {
            assert!(p3 == p2, "{what}: the scalar checkpoints");
        }
    }
}

/// A column's symbols at its length's bounds about `d` (BlockTooLong below its length,
/// the byte store at the bound just above it)
fn at_bounds(ll: &[u8], ds: &[isize], out: &mut Vec<Symbols>) {
    for &d in ds {
        let mut b = symbols_of(ll);
        b.max_len = (ll.len() as isize + d).max(1) as usize;
        out.push(b);
    }
}

/// The columns whose runs and bytes land on and about the checkpoints: a run to a
/// checkpoint, a run across one and two, a byte at one (and a block's last byte there),
/// runs of 63, 64 and 65 at the bound
fn checkpoint_columns() -> Vec<Vec<u8>> {
    let cp = CHECKPOINT;
    let mut v = vec![
        vec![7u8; cp],
        vec![7u8; cp + 100],
        [vec![1u8], vec![2u8; 2 * cp - 1]].concat(),
        [vec![1u8; cp - 1], vec![2u8], vec![3u8; 40]].concat(),
        [vec![1u8; cp], vec![2u8; 3]].concat(),
        [vec![7u8; cp], vec![8u8]].concat(),
    ];
    for tail in [63usize, 64, 65] {
        v.push([vec![4u8; cp - 30], vec![5u8], vec![6u8; tail]].concat());
    }
    // a run across one checkpoint that ends exactly on the next, then a byte (port 3's
    // mutant M2, the split loop's `>=` made `>`, survived without it), and the same run
    // ending the block there
    v.push([vec![1u8; 10], vec![2u8; 2 * cp - 10], vec![3u8]].concat());
    v.push([vec![1u8; 10], vec![2u8; 2 * cp - 10]].concat());
    v.push([vec![1u8; 10], vec![2u8; 3 * cp - 10], vec![3u8, 4]].concat());
    v
}

/// The longest runs' weights: 22 RUNA symbols, `RunTooLong`
fn run_too_long() -> Symbols {
    let mut long = symbols_of(&[1, 2]);
    long.syms = [vec![0u16; 22], vec![257]].concat();
    long
}

/// C29 is `mtf_checkpoints` on every column and error: its result, column, counts and
/// checkpoints, at bounds about each length
#[test]
fn register_checkpoints_are_checkpoints() {
    let cp = CHECKPOINT;
    let mut cases: Vec<Symbols> = Vec::new();
    for (len, sigma, lens) in [
        (3 * cp, 5u64, &[1usize, 2, 3, 64, 65, 200][..]),
        (2 * cp + 17, 200, &[1, 2][..]),
        (cp - 1, 3, &[40, 1000][..]),
        (cp, 2, &[cp / 2 + 3][..]),
        (5 * cp + 9, 1, &[cp + 5][..]),
        (100, 7, &[1][..]),
    ] {
        let ll = runs_column(len, sigma, lens, len as u64);
        // the bound at, just above and below the length: BlockTooLong where it fails
        at_bounds(&ll, &[900_000 - len as isize, 0, -1, -64, 1], &mut cases);
    }
    cases.push(run_too_long());
    let mut reach = Reach::default();
    for b in &cases {
        same_checkpoints(b, &mut reach);
    }
}

/// C29 on the inputs about its checkpoints and its raw stores, valid and not, each
/// reaching every class (`Reach`), against `mtf_checkpoints`: seeded (xorshift), bounded
/// (2,811 streams), printed
#[test]
fn register_checkpoints_reach_every_class() {
    let cp = CHECKPOINT;
    let mut cases: Vec<Symbols> = Vec::new();
    let bounds = [-65isize, -64, -63, -1, 0, 1, 31, 63, 64, 65];
    for ll in checkpoint_columns() {
        at_bounds(&ll, &bounds, &mut cases);
    }
    let mut r = xorshift(0xc29);
    let lens = [
        1usize,
        2,
        3,
        31,
        32,
        33,
        63,
        64,
        65,
        100,
        1000,
        cp / 2 + 3,
        cp + 5,
    ];
    for i in 0..120 {
        let len = [cp - 1, cp, cp + 1, 2 * cp, 2 * cp + 63, 3 * cp + 5][i % 6];
        let sigma = [1u64, 2, 3, 9, 200][i % 5];
        let ll = runs_column(len, sigma, &lens, r());
        at_bounds(&ll, &bounds[(i % 4)..(i % 4) + 6], &mut cases);
    }
    // malformed streams: random symbols, a random alphabet and bound, some without their
    // end-of-block symbol; and the longest runs' weights
    for i in 0..2_000 {
        let eob = 2 + (r() % 256) as u16;
        let len = (r() % (3 * cp as u64)) as usize;
        let run_heavy = i % 3 == 0;
        let mut syms: Vec<u16> = (0..len)
            .map(|_| {
                if run_heavy && r() % 4 != 0 {
                    (r() % 2) as u16
                } else {
                    (r() % eob as u64) as u16
                }
            })
            .collect();
        if i % 50 != 7 {
            syms.push(eob);
        }
        let mut perm: Vec<u8> = (0..=255).collect();
        for j in (1..256).rev() {
            perm.swap(j, (r() % (j as u64 + 1)) as usize);
        }
        cases.push(Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: perm.try_into().unwrap(),
            syms,
            eob,
            max_len: [cp, 2 * cp, 3 * cp + 7, 900_000][i % 4] - (r() % 70) as usize,
        });
    }
    cases.push(run_too_long());
    let mut reach = Reach::default();
    for b in &cases {
        same_checkpoints(b, &mut reach);
    }
    eprintln!("C29: {} streams, kernels {:?}", cases.len(), kernels());
    for (what, n) in reach.each() {
        eprintln!("{n:>9}  {what}");
        assert!(n > 0, "class not reached: {what}");
    }
}

/// The same under Miri, for its AVX2 kernel's raw stores (CI's AVX2 step, `-C
/// target-feature=+avx2,+bmi1`, `MIBZ2_EXPECT_AVX2` set; without AVX2, C29 is
/// `mtf_checkpoints`' scalar twin itself, so there is nothing to check and the test
/// returns): one stream a class, on the detected kernels only, each column fresh so that
/// its capacity is exactly the bound and Miri sees any store past it
#[test]
fn miri_register_checkpoints() {
    if std::env::var_os("MIBZ2_EXPECT_AVX2").is_some() {
        assert!(Kernels::detect().avx2, "AVX2 not detected");
    }
    if cfg!(miri) && !Kernels::detect().avx2 {
        return;
    }
    let cp = CHECKPOINT;
    let mut cases: Vec<Symbols> = Vec::new();
    // a run to a checkpoint, ending the block there; a run across one; a block's last byte
    // at one, its bound just below (BlockTooLong at the byte); a run of 64 at the bound
    // (the byte store) and past it (BlockTooLong in the run)
    at_bounds(&vec![7u8; cp], &[0], &mut cases);
    at_bounds(&vec![7u8; cp + 100], &[0], &mut cases);
    at_bounds(&[vec![7u8; cp], vec![8u8]].concat(), &[-1, 0], &mut cases);
    let tail = [vec![4u8; cp - 30], vec![5u8], vec![6u8; 64]].concat();
    at_bounds(&tail, &[-1, 0], &mut cases);
    cases.push(run_too_long());
    let mut short = symbols_of(&[3, 3, 3, 4]);
    short.syms.pop();
    cases.push(short);
    let mut reach = Reach::default();
    let k = Kernels::detect();
    for b in &cases {
        classify(b, &mut reach);
        let (mut ll, mut c, mut p) = (Vec::new(), [0; 256], Vec::new());
        let want = mtf_checkpoints_mem(b, &mut ll, &mut c, &mut p, k);
        let (mut ll2, mut c2, mut p2) = (Vec::new(), [9; 256], vec![[3; 256]]);
        let got = mtf_checkpoints(b, &mut ll2, &mut c2, &mut p2, k);
        assert_eq!(got, want);
        assert!(ll == ll2 && c == c2);
        if want.is_ok() {
            assert!(p == p2);
        }
    }
    for (what, n) in reach.each() {
        assert!(n > 0, "class not reached: {what}");
    }
}
