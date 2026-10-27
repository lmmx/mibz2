//! The sequential part of the decode: a .bz2's streams read in order, each block's header
//! and symbols (block.rs), each stream's header and end, its combined CRC checked. The
//! input comes from a `Source`: a slice, or chunks arriving on a channel (parallel.rs),
//! held only from the block being read on.

use super::bits::Bits;
use super::block::{self, Item, Symbols};
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

pub enum Event {
    /// a block's symbols; its index (counted across streams), first bit and end bit (the
    /// next item's first); its stream (counted from 0) and that stream's level
    Block { index: u64, bit: u64, end: u64, stream: u64, level: u32, symbols: Symbols },
    /// the input's end, after a whole stream
    Done,
}

enum State {
    Header { first: bool },
    Stream { level: u32, combined: u32 },
    Done,
}

/// The input a block's read first asks for, doubled while a block runs past it (a real
/// block is at most ~4.2 MB compressed; nearly all are under 1 MB), up to MAX_ITEM
const LOOKAHEAD: usize = 2 << 20;
const MAX_ITEM: usize = 64 << 20;

pub struct Scanner<S> {
    src: S,
    /// the next item's first bit
    bit: u64,
    state: State,
    blocks: u64,
    streams: u64,
    lookahead: usize,
}

impl<S: Source> Scanner<S> {
    pub fn new(src: S) -> Self {
        Scanner { src, bit: 0, state: State::Header { first: true }, blocks: 0, streams: 0, lookahead: LOOKAHEAD }
    }

    /// The next block (its symbols read into `syms`, reused), or the input's end
    pub fn next(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {
        loop {
            match self.state {
                State::Done => return Ok(Event::Done),
                State::Header { first } => {
                    let at = self.bit;
                    let err = |kind| Error { kind, block: None, bit: Some(at) };
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
                            self.state = State::Stream { level: (l - b'0') as u32, combined: 0 };
                            self.bit += 32;
                        }
                        _ => return Err(err(if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData })),
                    }
                }
                State::Stream { level, combined } => {
                    let at = self.bit;
                    let index = self.blocks;
                    let err = |kind| Error { kind, block: Some(index), bit: Some(at) };
                    let base = at / 8;
                    let (data, eof) = self.src.view(base, self.lookahead).map_err(err)?;
                    let mut br = Bits::at(data, at % 8);
                    let item = block::read_item(&mut br, level, syms);
                    let used = br.bit_pos() - at % 8;
                    if br.overran() {
                        // an item past the input held: more input, or the input's end
                        if eof {
                            return Err(err(ErrorKind::Truncated));
                        }
                        if self.lookahead >= MAX_ITEM {
                            return Err(err(ErrorKind::BlockTooLarge));
                        }
                        self.lookahead = (self.lookahead * 2).max(data.len() + 1).min(MAX_ITEM);
                        syms = match item {
                            Ok(Item::Block(b)) => b.syms,
                            Ok(Item::End(_)) => Vec::new(),
                            Err((_, s)) => s,
                        };
                        continue;
                    }
                    self.bit += used;
                    self.src.release(self.bit / 8);
                    match item.map_err(|(kind, _)| err(kind))? {
                        Item::End(stored) => {
                            if stored != combined {
                                return Err(err(ErrorKind::StreamCrc { stored, computed: combined }));
                            }
                            // the next stream starts at a byte
                            self.bit = self.bit.div_ceil(8) * 8;
                            self.state = State::Header { first: false };
                            self.streams += 1;
                            syms = Vec::new();
                        }
                        Item::Block(symbols) => {
                            // (the stored CRC: each block's computed CRC is checked
                            // against it by its worker, in order before this stream's end)
                            self.state = State::Stream { level, combined: combined.rotate_left(1) ^ symbols.crc };
                            self.blocks += 1;
                            let (end, stream) = (self.bit, self.streams);
                            return Ok(Event::Block { index, bit: at, end, stream, level, symbols });
                        }
                    }
                }
            }
        }
    }
}
