//! Research only (feature `open-emit-pipelined`; compiled for tests too): Open's `emit` as a
//! one-step software-pipelined walk, the next row's edge (and, wide, its code) loaded before the
//! current row's word is appended (docs/journal/2026-11-20-open-emit-pipeline.md). Never on a
//! production path: without the feature, `emit` is `emit_shipped`.
//!
//! Adapted from the proposal to the shipped walk's signature, loop bound and termination: at most
//! `edges.len()` rows appended; the walk stops after the row whose successor is the marked row, or
//! once `out` holds `n` bytes; an out-of-bounds successor panics only after the current row's word
//! is appended and both stopping conditions have been checked, as the shipped walk's next
//! iteration would. The panic message is the slice index's own.
use super::{append_code, repeat};

#[cold]
#[inline(never)]
fn out_of_bounds(index: usize, len: usize) -> ! {
    panic!("index out of bounds: the len is {len} but the index is {index}")
}

/// `emit`, pipelined: the same contract as `emit_shipped`
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_pipelined(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.clear();
    out.reserve(n);
    if extra.is_empty() {
        walk::<false>(edges, extra, spelling, offsets, n, mark, bits, out);
    } else {
        walk::<true>(edges, extra, spelling, offsets, n, mark, bits, out);
    }
    out.truncate(n);
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn walk<const WIDE: bool>(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    if edges.is_empty() {
        return;
    }
    let mask = (1 << bits) - 1;
    let first = edges[mark as usize];
    let mut code = if WIDE {
        extra[mark as usize]
    } else {
        first >> bits
    };
    let mut j = first & mask;
    for _ in 1..edges.len() {
        if j == mark {
            break;
        }
        let Some(&next_edge) = edges.get(j as usize) else {
            append_code(code, spelling, offsets, out);
            if out.len() >= n {
                return;
            }
            out_of_bounds(j as usize, edges.len());
        };
        let next_code = if WIDE {
            let Some(&c) = extra.get(j as usize) else {
                append_code(code, spelling, offsets, out);
                if out.len() >= n {
                    return;
                }
                out_of_bounds(j as usize, extra.len());
            };
            c
        } else {
            next_edge >> bits
        };
        let next_j = next_edge & mask;
        append_code(code, spelling, offsets, out);
        if out.len() >= n {
            return;
        }
        code = next_code;
        j = next_j;
    }
    append_code(code, spelling, offsets, out);
}

#[cfg(test)]
mod tests {
    use super::super::emit_shipped;
    use super::emit_pipelined;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    type Emit = fn(&[u32], &[u32], &[u8], &[u32], usize, u32, u32, &mut Vec<u8>);

    #[allow(clippy::too_many_arguments)]
    fn outcome(
        f: Emit,
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        offsets: &[u32],
        n: usize,
        mark: u32,
        bits: u32,
    ) -> Option<Vec<u8>> {
        // a dirty, too-small buffer, as a worker's is from block to block
        let mut out = vec![0xEE; 3];
        catch_unwind(AssertUnwindSafe(|| {
            f(edges, extra, spelling, offsets, n, mark, bits, &mut out)
        }))
        .ok()
        .map(|()| out)
    }

    #[derive(Default, Debug)]
    struct Tally {
        cases: u64,
        panics: u64,
        mismatches: u64,
    }

    /// The dictionary: word 0 "xy", word 1 empty, word 2 a slice past `spelling`, word 3 past
    /// `offsets`; codes 65 (a literal) and 256..=259
    const SPELLING: &[u8] = b"xyz";
    const OFFSETS: &[u32] = &[0, 2, 2, 5];
    const CODES: [u32; 5] = [65, 256, 257, 258, 259];

    fn check(t: &mut Tally, edges: &[u32], extra: &[u32], bits: u32) {
        for mark in 0..=edges.len() as u32 {
            for n in 0..=6 {
                let a = outcome(emit_shipped, edges, extra, SPELLING, OFFSETS, n, mark, bits);
                let b = outcome(
                    emit_pipelined,
                    edges,
                    extra,
                    SPELLING,
                    OFFSETS,
                    n,
                    mark,
                    bits,
                );
                t.cases += 1;
                t.panics += u64::from(a.is_none());
                if a != b {
                    t.mismatches += 1;
                    if t.mismatches <= 20 {
                        eprintln!(
                            "MISMATCH edges={edges:?} extra={extra:?} bits={bits} mark={mark} \
                             n={n}: shipped {a:?}, pipelined {b:?}"
                        );
                    }
                }
            }
        }
    }

    /// Every sequence of `len` items, each drawn from `0..radix`
    fn each_seq(len: usize, radix: usize, mut f: impl FnMut(&[usize])) {
        let mut v = vec![0; len];
        loop {
            f(&v);
            let mut i = 0;
            loop {
                if i == len {
                    return;
                }
                v[i] += 1;
                if v[i] < radix {
                    break;
                }
                v[i] = 0;
                i += 1;
            }
        }
    }

    /// The shipped walk and the pipelined walk agree, panic for panic and byte for byte, on
    /// every small construction: up to 4 rows, 1 to 3 bits, every successor in `0..=len` that
    /// the mask allows (`len` is out of bounds), every code (literals, words, out-of-range words),
    /// narrow and wide with `extra` of `len - 1`, `len` and `len + 1`, every mark in `0..=len`
    /// and every `n` in 0..=6
    #[test]
    fn pipelined_emit_matches_shipped_exhaustively() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let mut narrow = Tally::default();
        let mut wide = Tally::default();
        for len in 0..=4usize {
            for bits in 1..=3u32 {
                let succ = (len + 1).min(1 << bits);
                // narrow: each edge a code and a successor
                each_seq(len, succ * CODES.len(), |s| {
                    let edges: Vec<u32> = s
                        .iter()
                        .map(|&x| (CODES[x / succ] << bits) | (x % succ) as u32)
                        .collect();
                    check(&mut narrow, &edges, &[], bits);
                });
                // wide: each edge a successor (its own code bits set, and ignored); the codes
                // in `extra`. `extra[len]` is never read (its row is out of bounds of `edges`
                // first), so the longer `extra` varies only its first `len` codes
                each_seq(len, succ, |s| {
                    let edges: Vec<u32> = s.iter().map(|&x| (7 << bits) | x as u32).collect();
                    for xlen in [len.wrapping_sub(1), len, len + 1] {
                        if xlen == 0 || xlen > len + 1 {
                            continue; // empty `extra` is the narrow walk
                        }
                        let varied = xlen.min(len);
                        each_seq(varied, CODES.len(), |c| {
                            let mut extra: Vec<u32> = c.iter().map(|&x| CODES[x]).collect();
                            extra.resize(xlen, 66);
                            check(&mut wide, &edges, &extra, bits);
                        });
                    }
                });
            }
        }
        std::panic::set_hook(hook);
        eprintln!("narrow: {narrow:?}");
        eprintln!("wide: {wide:?}");
        assert_eq!(narrow.mismatches + wide.mismatches, 0);
    }

    /// Byte-exact on real blocks: the shipped and pipelined walks over the kernel's own
    /// construction of each exact column in `MIBZ2_EMIT_BLOCKS` (a directory of `block-<id>.L`
    /// and `manifest.json`), checked against a dense-LF inversion too
    #[test]
    #[ignore = "needs MIBZ2_EMIT_BLOCKS"]
    fn pipelined_emit_matches_shipped_on_real_blocks() {
        let dir = std::env::var("MIBZ2_EMIT_BLOCKS").expect("MIBZ2_EMIT_BLOCKS");
        let manifest = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
        let mut seen = 0;
        for id in [0, 2, 85, 143, 207, 210] {
            let file = format!("block-{id}.L");
            let ll = std::fs::read(format!("{dir}/{file}")).unwrap();
            let at = manifest.find(&format!("\"file\": \"{file}\"")).unwrap();
            let rest = &manifest[at..];
            let k = rest.find("\"primary_index\":").unwrap() + "\"primary_index\":".len();
            let p: u32 = rest[k..]
                .trim_start()
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap()
                .parse()
                .unwrap();
            let records = super::super::records_of(&ll);
            let n = ll.len();
            let mut kernel = super::super::Kernel::default();
            let (_, mark, bits) =
                kernel.construct::<true, true>(&records, n as u32, p, super::super::LIMIT);
            let (mut a, mut b) = (Vec::new(), Vec::new());
            let k = &kernel;
            emit_shipped(
                &k.edges,
                &k.extra,
                &k.spelling,
                &k.offsets,
                n,
                mark,
                bits,
                &mut a,
            );
            emit_pipelined(
                &k.edges,
                &k.extra,
                &k.spelling,
                &k.offsets,
                n,
                mark,
                bits,
                &mut b,
            );
            assert_eq!(a.len(), n);
            assert!(a == b, "block {id}: pipelined differs from shipped");
            assert!(
                a == dense(&ll, p),
                "block {id}: shipped differs from dense LF"
            );
            eprintln!(
                "block {id}: n={n} edges={} extra={} byte-exact",
                k.edges.len(),
                k.extra.len()
            );
            seen += 1;
        }
        assert_eq!(seen, 6);
    }

    fn dense(ll: &[u8], p: u32) -> Vec<u8> {
        let mut start = [0u32; 256];
        for &c in ll {
            start[c as usize] += 1;
        }
        let mut acc = 0;
        for s in start.iter_mut() {
            let c = *s;
            *s = acc;
            acc += c;
        }
        let mut t = vec![0u32; ll.len()];
        for (i, &c) in ll.iter().enumerate() {
            t[start[c as usize] as usize] = i as u32;
            start[c as usize] += 1;
        }
        let mut out = Vec::with_capacity(ll.len());
        let mut j = t[p as usize];
        for _ in 0..ll.len() {
            out.push(ll[j as usize]);
            j = t[j as usize];
        }
        out
    }
}
