//! A bzip2 decoder built for memory-level parallelism, block-parallel like lbzip2
//! (`WIKIDATA_BZ2_DECODER=native` in the split; lbzip2 stays the default). The design and
//! its measurements are docs/journal/2026-10-27-compute-frontier.md (section 2, the
//! prototype `kernels/bz2k`) and docs/journal/2026-10-27-bz2-decoder.md.
//!
//! A block is decoded in five phases:
//!
//! 1. its header and Huffman-coded symbols (`block`), the one sequential part: a block's
//!    end is known only once its symbols are read, so one thread reads them in order;
//! 2. RUNA/RUNB and move-to-front (`mtf`; AVX2 for the move);
//! 3. the T-vector, by a counting sort (`bwt::build`);
//! 4. the inverse BWT's walk, as 12 interleaved chains joined in order
//!    (`bwt::walk_segments`: the walk is bound by a load's latency, not by bandwidth;
//!    interleaved traversals are Kärkkäinen, Kempa and Puglisi's, "Slashing the Time for
//!    BWT Inversion", DCC 2012, pp. 99-108; phases 3 and 4 sit behind `bwt::InverseBwt`);
//! 5. RLE1 undone (`rle`; AVX2) and the block's CRC (`crc`; PCLMULQDQ), checked against
//!    the CRC the block stores.
//!
//! Phases 2-5 run on worker threads, a block each (`parallel`), and the blocks come out in
//! order. Each stream's combined CRC is checked at its end. Every SIMD kernel has a
//! portable scalar twin, chosen at run time (`Kernels`): nothing here needs a target CPU
//! beyond the baseline, and it builds and runs on aarch64 on the scalar kernels.
//!
//! The format is libbzip2 1.0.8's, with its checks (`block`), and these choices:
//! - **randomised blocks** (written only by bzip2 0.9.0 and earlier) are refused, with
//!   their own error;
//! - **bytes after the last stream** that do not begin another stream are an error
//!   (lbzip2 and bzip2 ignore them), so nothing after a stream is ever silently dropped;
//! - an **oversubscribed Huffman code** whose code points past the alphabet (where
//!   libbzip2 reads a stale table entry) is an error;
//! - a block whose **compressed form passes 64 MiB** is an error (a real block is at most
//!   about 4.2 MB; only a code-length table padded with no-op steps could be longer).

pub mod bits;
pub mod block;
pub mod bwt;
pub mod crc;
pub mod mtf;
pub mod parallel;
pub mod rle;
pub mod stream;

use std::fmt;

/// The SIMD kernels to use, detected once at run time (or forced to the scalar ones)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kernels {
    /// AVX2: the move-to-front and RLE1
    pub avx2: bool,
    /// PCLMULQDQ (with SSE4.1 and SSSE3): the CRC
    pub clmul: bool,
}

impl Kernels {
    /// The portable kernels, on any CPU
    pub const fn scalar() -> Self {
        Kernels { avx2: false, clmul: false }
    }

    /// The fastest kernels this CPU has
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Kernels {
                avx2: std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("bmi1"),
                clmul: std::is_x86_feature_detected!("pclmulqdq")
                    && std::is_x86_feature_detected!("sse4.1")
                    && std::is_x86_feature_detected!("ssse3"),
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            Kernels::scalar()
        }
    }

    /// `detect()`, unless WIKIDATA_BZ2_SIMD is `0` or `off`: then the scalar kernels
    pub fn from_env() -> Self {
        match std::env::var("WIKIDATA_BZ2_SIMD").as_deref() {
            Ok("0") | Ok("off") => Kernels::scalar(),
            _ => Kernels::detect(),
        }
    }
}

/// What is wrong with the input
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// no input at all
    Empty,
    /// the input does not begin with a bzip2 stream header ("BZh" and a level 1-9)
    NotBz2,
    BadBlockMagic,
    Randomised,
    BadOrigPtr,
    NoSymbols,
    BadGroups,
    BadSelectors,
    BadSelector,
    BadCodeLength,
    BadHuffmanCode,
    SelectorOverrun,
    RunTooLong,
    BlockTooLong,
    RunUnfinished,
    BlockCrc { stored: u32, computed: u32 },
    StreamCrc { stored: u32, computed: u32 },
    Truncated,
    TrailingData,
    BlockTooLarge,
    /// a block range (`decode_blocks`) whose last block ran past its end bit
    RangeOvershoot,
    /// the decode was stopped by its reader
    Cancelled,
    /// a bug: a worker panicked (caught, so the decode fails rather than the process)
    Internal(String),
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Empty => write!(f, "no input: not a bzip2 file"),
            Kind::NotBz2 => write!(f, "not a bzip2 file (no BZh1-9 stream header)"),
            Kind::BadBlockMagic => write!(f, "bad block header magic"),
            Kind::Randomised => write!(
                f,
                "a randomised block (written only by bzip2 0.9.0 and earlier): not supported by this decoder"
            ),
            Kind::BadOrigPtr => write!(f, "the block's primary index is out of range"),
            Kind::NoSymbols => write!(f, "the block uses no symbols"),
            Kind::BadGroups => write!(f, "the block's number of Huffman tables is not 2-6"),
            Kind::BadSelectors => write!(f, "the block has no selectors"),
            Kind::BadSelector => write!(f, "a selector names no Huffman table"),
            Kind::BadCodeLength => write!(f, "a Huffman code length is not 1-20"),
            Kind::BadHuffmanCode => write!(f, "a Huffman code is not in its table"),
            Kind::SelectorOverrun => write!(f, "the block's symbols outrun its selectors"),
            Kind::RunTooLong => write!(f, "a RUNA/RUNB run is too long"),
            Kind::BlockTooLong => write!(f, "the block is longer than its stream's level allows"),
            Kind::RunUnfinished => write!(f, "the block ends inside an RLE1 run (4 equal bytes, no count)"),
            Kind::BlockCrc { stored, computed } => {
                write!(f, "block CRC mismatch (stored {stored:#010x}, computed {computed:#010x})")
            }
            Kind::StreamCrc { stored, computed } => {
                write!(f, "stream CRC mismatch (stored {stored:#010x}, computed {computed:#010x})")
            }
            Kind::Truncated => write!(f, "unexpected end of input (truncated)"),
            Kind::TrailingData => write!(f, "trailing data after the last bzip2 stream"),
            Kind::BlockTooLarge => write!(f, "a block's compressed form is over 64 MiB"),
            Kind::RangeOvershoot => write!(f, "the last block runs past the range's end"),
            Kind::Cancelled => write!(f, "cancelled"),
            Kind::Internal(m) => write!(f, "internal error: {m}"),
        }
    }
}

/// An error, and where: the block (counted from the input's first, across its streams)
/// and the bit of the input it starts at
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: Kind,
    pub block: Option<u64>,
    pub bit: Option<u64>,
}

impl Error {
    pub fn new(kind: Kind) -> Self {
        Error { kind, block: None, bit: None }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bz2 data error: {}", self.kind)?;
        match (self.block, self.bit) {
            (Some(b), Some(at)) => write!(f, " (block {b}, at bit {at} = byte {})", at / 8),
            (None, Some(at)) => write!(f, " (at bit {at} = byte {})", at / 8),
            (Some(b), None) => write!(f, " (block {b})"),
            _ => Ok(()),
        }
    }
}

impl std::error::Error for Error {}

/// A worker's buffers, kept from block to block (about 7 MB at the 900 kB block size)
pub struct Scratch<I: bwt::InverseBwt = bwt::Segmented> {
    ll: Vec<u8>,
    counts: [u32; 256],
    inverse: I,
    rle: Vec<u8>,
}

impl<I: bwt::InverseBwt> Default for Scratch<I> {
    fn default() -> Self {
        Scratch { ll: Vec::new(), counts: [0; 256], inverse: I::default(), rle: Vec::new() }
    }
}

/// Phases 2-5 of a block whose symbols are read: its bytes appended to `out`, its CRC
/// checked. Returns the CRC.
pub fn decode_block<I: bwt::InverseBwt>(
    b: &block::Symbols,
    k: Kernels,
    s: &mut Scratch<I>,
    out: &mut Vec<u8>,
) -> Result<u32, Kind> {
    let n = mtf::mtf(b, &mut s.ll, &mut s.counts, k)?;
    if b.orig_ptr as usize >= n {
        return Err(Kind::BadOrigPtr);
    }
    s.inverse.invert(&s.ll, &s.counts, b.orig_ptr, &mut s.rle);
    let start = out.len();
    if let Err(kind) = rle::unrle(&s.rle, out, k) {
        out.truncate(start);
        return Err(kind);
    }
    let computed = crc::crc32(&out[start..], k);
    if computed != b.crc {
        out.truncate(start);
        return Err(Kind::BlockCrc { stored: b.crc, computed });
    }
    Ok(computed)
}

/// A whole .bz2 (its streams one after another) decoded on this thread, every block's and
/// stream's CRC checked: the reference the parallel decoder is tested against
pub fn decode(data: &[u8], k: Kernels) -> Result<Vec<u8>, Error> {
    decode_with::<bwt::Segmented>(data, k)
}

/// `decode` with the inverse-BWT kernel `I` (`bwt::Plain`: the plain walk, to test the
/// default against)
pub fn decode_with<I: bwt::InverseBwt>(data: &[u8], k: Kernels) -> Result<Vec<u8>, Error> {
    let mut sc = stream::Scanner::new(stream::SliceSource(data));
    let mut out = Vec::new();
    let mut s = Scratch::<I>::default();
    let mut syms = Vec::new();
    loop {
        match sc.next(syms)? {
            stream::Event::Block { index, bit, symbols } => {
                decode_block(&symbols, k, &mut s, &mut out)
                    .map_err(|kind| Error { kind, block: Some(index), bit: Some(bit) })?;
                syms = symbols.syms;
            }
            stream::Event::Done => return Ok(out),
        }
    }
}

/// What `decode_blocks` read: each block's bit range and CRC, and where it stopped
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Range {
    /// each block's (start bit, end bit, CRC), in order
    pub blocks: Vec<(u64, u64, u32)>,
    /// the bit after the last item read: `end_bit`, or the bit after a stream's end
    /// marker and its CRC, where the range met one
    pub end: u64,
    /// whether it met the stream's end marker (then `stored_combined` is its CRC)
    pub stream_end: bool,
    pub stored_combined: u32,
}

/// The blocks of `data` from `start_bit` (a block header's first bit) to `end_bit` (the
/// next block's first bit, or a stream end marker's), in a stream of `level` (1-9),
/// appended to `out`, each block's CRC checked; a range that meets its stream's end marker
/// stops there. For a reader that finds block boundaries itself and decodes ranges of
/// blocks in parallel (the sweep reader's block-aligned segments, which today take
/// `lbzip2 -n1` each): it can combine the returned CRCs (`combine`) to check its stream's.
pub fn decode_blocks(
    data: &[u8],
    start_bit: u64,
    end_bit: u64,
    level: u32,
    k: Kernels,
    out: &mut Vec<u8>,
) -> Result<Range, Error> {
    let mut s = Scratch::<bwt::Segmented>::default();
    let mut syms = Vec::new();
    let mut range = Range { end: start_bit, ..Range::default() };
    let mut bit = start_bit;
    while bit < end_bit {
        let index = range.blocks.len() as u64;
        let at = |kind| Error { kind, block: Some(index), bit: Some(bit) };
        let mut br = bits::Bits::at(data, bit);
        let item = block::read_item(&mut br, level, syms);
        if br.overran() {
            return Err(at(Kind::Truncated));
        }
        let next = br.bit_pos();
        match item.map_err(|(kind, _)| at(kind))? {
            block::Item::End(crc) => {
                range.stream_end = true;
                range.stored_combined = crc;
                range.end = next;
                return Ok(range);
            }
            block::Item::Block(b) => {
                if next > end_bit {
                    return Err(at(Kind::RangeOvershoot));
                }
                let crc = decode_block(&b, k, &mut s, out).map_err(at)?;
                range.blocks.push((bit, next, crc));
                syms = b.syms;
            }
        }
        bit = next;
        range.end = bit;
    }
    Ok(range)
}

/// A stream's combined CRC from its blocks' CRCs, in order
pub fn combine(crcs: impl IntoIterator<Item = u32>) -> u32 {
    crcs.into_iter().fold(0, |c, b| c.rotate_left(1) ^ b)
}
