// 12-byte rules (`W12LazyCells`, a bench arm; the second ChatGPT relay's §5 comparator, #145
// LOG 6924013198, ruled to its induction side in the Coordinator's message of 16 November): the
// lazy induction unchanged, but a word kept as a rule is stored in 12 bytes, not 16. Included in
// `cells.rs`.
//
// **A rule** is its two children, each packed in 48 bits: a 27-bit reference, then the length
// and the mark as 21 bits (length << 1 | mark, the descriptor's low half rotated by one). A
// child's reference is a rule's index if the child is a rule (longer than `SMALL`, or marked
// and longer than one byte), else its offset in the arena. So a rule's word (`cw`'s upper half)
// is its index, `(rp − rules) / 12`, an exact division by a multiply. Packing is a shift, a
// rotate, a shift and an OR, with no mask; a rule is two 8-byte stores, the second at 6 bytes
// (its last 2 bytes, zero, overwritten by the next rule's first store). The induction keeps no
// new value live.
//
// **The writer** (`write_w12`) is `write_cells` on these rules: the replay harness's W12, which
// wrote every Wikidata block as `write_cells` does at 1.01 to 1.03 of its time (#503).
//
// **Bounds:** a length must fit 20 bits and a reference 27. Blocks of 2^20 bytes or more, or a
// buffer past 2^27 bytes, are refused (`Refusal::Invalid`) and decoded by the fallback; no
// bzip2 block is that large.

/// The kernel `CellsVariant<V>` runs for V = `LAZY_W12`: lazy letters, 12-byte rules
pub(crate) const LAZY_W12: u8 = 3;

/// Lazy letters with 12-byte rules (a bench arm; the same output as `LazyCells`)
#[doc(hidden)]
pub type W12LazyCells = CellsVariant<LAZY_W12>;
/// The same with `ForcedCells`' open admission, for tests and fuzzing
#[doc(hidden)]
pub type ForcedW12LazyCells = CellsVariant<LAZY_W12, true>;

/// Whether a variant's first stage and induction are the lazy letters'
#[inline(always)]
fn lazy_variant(v: u8) -> bool {
    v == LAZY || v == LAZY_W12 || v == LAZY_WIDE
}

/// A 12-byte rule's stride; where its second child starts
const RULE12: usize = 12;
const C12_B: usize = 6;
/// A child: its reference's bits; its length and mark, above them
const C12_REF: u64 = (1 << 27) - 1;
const C12_LM: u32 = 27;
/// The longest length a child holds
const C12_LEN: u64 = (1 << 20) - 1;
/// The largest buffer whose offsets fit a reference
const C12_REACH: usize = 1 << 27;
/// 1/3 mod 2^32: (4k · 3)/4 · this = k, so a multiple of 12 divides by a shift and a multiply
const THIRD: u32 = 0xAAAA_AAAB;

/// A descriptor (`cw`: the word, the length with the mark) as a 12-byte rule's child
#[inline(always)]
fn pack12(w: u64) -> u64 {
    (w >> 32) | ((w as u32).rotate_left(1) as u64) << C12_LM
}

/// A child's length with its mark, as a descriptor's low half
#[cfg(test)]
fn unpack12(c: u64) -> u64 {
    (c & C12_REF) << 32 | ((c >> C12_LM) as u32).rotate_right(1) as u64
}

/// A rule's index from its offset past the rules' start (a multiple of 12)
#[inline(always)]
fn rule12_index(off: usize) -> usize {
    ((off >> 2) as u32).wrapping_mul(THIRD) as usize
}

impl Kernel {
    /// `write_cells` on 12-byte rules: the block from the induction's word (`word`, a rule's
    /// index, of marked length `len`), into `out`
    #[inline(never)]
    fn write_w12(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {
        let p = (len & !MARK) as usize;
        out.resize(n + 32, 0);
        let dst = out.as_mut_ptr();
        let base = self.buf.as_mut_ptr();
        let ro = self.lay.rules;
        let nr = (self.rule_end - ro) / RULE12;
        // a child's reference, its length, whether it is marked
        let at = |c: u64| (c & C12_REF) as usize;
        let ln = |c: u64| (c >> (C12_LM + 1)) as usize;
        let marked = |c: u64| (c >> C12_LM) & 1 != 0;
        // SAFETY: as `write_cells`. A rule's children are words made before it: a rule's
        // index below its own, or a word kept as bytes at its arena offset with 16 bytes
        // after it. A position is below the period; `out` has 16 bytes past it. A rule's 12
        // bytes lie in the rules' region, read as 8 at its start and 8 at 4 bytes in.
        unsafe {
            let rule = |j: usize| {
                let r = ro + RULE12 * j;
                (rd64(base, r) & ((1 << 48) - 1), rd64(base, r + 4) >> 16)
            };
            // each rule's first place: in the dead top slots, as `write_cells` keeps them,
            // else in a vector of the kernel's own
            let skip = base.add(self.lay.tops).align_offset(4);
            let pos = if skip < 4 && skip + 4 * nr <= self.lay.ends[0] - self.lay.tops {
                let pos = base.add(self.lay.tops + skip) as *mut u32;
                core::ptr::write_bytes(pos, 0xFF, nr);
                pos
            } else {
                if self.pos.capacity() < nr {
                    self.pos = Vec::new();
                    self.pos = vec![NIL; nr];
                } else {
                    self.pos.resize(nr, NIL);
                    self.pos.fill(NIL);
                }
                self.pos.as_mut_ptr()
            };
            // the row's place: down the marked rules to the row's byte, the one marked word
            // of one byte
            let root = pack12((word as u64) << 32 | len as u64);
            let (mut w, mut j) = (root, 0usize);
            while ln(w) != 1 {
                let (a, b) = rule(at(w));
                if marked(a) {
                    w = a;
                } else {
                    j += ln(a);
                    w = b;
                }
            }
            let start = (j + SHIFT) % p;
            // the marked rules: each short factor written where it lies, each long one's place
            // noted
            let (mut w, mut q) = (root, 0usize);
            while ln(w) != 1 {
                let (a, b) = rule(at(w));
                let la = ln(a);
                let (plain, pq, next) = if marked(a) { (b, q + la, a) } else { (a, q, b) };
                let d = (pq + p - start) % p;
                let lx = ln(plain);
                if lx <= SMALL as usize {
                    copy_small(dst.add(d), base.add(at(plain)), lx);
                } else {
                    let x = pos.add(at(plain));
                    *x = (*x).min(d as u32);
                }
                if !marked(a) {
                    q += la;
                }
                w = next;
            }
            *dst.add((q + p - start) % p) = *base.add(at(w));
            // where each rule first lies, then its bytes
            for j in (0..nr).rev() {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(j);
                let la = ln(a);
                if la > SMALL as usize {
                    let x = pos.add(at(a));
                    *x = (*x).min(q);
                }
                if ln(b) > SMALL as usize {
                    let x = pos.add(at(b));
                    *x = (*x).min(q + la as u32);
                }
            }
            for j in 0..nr {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(j);
                let (ll, rr) = (ln(a), ln(b));
                let d = dst.add(q as usize);
                if ll <= SMALL as usize {
                    copy_small(d, base.add(at(a)), ll);
                } else {
                    let f = *pos.add(at(a));
                    if f != q {
                        copy_long(d, dst.add(f as usize), ll);
                    }
                }
                let d = d.add(ll);
                if rr <= SMALL as usize {
                    copy_small(d, base.add(at(b)), rr);
                } else {
                    let f = *pos.add(at(b));
                    if f != q + ll as u32 {
                        copy_long(d, dst.add(f as usize), rr);
                    }
                }
            }
            // the marked rules again: a long factor not at its first place, copied from it
            let (mut w, mut q) = (root, 0usize);
            while ln(w) != 1 {
                let (a, b) = rule(at(w));
                let la = ln(a);
                let (plain, pq, next) = if marked(a) { (b, q + la, a) } else { (a, q, b) };
                let d = (pq + p - start) % p;
                let lx = ln(plain);
                if lx > SMALL as usize {
                    let f = *pos.add(at(plain)) as usize;
                    if f != d {
                        copy_long(dst.add(d), dst.add(f), lx);
                    }
                }
                if !marked(a) {
                    q += la;
                }
                w = next;
            }
        }
        // a cycle shorter than the block: the block is its copies
        let mut have = p.min(n);
        while have < n {
            let more = have.min(n - have);
            out.copy_within(..more, have);
            have += more;
        }
        out.truncate(n);
    }
}

#[cfg(test)]
mod w12_tests {
    use super::*;

    struct Lcg(u64);
    impl Lcg {
        fn below(&mut self, m: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) % m
        }
    }

    fn records(col: &[u8]) -> Vec<u16> {
        let mut recs: Vec<u16> = Vec::new();
        for &c in col {
            match recs.last_mut() {
                Some(last) if *last >> 8 == c as u16 && *last & 0xFF < 127 => *last += 1,
                _ => recs.push((c as u16) << 8 | 1),
            }
        }
        recs
    }

    /// The packing is exact for every length, mark and reference in bounds
    #[test]
    fn packs_and_unpacks() {
        for r in [0u64, 1, 12345, (1 << 27) - 1] {
            for l in [1u64, 2, 16, 17, 900_000, (1 << 20) - 1] {
                for m in [0u64, 1 << 31] {
                    let d = r << 32 | m | l;
                    assert_eq!(unpack12(pack12(d)), d);
                    assert!(pack12(d) < 1 << 48);
                }
            }
        }
        for k in [0usize, 1, 2, 1000, 1 << 22] {
            assert_eq!(rule12_index(RULE12 * k), k);
        }
    }

    /// Arbitrary columns, most of them no BWT: 12-byte rules give the lazy kernel's result,
    /// refusal or bytes, on every one
    #[test]
    fn arbitrary_columns_as_the_lazy_kernel() {
        let mut rng = Lcg(0x12_12);
        let mut lazy = Kernel {
            variant: LAZY,
            ..Kernel::default()
        };
        let mut w12 = Kernel {
            variant: LAZY_W12,
            ..Kernel::default()
        };
        let mut admitted = 0;
        for _ in 0..4000 {
            let n = 1 + rng.below(800) as usize;
            let sigma = 1 + rng.below(5);
            let col: Vec<u8> = (0..n).map(|_| b'a' + rng.below(sigma) as u8).collect();
            let p = rng.below(n as u64) as u32;
            let recs = records(&col);
            let mut want = Vec::new();
            let w = lazy.try_invert(&recs, n, p, n, true, &mut want);
            let mut got = Vec::new();
            let g = w12.try_invert(&recs, n, p, n, true, &mut got);
            assert_eq!(g.is_ok(), w.is_ok(), "n {n} p {p}");
            if let (Ok(a), Ok(b)) = (w, g) {
                assert_eq!(a, b, "n {n} p {p}");
                assert!(got[..n] == want[..n], "n {n} p {p}");
                admitted += 1;
            }
        }
        assert!(admitted > 1000, "{admitted}");
    }
}
