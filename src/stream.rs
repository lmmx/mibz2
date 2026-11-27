#![forbid(unsafe_code)]
//! The sequential part of the decode: a .bz2's streams read in order, each block's header
//! and symbols (block.rs), each stream's header and end, its combined CRC checked. The
//! input comes from a `Source`: a slice, or chunks arriving on a channel (parallel.rs),
//! held only from the block being read on.

use super::bits::Bits;
use super::block::{self, Item, ScanScratch, Symbols};
use super::{Error, ErrorKind};

/// The input, by absolute byte offset
pub trait Source {
    /// Every byte held from `from` on: at least `want` bytes unless the input ends sooner,
    /// and whether it has ended (all of it to its end is then given). Err(Cancelled) if the
    /// decode was stopped.
    fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind>;
    /// No byte before `upto` will be asked for again
    fn release(&mut self, _upto: u64) {}
}

pub struct SliceSource<'a>(pub &'a [u8]);

impl Source for SliceSource<'_> {
    fn view(&mut self, from: u64, _want: usize) -> Result<(&[u8], bool), ErrorKind> {
        Ok((self.0.get(from as usize..).unwrap_or(&[]), true))
    }
}

#[allow(clippy::large_enum_variant)] // (as block.rs's Item: moved once a block)
pub enum Event {
    /// a block's symbols; its index (counted across streams), first bit and end bit (the
    /// next item's first); its stream (counted from 0) and that stream's level
    Block {
        index: u64,
        bit: u64,
        end: u64,
        stream: u64,
        level: u32,
        symbols: Symbols,
    },
    /// the input's end, after a whole stream
    Done,
}

enum State {
    Header { first: bool },
    Stream { level: u32, combined: u32 },
    Done,
}

/// First block's initial lookahead before any block lengths have been observed.
const INITIAL_LOOKAHEAD: usize = 128 << 10;
/// Adaptive per-block lookahead bounds over the last 16 completed items' compressed byte lengths.
const MIN_LOOKAHEAD: usize = 64 << 10;
const MAX_LOOKAHEAD: usize = 1 << 20;
const LOOKAHEAD_PAD: usize = 4 << 10;
const RETRY_STEP_INIT: usize = 128 << 10;
const MAX_ITEM: usize = 64 << 20;

pub struct Scanner<S> {
    src: S,
    /// the next item's first bit
    bit: u64,
    state: State,
    blocks: u64,
    streams: u64,
    /// compressed byte lengths of the last 16 completed items (sliding-window peak for lookahead)
    recent_lens: [usize; 16],
    recent_idx: usize,
    scratch: ScanScratch,
}

impl<S: Source> Scanner<S> {
    pub fn new(src: S) -> Self {
        Scanner {
            src,
            bit: 0,
            state: State::Header { first: true },
            blocks: 0,
            streams: 0,
            recent_lens: [0; 16],
            recent_idx: 0,
            scratch: ScanScratch::default(),
        }
    }

    #[inline(always)]
    fn initial_want(&self) -> usize {
        let max16 = self.recent_lens.iter().copied().max().unwrap_or(0);
        if max16 == 0 {
            INITIAL_LOOKAHEAD
        } else {
            (max16 + (max16 >> 2) + LOOKAHEAD_PAD).clamp(MIN_LOOKAHEAD, MAX_LOOKAHEAD)
        }
    }

    /// The next block (its symbols read into `syms`, reused), or the input's end
    pub fn next(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {
        loop {
            match self.state {
                State::Done => return Ok(Event::Done),
                State::Header { first } => {
                    let at = self.bit;
                    let err = |kind| Error {
                        kind,
                        block: None,
                        bit: Some(at),
                    };
                    let (data, eof) = self.src.view(at / 8, 4).map_err(err)?;
                    if data.is_empty() && eof {
                        if first {
                            return Err(err(ErrorKind::Empty));
                        }
                        self.state = State::Done;
                        continue;
                    }
                    match data {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                            self.state = State::Stream {
                                level: (l - b'0') as u32,
                                combined: 0,
                            };
                            self.bit += 32;
                        }
                        _ => {
                            return Err(err(if first {
                                ErrorKind::NotBz2
                            } else {
                                ErrorKind::TrailingData
                            }))
                        }
                    }
                }
                State::Stream { level, combined } => {
                    let at = self.bit;
                    let index = self.blocks;
                    let err = |kind| Error {
                        kind,
                        block: Some(index),
                        bit: Some(at),
                    };
                    let base = at / 8;
                    let mut want = self.initial_want();
                    let mut retry_step = RETRY_STEP_INIT;
                    loop {
                        let (data, eof) = self.src.view(base, want).map_err(err)?;
                        let data_len = data.len();
                        let mut br = Bits::at(data, at % 8);
                        let item =
                            block::read_item_with_scratch(&mut br, level, syms, &mut self.scratch);
                        let used = br.bit_pos() - at % 8;
                        if br.overran() {
                            // an item past the input held: more input, or the input's end
                            if eof {
                                return Err(err(ErrorKind::Truncated));
                            }
                            if want >= MAX_ITEM {
                                return Err(err(ErrorKind::BlockTooLarge));
                            }
                            want = (data_len + retry_step).max(want * 2).min(MAX_ITEM);
                            retry_step = (retry_step * 2).min(MAX_ITEM);
                            syms = match item {
                                Ok(Item::Block(b)) => b.syms,
                                Ok(Item::End(_)) => Vec::new(),
                                Err((_, s)) => s,
                            };
                            continue;
                        }
                        self.bit += used;
                        self.src.release(self.bit / 8);
                        let used_bytes = used.div_ceil(8) as usize;
                        self.recent_lens[self.recent_idx] = used_bytes;
                        self.recent_idx = (self.recent_idx + 1) & 15;
                        match item.map_err(|(kind, _)| err(kind))? {
                            Item::End(stored) => {
                                if stored != combined {
                                    return Err(err(ErrorKind::StreamCrc {
                                        stored,
                                        computed: combined,
                                    }));
                                }
                                // the next stream starts at a byte
                                self.bit = self.bit.div_ceil(8) * 8;
                                self.state = State::Header { first: false };
                                self.streams += 1;
                                syms = Vec::new();
                                break;
                            }
                            Item::Block(symbols) => {
                                // (the stored CRC: each block's computed CRC is checked
                                // against it by its worker, in order before this stream's end)
                                self.state = State::Stream {
                                    level,
                                    combined: combined.rotate_left(1) ^ symbols.crc,
                                };
                                self.blocks += 1;
                                let (end, stream) = (self.bit, self.streams);
                                return Ok(Event::Block {
                                    index,
                                    bit: at,
                                    end,
                                    stream,
                                    level,
                                    symbols,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Source` serving a slice capped to `want` bytes per call (exercising adaptive lookahead
    /// and `br.overran()` retry growth cleanly under Miri).
    struct CappedSource<'a> {
        data: &'a [u8],
        released: u64,
        wants: Vec<usize>,
    }

    impl Source for CappedSource<'_> {
        fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
            self.wants.push(want);
            let start = (from as usize).min(self.data.len());
            let end = start.saturating_add(want).min(self.data.len());
            let eof = end == self.data.len();
            Ok((&self.data[start..end], eof))
        }

        fn release(&mut self, upto: u64) {
            assert!(upto >= self.released);
            self.released = upto;
        }
    }

    #[test]
    fn adaptive_lookahead_and_retry_growth() {
        use bzip2::write::BzEncoder;
        use bzip2::Compression;
        use std::io::Write;

        // Multi-stream input: tiny compressible block first, then an incompressible block that
        // exceeds the adaptive 64 KiB floor so `Scanner::next` must grow `want` and retry cleanly.
        let mut bz = Vec::new();
        {
            let mut enc = BzEncoder::new(&mut bz, Compression::new(1));
            enc.write_all(&vec![b'a'; 10_000]).unwrap();
            enc.finish().unwrap();
        }
        {
            let mut x = 0x9e37_79b9_7f4a_7c15u64;
            let mut noise = vec![0u8; 95_000];
            for b in &mut noise {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                *b = x as u8;
            }
            let mut enc = BzEncoder::new(&mut bz, Compression::new(1));
            enc.write_all(&noise).unwrap();
            enc.finish().unwrap();
        }

        let mut want_events = Vec::new();
        {
            let mut sc = Scanner::new(SliceSource(&bz));
            while let Event::Block {
                bit, end, symbols, ..
            } = sc.next(Vec::new()).unwrap()
            {
                want_events.push((bit, end, symbols.crc, symbols.syms));
            }
        }

        let mut got_events = Vec::new();
        let src = CappedSource {
            data: &bz,
            released: 0,
            wants: Vec::new(),
        };
        let mut sc = Scanner::new(src);
        while let Event::Block {
            bit, end, symbols, ..
        } = sc.next(Vec::new()).unwrap()
        {
            got_events.push((bit, end, symbols.crc, symbols.syms));
        }
        assert_eq!(got_events, want_events);
        // First block requested INITIAL_LOOKAHEAD (128 KiB), second dropped to MIN_LOOKAHEAD
        // (64 KiB) from the tiny first block, then grew on `overran()` to finish cleanly.
        assert!(sc.src.wants.contains(&INITIAL_LOOKAHEAD));
        assert!(sc.src.wants.contains(&MIN_LOOKAHEAD));
        assert!(sc.src.wants.iter().any(|&w| w > MIN_LOOKAHEAD));
    }

    #[test]
    fn miri_adaptive_lookahead_clamps_and_decays_after_16_blocks() {
        let mut sc = Scanner::new(SliceSource(&[]));
        assert_eq!(sc.initial_want(), INITIAL_LOOKAHEAD);
        // One huge 900 KiB block raises lookahead to MAX_LOOKAHEAD (1 MiB)
        sc.recent_lens[0] = 900 << 10;
        assert_eq!(sc.initial_want(), MAX_LOOKAHEAD);
        // Overwriting all 16 slots with tiny 10 KiB blocks decays lookahead back to MIN_LOOKAHEAD
        sc.recent_lens = [10 << 10; 16];
        assert_eq!(sc.initial_want(), MIN_LOOKAHEAD);
        // Moderate 200 KiB peak -> 200 KiB + 50 KiB + 4 KiB = 254 KiB
        sc.recent_lens[3] = 200 << 10;
        assert_eq!(sc.initial_want(), (200 << 10) + (50 << 10) + LOOKAHEAD_PAD);
    }
}
