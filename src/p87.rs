//! P87's four-core diagnostic (feature `p87` only; never in a default build:
//! docs/journal/2026-11-01-p87-four-core-protocol.md). Where the inverse BWT's assembly
//! writes a block's RLE1-coded bytes: into `rle`, the scratch's own buffer for them, as the
//! shipped decoder does (arm A); or into `ll`, the MTF stage's last column, which nothing
//! reads once the T-vector is built (arm B).
//!
//! **What differs between the arms is one pointer.** Both run `decode_block` below: one
//! compiled function, the kernel's own steps (`InverseBwt::p87_build`, `p87_walk`,
//! `p87_assemble`: `Segmented::invert`'s calls, in its order) and RLE1 and the CRC from
//! the chosen buffer. The build and the walk read and write the same buffers in both arms.
//!
//! **What is equalised:** on a scratch's first block, `ll` and `rle` are both allocated at
//! the level's block capacity and written once (prefaulted), in an order the run sets
//! (`MIBZ2_P87_PLACEMENT`). Neither is touched again except by its arm's own steps. In arm
//! B, `rle` stays allocated and is never written again.
//!
//! **The arm** is process-wide: `set`, or else `MIBZ2_P87_ARM` (`A`: into `rle`; `B`: into
//! `ll`; unset or anything else: the shipped path) and `MIBZ2_P87_PLACEMENT` (`1`: `rle`
//! allocated first; otherwise `ll` first), read at the first block.
use crate::bwt::InverseBwt;
use crate::{block, crc, mtf, rle, ErrorKind, Kernels, Scratch};
use std::sync::atomic::{AtomicU8, Ordering};

/// Which path `decode_block` takes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    /// the shipped `crate::decode_block`, unchanged
    Shipped,
    /// arm A: the assembly into `rle` (the shipped buffer), through this module's path
    IntoRle,
    /// arm B: the assembly into `ll`, once the build has read it
    IntoLl,
}

const UNSET: u8 = 0xff;
static STATE: AtomicU8 = AtomicU8::new(UNSET);

fn encode(arm: Arm, rle_first: bool) -> u8 {
    (match arm {
        Arm::Shipped => 0,
        Arm::IntoRle => 1,
        Arm::IntoLl => 2,
    }) | (u8::from(rle_first) << 4)
}

fn decode(v: u8) -> (Arm, bool) {
    let arm = match v & 0xf {
        1 => Arm::IntoRle,
        2 => Arm::IntoLl,
        _ => Arm::Shipped,
    };
    (arm, v >> 4 & 1 == 1)
}

/// Set the process's arm and placement (a harness or a test: before the first block)
pub fn set(arm: Arm, rle_first: bool) {
    STATE.store(encode(arm, rle_first), Ordering::Relaxed);
}

/// The process's arm, and whether `rle` is allocated before `ll` (from the environment
/// unless `set`)
pub fn arm() -> (Arm, bool) {
    let v = STATE.load(Ordering::Relaxed);
    if v != UNSET {
        return decode(v);
    }
    let arm = match std::env::var("MIBZ2_P87_ARM").as_deref() {
        Ok("A") => Arm::IntoRle,
        Ok("B") => Arm::IntoLl,
        _ => Arm::Shipped,
    };
    let rle_first = std::env::var("MIBZ2_P87_PLACEMENT").as_deref() == Ok("1");
    let v = encode(arm, rle_first);
    STATE.store(v, Ordering::Relaxed);
    decode(v)
}

/// `v` allocated at exactly `cap` bytes and written once, then emptied (its capacity and
/// resident pages kept)
fn prefault(v: &mut Vec<u8>, cap: usize) {
    v.clear();
    v.reserve_exact(cap);
    v.resize(cap, 0);
    v.clear();
}

/// `crate::decode_block` with the inverse BWT in its three steps, the assembly (and a block
/// too short to segment, its plain walk) written into `ll` if `into_ll`, else into `rle`;
/// RLE1 and the CRC then read that buffer. For a kernel with `p87_steps` and no
/// checkpoints (the caller checks).
pub fn decode_block<I: InverseBwt>(
    b: &block::Symbols,
    k: Kernels,
    s: &mut Scratch<I>,
    out: &mut Vec<u8>,
    into_ll: bool,
    rle_first: bool,
) -> Result<u32, ErrorKind> {
    if !s.p87_ready {
        let cap = b.max_len;
        if rle_first {
            prefault(&mut s.rle, cap);
            prefault(&mut s.ll, cap);
        } else {
            prefault(&mut s.ll, cap);
            prefault(&mut s.rle, cap);
        }
        s.p87_ready = true;
    }
    let n = mtf::mtf(b, &mut s.ll, &mut s.counts, k)?;
    if b.orig_ptr as usize >= n {
        return Err(ErrorKind::BadOrigPtr);
    }
    #[cfg(feature = "trace")]
    crate::trace::mark_sized(
        crate::trace::Kind::InvertStart,
        crate::trace::NONE,
        n as u64,
        crate::trace::UNREAD,
    );
    // the build: `ll`'s last read
    s.inverse.p87_build(&s.ll, &s.counts);
    #[cfg(feature = "trace")]
    crate::trace::mark(crate::trace::Kind::WalkStart, crate::trace::NONE);
    let dest = if into_ll { &mut s.ll } else { &mut s.rle };
    if s.inverse.p87_walk(b.orig_ptr, dest) {
        #[cfg(feature = "trace")]
        crate::trace::mark(crate::trace::Kind::AssembleStart, crate::trace::NONE);
        s.inverse.p87_assemble(dest);
    }
    #[cfg(feature = "trace")]
    crate::trace::mark(crate::trace::Kind::InvertEnd, crate::trace::NONE);
    let dest: &[u8] = dest;
    let start = out.len();
    if let Err(kind) = rle::unrle(dest, out, k) {
        out.truncate(start);
        return Err(kind);
    }
    let computed = crc::crc32(&out[start..], k);
    if computed != b.crc {
        out.truncate(start);
        return Err(ErrorKind::BlockCrc {
            stored: b.crc,
            computed,
        });
    }
    Ok(computed)
}

/// The scratch's `ll` and `rle` capacities, and `rle`'s length (the tests' check of the
/// equalised allocation, and that arm B never writes `rle` after its prefault)
pub fn buffers<I: InverseBwt>(s: &Scratch<I>) -> (usize, usize, usize) {
    (s.ll.capacity(), s.rle.capacity(), s.rle.len())
}
