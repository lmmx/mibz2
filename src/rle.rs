//! Phase 5's RLE1 undone: 4 equal bytes, then a byte counting 0-255 more of them. Scalar,
//! or (AVX2) the test for 4 equal bytes made at 32 positions at once, the stretches
//! between runs copied whole.
//!
//! A block's RLE1-coded bytes are either one slice (`unrle`), or the room walk's segments in
//! the output's order (`unrle_chain`), decoded where they lie with no copy joining them
//! first. RLE1's parse is positional (a token is a literal, or 4 equal bytes and a count),
//! so a token may straddle segments, and a segment may be one byte; the chain's decode
//! takes each token whole from a small stitch of the bytes either side of a boundary, then
//! goes on in the next segment past what the stitch took.

use super::crc::Running;
use super::{ErrorKind, Kernels};

/// `rle` with its runs undone, appended to `out`. Err if it ends with a run's 4 bytes and
/// no count: libbzip2 then reads a count past the block's end and refuses the block
pub fn unrle(rle: &[u8], out: &mut Vec<u8>, k: Kernels) -> Result<(), ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected
            return unsafe { unrle_avx2(rle, out) };
        }
    }
    let _ = k;
    unrle_scalar(rle, out, 0)
}

/// From `rle[i..]` (i at a run's start or a literal, never inside a run)
pub fn unrle_scalar(rle: &[u8], out: &mut Vec<u8>, mut i: usize) -> Result<(), ErrorKind> {
    let n = rle.len();
    out.reserve(n - i.min(n));
    while i < n {
        let c = rle[i];
        if i + 3 < n && rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            let Some(&extra) = rle.get(i + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            out.extend_from_slice(&[c, c, c, c]);
            out.resize(out.len() + extra as usize, c);
            i += 5;
        } else {
            out.push(c);
            i += 1;
        }
    }
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn unrle_avx2(rle: &[u8], out: &mut Vec<u8>) -> Result<(), ErrorKind> {
    let n = rle.len();
    out.reserve(n + Avx2::PAD);
    // whole steps while a step's window remains, then the last bytes one token at a time
    // SAFETY: AVX2 is enabled here
    let rest = steps::<Avx2>(rle, out);
    unrle_scalar(rle, out, n - rest)
}

/// One step of RLE1's decode: given the WINDOW bytes from a token's start (`w`, a slice
/// the caller cuts, so bounds-checked, of initialised bytes), the token or tokens it takes
/// (each whole) appended to `out`. It reads only `w` (taken as an array of WINDOW bytes, so
/// every read, vector loads among them, is within it by its type) and returns how far it
/// took, 1 to WINDOW
trait Step {
    /// the bytes a step reads
    const WINDOW: usize;
    /// what `unrle` reserves for `out` past the input's length (its ledger figure)
    const PAD: usize;
    /// # Safety
    /// The CPU has the step's target features. (A `w` shorter than WINDOW panics.)
    unsafe fn step(w: &[u8], out: &mut Vec<u8>) -> usize;
}

/// One token: a literal, or a run's 4 bytes and its count
struct Scalar;

impl Step for Scalar {
    const WINDOW: usize = 5;
    const PAD: usize = 0;

    #[inline(always)]
    unsafe fn step(w: &[u8], out: &mut Vec<u8>) -> usize {
        let w: &[u8; 5] = w.first_chunk().expect("a step's window");
        let c = w[0];
        if w[1] == c && w[2] == c && w[3] == c {
            out.extend_from_slice(&[c, c, c, c]);
            out.resize(out.len() + w[4] as usize, c);
            5
        } else {
            out.push(c);
            1
        }
    }
}

/// The test for a run made at 32 positions at once: the 32 bytes copied whole if none
/// starts a run, else the bytes before the first run and that run
#[cfg(target_arch = "x86_64")]
struct Avx2;

#[cfg(target_arch = "x86_64")]
impl Step for Avx2 {
    // 4 loads of 32 bytes at i..=i+3 (to i + 34), and a run starting by i + 31 has its count
    // by i + 35
    const WINDOW: usize = 36;
    const PAD: usize = 64;

    #[inline(always)]
    unsafe fn step(w: &[u8], out: &mut Vec<u8>) -> usize {
        use std::arch::x86_64::*;
        // (the caller cuts exactly WINDOW bytes, so this check folds away)
        let w: &[u8; 36] = w.first_chunk().expect("a step's window");
        // SAFETY (the loads): each reads 32 bytes from offsets 0..=3 of `w`, the last at 34,
        // within its 36 initialised bytes; AVX2 is the caller's to have enabled
        let p = w.as_ptr();
        let v0 = _mm256_loadu_si256(p as *const __m256i);
        let v1 = _mm256_loadu_si256(p.add(1) as *const __m256i);
        let v2 = _mm256_loadu_si256(p.add(2) as *const __m256i);
        let v3 = _mm256_loadu_si256(p.add(3) as *const __m256i);
        let eq = _mm256_and_si256(
            _mm256_cmpeq_epi8(v0, v1),
            _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
        );
        let m = _mm256_movemask_epi8(eq) as u32;
        if m == 0 {
            out.extend_from_slice(&w[..32]);
            return 32;
        }
        let j = m.trailing_zeros() as usize; // the first run's start, at most 31
        out.extend_from_slice(&w[..j]);
        let c = w[j];
        out.extend_from_slice(&[c, c, c, c]);
        // (its count at j + 4, at most 35: within the window)
        out.resize(out.len() + w[j + 4] as usize, c);
        j + 5
    }
}

/// The widest step's window (`Avx2`'s): a stitch holds at most two
const MAX_WINDOW: usize = 36;

/// Whole steps over `buf` from its start while a window remains: the bytes left after the
/// last (fewer than a window, from a token's start)
///
/// # Safety
/// The CPU has `S`'s target features.
#[inline(always)]
unsafe fn steps<S: Step>(buf: &[u8], out: &mut Vec<u8>) -> usize {
    let mut rest = buf;
    while let Some(w) = rest.get(..S::WINDOW) {
        // (a step takes at most its window, so this is within `rest`)
        rest = &rest[S::step(w, out)..];
    }
    rest.len()
}

/// `unrle` of the bytes `chunks` yields, in order, as if they were joined (`n` of them in
/// all: the block's length), appended to `out`, with the CRC of what it appends folded into
/// `crc` as it goes (`crc.finish` then gives it). The same bytes, and the same error under
/// the same condition, as `unrle` of the joined slice: an RLE1 decode is a function of its
/// input's bytes, wherever the boundaries between them fall. The pieces may be of any
/// length, one byte or none among them.
pub fn unrle_chain<'a, I>(
    chunks: I,
    n: usize,
    out: &mut Vec<u8>,
    crc: &mut Running,
    k: Kernels,
) -> Result<(), ErrorKind>
where
    I: Iterator<Item = &'a [u8]> + Clone,
{
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 (and BMI1) was detected
            return unsafe { unrle_chain_avx2(chunks, n, out, crc, k) };
        }
    }
    // SAFETY: the scalar step needs no target feature
    unsafe { chain::<Scalar, I>(chunks, n, out, crc, k) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn unrle_chain_avx2<'a, I>(
    chunks: I,
    n: usize,
    out: &mut Vec<u8>,
    crc: &mut Running,
    k: Kernels,
) -> Result<(), ErrorKind>
where
    I: Iterator<Item = &'a [u8]> + Clone,
{
    chain::<Avx2, I>(chunks, n, out, crc, k)
}

/// The chain's decode by the step `S`. In each piece, whole steps while a step's window
/// lies in it. The `lead` bytes left (fewer than a window, at a token's start) go into a
/// stitch with the next WINDOW bytes of the chain (from as many pieces as that takes),
/// and steps are taken there until one ends past the lead: the tokens straddling the
/// boundary are then taken whole, and the decode goes on that far into the pieces after
/// (`skip`). Every step taken reads only real bytes of the chain, in order, at a token's
/// start, so the tokens are `unrle`'s. When the chain ends within a stitch, the stitch
/// holds all that is left of the block, decoded to its end as `unrle` decodes a block's
/// last bytes, an unfinished run its error.
///
/// The usual boundary (this piece and the next both at least MAX_WINDOW bytes, as nearly
/// all of a block's are) takes the stitch as two fixed MAX_WINDOW-byte copies, the lead
/// the last of the first half, so it needs no gather across pieces: the same bytes in the
/// same order from the lead on, so the same steps.
///
/// # Safety
/// The CPU has `S`'s target features.
#[inline(always)]
unsafe fn chain<'a, S: Step, I>(
    mut chunks: I,
    n: usize,
    out: &mut Vec<u8>,
    crc: &mut Running,
    k: Kernels,
) -> Result<(), ErrorKind>
where
    I: Iterator<Item = &'a [u8]> + Clone,
{
    const { assert!(S::WINDOW <= MAX_WINDOW) };
    out.reserve(n + S::PAD);
    let mut stitch = [0u8; 2 * MAX_WINDOW];
    // the bytes of the pieces to come that a stitch has already decoded
    let mut skip = 0;
    while let Some(c) = chunks.next() {
        if skip >= c.len() {
            skip -= c.len();
            continue;
        }
        // SAFETY: the features are the caller's
        let lead = steps::<S>(&c[skip..], out);
        let i = c.len() - lead;
        skip = 0;
        if lead > 0 {
            if let (Some(a), Some(b)) = (
                c.last_chunk::<MAX_WINDOW>(),
                chunks
                    .clone()
                    .next()
                    .and_then(|d| d.first_chunk::<MAX_WINDOW>()),
            ) {
                let (h0, h1) = stitch.split_at_mut(MAX_WINDOW);
                h0.copy_from_slice(a);
                h1.copy_from_slice(b);
                let mut t = MAX_WINDOW - lead;
                while t < MAX_WINDOW {
                    // SAFETY: the features are the caller's. (t < MAX_WINDOW, so the window
                    // ends by 2 * MAX_WINDOW, the stitch's length)
                    t += S::step(&stitch[t..t + S::WINDOW], out);
                }
                // within the next piece, which has MAX_WINDOW bytes, a step at most a
                // window
                skip = t - MAX_WINDOW;
                crc.fold(out, k);
                continue;
            }
            let want = lead + S::WINDOW;
            stitch[..lead].copy_from_slice(&c[i..]);
            let mut w = lead;
            let mut ahead = chunks.clone();
            while w < want {
                let Some(d) = ahead.next() else { break };
                let t = d.len().min(want - w);
                stitch[w..w + t].copy_from_slice(&d[..t]);
                w += t;
            }
            if w < want {
                // the chain ends in the stitch: all that is left of the block
                return unrle_scalar(&stitch[..w], out, 0);
            }
            let mut t = 0;
            while t < lead {
                // SAFETY: the features are the caller's. (t < lead, so the window ends
                // before want = w, within the stitch's bytes)
                t += S::step(&stitch[t..t + S::WINDOW], out);
            }
            // fewer than a window past the lead, which the stitch had after it: within
            // the pieces to come
            skip = t - lead;
        }
        crc.fold(out, k);
    }
    debug_assert_eq!(skip, 0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crc::crc32;

    struct Rng(u64);
    impl Rng {
        fn below(&mut self, m: u64) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0 % m
        }
    }

    /// `unrle_chain` over `rle` cut at `cuts` (ascending, within 0..=rle.len(), repeats
    /// giving empty pieces) is `unrle` of it whole: the same bytes after a prefix already in
    /// `out`, the same error, and the CRC of the bytes appended, by every kernel
    fn agree(rle: &[u8], cuts: &[usize], what: &str) {
        let mut pieces = Vec::new();
        let mut at = 0;
        for &c in cuts.iter().chain([&rle.len()]) {
            pieces.push(&rle[at..c]);
            at = c;
        }
        let mut want = b"prefix".to_vec();
        let want_err = unrle_scalar(rle, &mut want, 0).err();
        for k in [Kernels::scalar(), Kernels::detect()] {
            let mut whole = b"prefix".to_vec();
            assert_eq!(
                unrle(rle, &mut whole, k).err(),
                want_err,
                "{what}: unrle, {k:?}"
            );
            assert!(want_err.is_some() || whole == want, "{what}: unrle, {k:?}");
            let mut got = b"prefix".to_vec();
            let mut crc = Running::new(got.len());
            let err = unrle_chain(pieces.iter().copied(), rle.len(), &mut got, &mut crc, k).err();
            assert_eq!(err, want_err, "{what}: the error, {k:?}");
            if err.is_none() {
                assert!(got == want, "{what}: the bytes, {k:?}");
                assert_eq!(crc.finish(&got, k), crc32(&want[6..], k), "{what}: the CRC");
            }
        }
    }

    /// RLE1-coded bytes: literals over a small alphabet (so that 4 equal bytes come by
    /// chance as well), runs placed whole (4 bytes and a count), and the count bytes
    /// themselves often equal to the run's byte
    fn coded(rng: &mut Rng, len: usize) -> Vec<u8> {
        let wide = rng.below(2) == 0;
        let alphabet = 1 + rng.below(if wide { 256 } else { 3 });
        let mut v = Vec::with_capacity(len + 5);
        while v.len() < len {
            let c = rng.below(alphabet) as u8;
            if rng.below(6) == 0 {
                v.extend_from_slice(&[c, c, c, c]);
                v.push(match rng.below(3) {
                    0 => c,
                    1 => rng.below(4) as u8,
                    _ => rng.below(256) as u8,
                });
            } else {
                v.push(c);
            }
        }
        v.truncate(len);
        v
    }

    /// Runs at every offset to a boundary: a run (4 bytes and its count) placed at each
    /// position p about one cut, in literals that never make a run of their own, so the
    /// boundary falls before, inside (its count byte a piece's first among them) and after
    /// it; and with a second cut a byte later (a 1-byte piece), at both kernels' windows
    #[test]
    fn a_run_at_every_offset_to_a_boundary() {
        for q in [1, 5, 35, 36, 37, 40, 71, 72, 80] {
            for p in 0..q + 40 {
                let mut rle: Vec<u8> = (0..q + 90).map(|i| (i % 7) as u8 + 1).collect();
                rle[p..p + 5].copy_from_slice(&[9, 9, 9, 9, 3]);
                agree(&rle, &[q], &format!("run at {p}, cut at {q}"));
                agree(
                    &rle,
                    &[q, q + 1],
                    &format!("run at {p}, 1-byte piece at {q}"),
                );
                agree(
                    &rle,
                    &[q, q + 1, q + 2, q + 3],
                    &format!("run at {p}, 3 x 1 at {q}"),
                );
                // a piece holding the whole run, and empty pieces about it
                agree(&rle, &[p, p, p + 5, p + 5], &format!("run at {p} alone"));
            }
        }
    }

    /// A block ending in a run's 4 bytes with no count (the error), or in fewer than 4
    /// equal bytes (literals), cut at every point near its end and into 1-byte pieces
    #[test]
    fn a_run_unfinished_in_the_last_piece() {
        for tail in 1..=6usize {
            for len in [tail, tail + 1, 40, 100] {
                let mut rle: Vec<u8> = (0..len).map(|i| (i % 5) as u8 + 1).collect();
                for b in &mut rle[len - tail..] {
                    *b = 200;
                }
                for cut in 0..=len {
                    agree(
                        &rle,
                        &[cut],
                        &format!("{tail} equal at the end of {len}, cut {cut}"),
                    );
                }
                let ones: Vec<usize> = (1..len).collect();
                agree(
                    &rle,
                    &ones,
                    &format!("{tail} equal at the end of {len}, 1-byte pieces"),
                );
                let tail_ones: Vec<usize> = (len.saturating_sub(8)..len).collect();
                agree(
                    &rle,
                    &tail_ones,
                    &format!("{tail} at the end of {len}, last 8 alone"),
                );
            }
        }
    }

    /// Random coded blocks cut at random: pieces of one byte only, of a few bytes, about a
    /// window, and long; empty pieces among them
    #[test]
    fn random_chains_agree_with_the_whole() {
        let mut rng = Rng(0x0123_4567_89ab_cdef);
        for case in 0..4000 {
            let len = rng.below(if case % 4 == 0 { 3000 } else { 300 }) as usize;
            let mut rle = coded(&mut rng, len);
            if case % 3 == 0 && len >= 4 {
                // end in an unfinished run
                let c = rle[len - 4];
                for b in &mut rle[len - 4..] {
                    *b = c;
                }
            }
            let most = [1u64, 2, 4, 8, 40, 80, 600][rng.below(7) as usize];
            let mut cuts = Vec::new();
            let mut at = 0;
            loop {
                at += rng.below(most + 1) as usize;
                if at > len {
                    break;
                }
                cuts.push(at);
            }
            agree(&rle, &cuts, &format!("case {case}, pieces up to {most}"));
        }
    }

    /// The CRC is folded as the output grows: a block that expands to many times `FOLD`
    /// (runs of 255), in pieces of every size about the fold, gives `crc32`'s
    #[test]
    fn the_folded_crc_over_long_output() {
        let mut rng = Rng(0xfeed_beef);
        let rle: Vec<u8> = (0..4000)
            .flat_map(|i| {
                let c = (i % 13) as u8;
                if i % 3 == 0 {
                    vec![c, c, c, c, 255]
                } else {
                    vec![c, c.wrapping_add(1)]
                }
            })
            .collect();
        for most in [1u64, 37, 700, 5000] {
            let mut cuts = Vec::new();
            let mut at = 0;
            loop {
                at += 1 + rng.below(most) as usize;
                if at >= rle.len() {
                    break;
                }
                cuts.push(at);
            }
            agree(&rle, &cuts, &format!("long output, pieces up to {most}"));
        }
    }

    /// For Miri (`cargo +nightly miri test --lib -- miri_`): the chain's decode (its stitch
    /// and its skips over the pieces to come) on short chains, scalar and detected kernels.
    /// Under Miri `Kernels::detect` finds only the target's features, so the AVX2 step and
    /// the clmul CRC are interpreted with them enabled: `RUSTFLAGS="-C
    /// target-feature=+avx2,+bmi1,+pclmulqdq,+sse4.1,+ssse3" cargo +nightly miri test --lib
    /// -- miri_chain`.
    #[test]
    fn miri_chain_stitches() {
        let mut rng = Rng(0x5eed);
        for case in 0..60 {
            let len = rng.below(120) as usize;
            let rle = coded(&mut rng, len);
            let most = [1u64, 3, 9, 50][case % 4];
            let mut cuts = Vec::new();
            let mut at = 0;
            loop {
                at += rng.below(most + 1) as usize;
                if at > len {
                    break;
                }
                cuts.push(at);
            }
            agree(&rle, &cuts, &format!("miri case {case}"));
        }
    }
}
