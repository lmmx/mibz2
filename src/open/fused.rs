// The fused front (`Kernel::run_symbols`, `OpenCells::invert_symbols`): the construction's
// first pass taken from the front's literal walk, against the records' path, byte for byte.
use super::*;
use crate::block::Symbols;
use crate::bwt::{InverseBwt, Plain};

/// Whether this CPU runs the AVX2 front, the only one the fused path has: elsewhere every
/// block is left to the records' path, and the tests check that instead
fn fused_here() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        let k = Kernels::detect();
        k.avx2
            && std::is_x86_feature_detected!("bmi2")
            && std::is_x86_feature_detected!("lzcnt")
            && std::is_x86_feature_detected!("popcnt")
    }
    #[cfg(not(target_arch = "x86_64"))]
    false
}

/// The plain inverse of column `l` at row `p`: the oracle
fn plain(l: &[u8], p: u32) -> Vec<u8> {
    let mut counts = [0u32; 256];
    for &c in l {
        counts[c as usize] += 1;
    }
    let mut out = Vec::new();
    Plain::default().invert(l, &counts, p, &mut out);
    out
}

/// A column as a block's symbols at row `p`: its bytes in use, move-to-front indices, and
/// RUNA and RUNB for the zeros (as bzip2 codes them), under a bound of `max_len` rows
fn symbols_of(ll: &[u8], p: u32, max_len: usize) -> Symbols {
    let mut used = [false; 256];
    ll.iter().for_each(|&c| used[c as usize] = true);
    let mut seq_to_unseq = [0u8; 256];
    let mut list: Vec<u8> = (0..=255u8).filter(|&c| used[c as usize]).collect();
    seq_to_unseq[..list.len()].copy_from_slice(&list);
    let eob = list.len() as u16 + 1;
    let (mut syms, mut zeros) = (Vec::new(), 0u32);
    let flush = |zeros: &mut u32, syms: &mut Vec<u16>| {
        // (bijective base 2: RUNA 1, RUNB 2, weights doubling)
        while *zeros > 0 {
            syms.push(((*zeros - 1) & 1) as u16);
            *zeros = (*zeros - 1) >> 1;
        }
    };
    for &c in ll {
        let i = list.iter().position(|&x| x == c).unwrap();
        if i == 0 {
            zeros += 1;
            continue;
        }
        flush(&mut zeros, &mut syms);
        syms.push(i as u16 + 1);
        list.remove(i);
        list.insert(0, c);
    }
    flush(&mut zeros, &mut syms);
    syms.push(eob);
    Symbols {
        crc: 0,
        orig_ptr: p,
        seq_to_unseq,
        syms,
        eob,
        max_len,
    }
}

/// The block by the fused path (forced: whatever its record count), into an output poisoned
/// first; and by the records' path (`runs::front`, then the shipped `invert_runs`). Both must
/// be the plain inverse. Returns whether the fused path took it.
fn both(
    fused: &mut ForcedOpenCells,
    records: &mut ForcedOpenCells,
    l: &[u8],
    p: u32,
    what: &str,
) -> bool {
    let k = Kernels::detect();
    let want = plain(l, p);
    let b = symbols_of(l, p, l.len().max(1));
    let mut r = Runs::default();
    let mut out: Vec<u8> = want.iter().map(|b| !b).collect();
    let took = fused.invert_symbols(&b, &mut r, k, &mut out);
    if let Some(n) = took {
        assert_eq!(n, l.len(), "{what}: rows");
        assert!(out == want, "{what}: fused");
    }
    let n = crate::runs::front(&b, &mut r, k).unwrap();
    assert_eq!(n, l.len(), "{what}: the front");
    let mut out2: Vec<u8> = want.iter().map(|b| !b).collect();
    assert!(records.invert_runs(&r, n, p, &mut out2, k), "{what}");
    assert!(out2 == want, "{what}: records");
    took.is_some()
}

/// Every binary column through 11 rows and every ternary column through 6, at every row: the
/// fused path gives the plain inverse, as the records' path does, and takes every block where
/// the AVX2 front runs
#[test]
fn fused_every_small_column_at_every_row() {
    let (mut f, mut r) = (ForcedOpenCells::default(), ForcedOpenCells::default());
    let (mut cases, mut taken) = (0usize, 0usize);
    for (base, len) in [(2usize, 11usize), (3, 6)] {
        for n in 1..=len {
            for mut z in 0..base.pow(n as u32) {
                let l: Vec<u8> = (0..n)
                    .map(|_| {
                        let c = z % base;
                        z /= base;
                        // bytes far apart, so that the MTF indices are not the bytes
                        (c * 97 + 3) as u8
                    })
                    .collect();
                for p in 0..n as u32 {
                    taken += usize::from(both(&mut f, &mut r, &l, p, &format!("{l:?} at {p}")));
                    cases += 1;
                }
            }
        }
    }
    // the sum of n 2^n for n 1..=11 and of n 3^n for n 1..=6
    assert_eq!(cases, 40_962 + 6_015);
    assert_eq!(taken, if fused_here() { cases } else { 0 });
    assert_eq!(f.fused, taken);
}

/// A deterministic generator (an LCG)
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next() % n.max(1)
    }
}

/// Arbitrary columns of 1 to 256 bytes, runs short and long (past 127 rows, and past the
/// limit around the marked row), at the first, last, middle and random rows, through one pair
/// of kernels reused: the fused path is the records' path and the plain inverse
#[test]
fn fused_arbitrary_columns_through_one_kernel() {
    let mut g = Lcg(0x6675_7365);
    let (mut f, mut r) = (ForcedOpenCells::default(), ForcedOpenCells::default());
    let mut taken = 0;
    let mut cases = 0;
    for &n in &[1usize, 2, 17, 64, 127, 128, 129, 255, 1000, 4097, 70_000] {
        for &alphabet in &[1u32, 2, 3, 17, 40, 256] {
            for &mean in &[1u32, 3, 16, 17, 64, 130, 400, 5000] {
                let mut l = Vec::with_capacity(n);
                while l.len() < n {
                    let c = g.below(alphabet) as u8;
                    let w = 1 + g.below(2 * mean) as usize;
                    l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
                }
                for p in [0, n as u32 - 1, n as u32 / 2, g.below(n as u32)] {
                    let what = format!("n {n}, alphabet {alphabet}, mean {mean}, row {p}");
                    taken += usize::from(both(&mut f, &mut r, &l, p, &what));
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(taken, if fused_here() { cases } else { 0 });
}

/// `OpenCells` (unforced) takes a block by the fused path exactly where `invert_runs` would
/// have taken it by the open kernel (`LazyCells`' record-count gates), and leaves every other
/// block to the records' path, `out` untouched
#[test]
fn fused_admits_on_lazy_cells_count_gates() {
    let mut g = Lcg(0x6761_7465);
    let k = Kernels::detect();
    let mut o = OpenCells::default();
    let (mut admitted, mut refused) = (0, 0);
    for &n in &[2_000usize, 20_000, 120_000] {
        for &mean in &[1u32, 2, 4, 6, 8, 12, 40, 63, 64, 65, 100, 400] {
            let mut l = Vec::with_capacity(n);
            while l.len() < n {
                let c = g.below(40) as u8;
                let w = 1 + g.below(2 * mean) as usize;
                l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
            }
            let p = g.below(n as u32);
            let b = symbols_of(&l, p, n);
            let mut r = Runs::default();
            let n2 = crate::runs::front(&b, &mut r, k).unwrap();
            let gates = OpenCells::admits(r.records().len(), n2, p);
            let mut out = vec![7u8; 3];
            let took = o.invert_symbols(&b, &mut r, k, &mut out);
            if fused_here() {
                assert_eq!(took.is_some(), gates, "n {n}, mean {mean}");
            } else {
                assert!(took.is_none());
            }
            match took {
                Some(_) => {
                    assert!(out == plain(&l, p));
                    admitted += 1;
                }
                None => {
                    assert_eq!(out, [7, 7, 7]);
                    refused += 1;
                }
            }
        }
    }
    if fused_here() {
        assert!(admitted > 5 && refused > 5, "{admitted} {refused}");
    }
}

/// Where the fused path leaves a block to the records' path: a run longer than the rows the
/// block may have, a row `p` past the block, a byte in use twice (two runs of one byte in a
/// row), and the scalar kernels (no AVX2 front)
#[test]
fn fused_refusals() {
    let k = Kernels::detect();
    let mut f = ForcedOpenCells::default();
    let mut r = Runs::default();
    let mut out = Vec::new();
    let l: Vec<u8> = b"aaaabbbbbbbbbbbbbbbbbbbbbbbbbbbccdaaaaaaaaaaaaaaaaaaaaaaaa".to_vec();
    let ok = symbols_of(&l, 5, l.len());
    assert_eq!(
        f.invert_symbols(&ok, &mut r, k, &mut out).is_some(),
        fused_here()
    );
    // the bound one row short
    let mut b = symbols_of(&l, 5, l.len());
    b.max_len = l.len() - 1;
    assert!(f.invert_symbols(&b, &mut r, k, &mut out).is_none());
    // the row past the block
    let mut b = symbols_of(&l, 5, l.len());
    b.orig_ptr = l.len() as u32;
    assert!(f.invert_symbols(&b, &mut r, k, &mut out).is_none());
    // a byte in use twice: index 1 then names the open run's byte again
    let mut b = symbols_of(&l, 5, l.len());
    b.seq_to_unseq[1] = b.seq_to_unseq[0];
    assert!(f.invert_symbols(&b, &mut r, k, &mut out).is_none());
    // the scalar kernels
    assert!(f
        .invert_symbols(&ok, &mut r, Kernels::scalar(), &mut out)
        .is_none());
}

/// The front's runs (`runs::front_runs`) are its records before their cut at 127: on blocks of
/// every shape the front's own tests make (long runs, the cap's edges, errors), wherever the
/// fused walk completes, the front succeeds with the same rows and the runs cut at 127 are its
/// records exactly
#[test]
fn front_runs_are_the_records_uncut() {
    struct Collect(Vec<(u8, u32)>);
    impl crate::runs::RunVisitor for Collect {
        fn run(&mut self, c: u8, rows: u32) -> bool {
            self.0.push((c, rows));
            true
        }
    }
    let k = Kernels::detect();
    let mut g = Lcg(0x7275_6e73);
    let mut r = Runs::default();
    let mut done = 0;
    for case in 0..3000 {
        let n = 1 + g.below(if case % 10 == 0 { 300_000 } else { 3000 }) as usize;
        let alphabet = 1 + g.below(if case % 3 == 0 { 256 } else { 8 });
        let mean = [1u32, 4, 30, 126, 127, 128, 300, 20_000][g.below(8) as usize];
        let mut l = Vec::with_capacity(n);
        while l.len() < n {
            let c = g.below(alphabet) as u8;
            let w = 1 + g.below(2 * mean) as usize;
            l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
        }
        let b = symbols_of(&l, 0, n);
        let mut v = Collect(Vec::new());
        let some = crate::runs::front_runs(&b, &mut r, k, &mut v).is_some();
        assert_eq!(some, fused_here(), "case {case}");
        if !some {
            continue;
        }
        assert_eq!(crate::runs::front(&b, &mut r, k), Ok(n), "case {case}");
        let mut cut = Vec::new();
        for &(c, mut rows) in &v.0 {
            while rows > 0 {
                let len = rows.min(127);
                cut.push(u16::from(c) << 8 | len as u16);
                rows -= len;
            }
        }
        assert!(cut == r.records(), "case {case}");
        assert_eq!(v.0.iter().map(|x| x.1 as usize).sum::<usize>(), n);
        done += 1;
    }
    assert_eq!(done, if fused_here() { 3000 } else { 0 });
}

/// (ignored; `MIBZ2_ADMITTED_BWT` names the directory of the six admitted Wikidata columns,
/// `data/admitted-bwt`) each column at its primary index: the fused path, the records' path
/// and the plain inverse agree
#[test]
#[ignore]
fn fused_admitted_blocks() {
    let dir = std::env::var("MIBZ2_ADMITTED_BWT").expect("MIBZ2_ADMITTED_BWT");
    let manifest = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let (mut f, mut r) = (ForcedOpenCells::default(), ForcedOpenCells::default());
    let mut blocks = 0;
    for part in manifest.split("\"file\": \"").skip(1) {
        let file = &part[..part.find('"').unwrap()];
        let at = part.find("\"primary_index\": ").unwrap() + 17;
        let p: u32 = part[at..]
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let l = std::fs::read(format!("{dir}/{file}")).unwrap();
        assert!(both(&mut f, &mut r, &l, p, file) || !fused_here(), "{file}");
        eprintln!(
            "{file}: n {}, p {p}: fused, records and plain agree",
            l.len()
        );
        blocks += 1;
    }
    assert_eq!(blocks, 6);
}
