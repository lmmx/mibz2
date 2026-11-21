// The slab queues (`construct_slab`, Carbon cycle 11's `FlatQ`) against the `Vec` queues they
// replace: the same table on arbitrary columns, the partition's bound met with equality on
// columns built to reach it, and counts that are not the records' sent safely to the `Vec`s.
use super::*;

/// A deterministic generator (an LCG), as `b5.rs`'
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

/// A column of `n` rows: runs of length 1 to `2 * mean` over `alphabet` bytes
fn runs_column(g: &mut Lcg, n: usize, alphabet: u32, mean: u32) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    while l.len() < n {
        let c = g.below(alphabet) as u8;
        let w = 1 + g.below(2 * mean) as usize;
        l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
    }
    l
}

/// The table a construction leaves: its edges, codes, words and marked row
type Table = (Vec<u32>, Vec<u32>, Vec<u8>, Vec<u32>, u32, u32);

fn table(k: &Kernel, mark: u32, bits: u32) -> Table {
    (
        k.edges.clone(),
        k.extra.clone(),
        k.spelling.clone(),
        k.offsets.clone(),
        mark,
        bits,
    )
}

/// The `Vec` queues' table for column `l` at row `p`, and the slab's from `meta` (None if
/// the slab refused the block, which is then also checked to have been built by the `Vec`s)
fn both(
    v: &mut Kernel,
    s: &mut Kernel,
    l: &[u8],
    p: u32,
    limit: u32,
    meta: Meta,
) -> (Table, Option<Table>) {
    let records = records_of(l);
    let n = l.len() as u32;
    let (vc, mark, bits) = v.construct::<true, true>(&records, None, n, p, limit);
    let reference = table(v, mark, bits);
    let slab = s
        .construct_slab::<true>(&records, meta, n, p, limit)
        .map(|(sc, mark, bits)| {
            assert_eq!(
                (sc.retained, sc.thick, sc.bands),
                (vc.retained, vc.thick, vc.bands)
            );
            table(s, mark, bits)
        });
    // and through `construct`, which falls back where the slab refuses
    let (_, mark, bits) = s.construct::<true, true>(&records, Some(meta), n, p, limit);
    assert_eq!(table(s, mark, bits), reference);
    (reference, slab)
}

/// The retained rows of each byte of `l` at row `p` and `limit`, counted from the column
fn thin_of(l: &[u8], p: usize, limit: usize) -> [u32; 256] {
    let mut thin = [0u32; 256];
    let mut i = 0;
    while i < l.len() {
        let mut end = i + 1;
        while end < l.len() && l[end] == l[i] {
            end += 1;
        }
        if end - i <= limit {
            thin[usize::from(l[i])] += (end - i) as u32;
        } else if (i..end).contains(&p) {
            thin[usize::from(l[i])] += 1;
        }
        i = end;
    }
    thin
}

/// Arbitrary columns (BWTs or not), rows and limits, across the 16-row store and the record's
/// 127: the slab takes every block its true counts describe, and its table is the `Vec`
/// queues', edge for edge
#[test]
fn slab_matches_vec_queues_on_arbitrary_columns() {
    let mut g = Lcg(0x666c_6174);
    let (mut v, mut s) = (Kernel::default(), Kernel::default());
    let mut cases = 0;
    for &n in &[1usize, 2, 17, 127, 128, 129, 1000, 4097, 70_000] {
        for &alphabet in &[1u32, 2, 3, 17, 256] {
            for &mean in &[1u32, 3, 8, 16, 17, 64, 130, 400] {
                let l = runs_column(&mut g, n, alphabet, mean);
                let (counts, per) = meta_of(&records_of(&l));
                let meta = Meta {
                    counts: &counts,
                    per: &per,
                };
                for p in [0, n as u32 - 1, n as u32 / 2, g.below(n as u32)] {
                    for limit in [0, 1, 4, 15, 16, 17, 40, 127, 128, 1000] {
                        let (reference, slab) = both(&mut v, &mut s, &l, p, limit, meta);
                        assert_eq!(
                            slab.as_ref(),
                            Some(&reference),
                            "n {n}, alphabet {alphabet}, mean {mean}, row {p}, limit {limit}"
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    println!("{cases} columns, rows and limits: the slab's table is the Vec queues'");
}

/// Columns built to drive a byte's retained rows to the partition's bound: thin runs of
/// exactly `limit` rows (`limit * per` = `counts`), the same with the marked row in the
/// byte's one thick run (`limit * (per - 1) + 1`), single rows (`counts` = `per`), and at a
/// limit of 0 the marked row alone. Each byte's stores end on its room's last slot or before,
/// the bound is met with equality where the column is built to meet it, and the table is the
/// `Vec` queues'
#[test]
fn slab_bound_met_with_equality() {
    let (mut v, mut s) = (Kernel::default(), Kernel::default());
    let mut tight = 0;
    for limit in [1usize, 2, 15, 16, 17, 127] {
        for runs in [1usize, 2, 7, 300] {
            let mut shapes: Vec<(Vec<u8>, usize)> = Vec::new();
            // byte 0 in thin runs of `limit`, each between single rows of 1..=3
            let mut l = Vec::new();
            for r in 0..runs {
                l.extend(std::iter::repeat_n(0u8, limit));
                l.push(1 + (r % 3) as u8);
            }
            let p = l.len() / 2;
            shapes.push((l.clone(), p));
            // and one thick run of byte 0 holding the marked row
            let at = l.len();
            l.extend(std::iter::repeat_n(0u8, limit + 1 + runs));
            shapes.push((l.clone(), at + runs / 2));
            // single rows of 256 bytes, each its own record
            let l: Vec<u8> = (0..runs * 256).map(|i| (i % 256) as u8).collect();
            shapes.push((l, runs));
            for (l, p) in shapes {
                let (counts, per) = meta_of(&records_of(&l));
                let meta = Meta {
                    counts: &counts,
                    per: &per,
                };
                let room = room(meta, limit as u32);
                let thin = thin_of(&l, p, limit);
                for a in 0..256 {
                    if thin[a] > 0 {
                        assert!(thin[a] + BLOCK as u32 - 1 <= room[a]);
                        tight += usize::from(thin[a] + BLOCK as u32 - 1 == room[a]);
                    }
                }
                let (reference, slab) = both(&mut v, &mut s, &l, p as u32, limit as u32, meta);
                assert_eq!(slab, Some(reference), "limit {limit}, runs {runs}");
            }
        }
    }
    // at a limit of 0 only the marked row is retained: one slot in its byte's room
    let l = vec![5u8; 300];
    let (counts, per) = meta_of(&records_of(&l));
    let meta = Meta {
        counts: &counts,
        per: &per,
    };
    assert!(room(meta, 0)[5] >= 1 + BLOCK as u32 - 1);
    let (reference, slab) = both(&mut v, &mut s, &l, 150, 0, meta);
    assert_eq!(slab, Some(reference));
    assert!(tight > 100, "{tight} bytes met the bound with equality");
}

/// Counts that are not the records' (a byte's records or rows understated, rows moved from
/// one byte to another, a total that is not the block's): the slab refuses the block where its
/// partition would not hold, never writes past a byte's room, and the block is built exactly,
/// by the `Vec`s where the slab refused it
#[test]
fn slab_falls_back_on_wrong_counts() {
    let mut g = Lcg(0x6261_6463);
    let (mut v, mut s) = (Kernel::default(), Kernel::default());
    let mut refused = 0;
    for &n in &[64usize, 1000, 4097, 70_000] {
        for &alphabet in &[2u32, 17, 256] {
            for &mean in &[1u32, 8, 16, 64] {
                let l = runs_column(&mut g, n, alphabet, mean);
                let (counts, per) = meta_of(&records_of(&l));
                let p = g.below(n as u32);
                let mut forged = Vec::new();
                let mut a = (counts, per);
                a.1 = per.map(|r| r.min(1));
                forged.push(a);
                let mut b = (counts, per);
                b.0 = counts.map(|w| w.min(1));
                forged.push(b);
                let mut c = (counts, per);
                let (hi, lo) = (
                    (0..256).max_by_key(|&x| counts[x]).unwrap(),
                    (0..256)
                        .filter(|&x| counts[x] > 0)
                        .min_by_key(|&x| counts[x])
                        .unwrap(),
                );
                if hi != lo {
                    let moved = counts[hi] / 2;
                    c.0[hi] -= moved;
                    c.0[lo] += moved;
                    forged.push(c);
                }
                let mut d = (counts, per);
                d.0[0] += 1;
                forged.push(d);
                for (fc, fp) in &forged {
                    let meta = Meta {
                        counts: fc,
                        per: fp,
                    };
                    let (reference, slab) = both(&mut v, &mut s, &l, p, LIMIT, meta);
                    match slab {
                        Some(t) => assert_eq!(t, reference),
                        None => refused += 1,
                    }
                }
            }
        }
    }
    assert!(refused > 20, "{refused} blocks refused");
}

/// (ignored; `MIBZ2_ADMITTED_BWT`, as `fused.rs`') the six admitted Wikidata columns at their
/// primary index: the slab takes each, its table is the `Vec` queues', and the kernel's output
/// is the plain inverse's
#[test]
#[ignore]
fn slab_admitted_blocks() {
    use crate::bwt::{InverseBwt, Plain};
    let dir = std::env::var("MIBZ2_ADMITTED_BWT").expect("MIBZ2_ADMITTED_BWT");
    let manifest = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let (mut v, mut s) = (Kernel::default(), Kernel::default());
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
        let r = Runs::of_column(&l);
        let (reference, slab) = both(&mut v, &mut s, &l, p, LIMIT, r.meta());
        assert_eq!(slab, Some(reference), "{file}");
        let mut counts = [0u32; 256];
        for &c in &l {
            counts[c as usize] += 1;
        }
        let (mut want, mut got) = (Vec::new(), Vec::new());
        Plain::default().invert(&l, &counts, p, &mut want);
        s.run::<true, true>(r.records(), Some(r.meta()), l.len(), p, LIMIT, &mut got);
        assert_eq!(got, want, "{file}");
        let room = room(r.meta(), LIMIT);
        eprintln!(
            "{file}: n {}, p {p}: slab {} slots ({} KiB), retained {}; exact",
            l.len(),
            room.iter().sum::<u32>(),
            room.iter().sum::<u32>() / 256,
            thin_of(&l, p as usize, LIMIT as usize).iter().sum::<u32>(),
        );
        blocks += 1;
    }
    assert_eq!(blocks, 6);
}

/// (ignored; timing, `MIBZ2_ADMITTED_BWT`) best of 41 per admitted block, interleaved, hot:
/// the kernel with its queues in the slab against the `Vec` queues, in the same build and
/// process (both with the lazy rows and the primary split)
#[test]
#[ignore]
fn slab_against_vec_timing() {
    use std::time::Instant;
    let dir = std::env::var("MIBZ2_ADMITTED_BWT").expect("MIBZ2_ADMITTED_BWT");
    let manifest = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let (mut v, mut s) = (Kernel::default(), Kernel::default());
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
        let r = Runs::of_column(&l);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        let (mut tv, mut ts) = (f64::INFINITY, f64::INFINITY);
        for i in 0..82 {
            let t = Instant::now();
            if i % 2 == 0 {
                v.run::<true, true>(r.records(), None, l.len(), p, LIMIT, &mut a);
                tv = tv.min(t.elapsed().as_secs_f64() * 1e6);
            } else {
                s.run::<true, true>(r.records(), Some(r.meta()), l.len(), p, LIMIT, &mut b);
                ts = ts.min(t.elapsed().as_secs_f64() * 1e6);
            }
        }
        assert_eq!(a, b);
        eprintln!(
            "{file}: vec {tv:.0} us, slab {ts:.0} us, slab/vec {:.3}",
            ts / tv
        );
    }
}
