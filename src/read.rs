//! `BzReader`: a .bz2 from any `Read`, as a `Read` of its decoded bytes, on the calling
//! thread or block-parallel; and `scan`, the blocks of a .bz2 in memory without decoding
//! them past their Huffman symbols (for a caller that splits the work itself).

use crate::parallel::Plan;
use crate::stream::{Event, Scanner, SliceSource, Source};
use crate::{decode_block, Error, ErrorKind, Kernels, Scratch, StreamDecoder};
use std::io::{self, Read};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// The input chunks a parallel `BzReader` reads from its source
const CHUNK: usize = 4 << 20;
/// Bytes a sequential reader's window keeps behind it before it drops them
const COMPACT: usize = 4 << 20;

/// A .bz2 stream (or several, one after another) read from `R`, decoded.
///
/// - `BzReader::new(r)` decodes on the calling thread, a block at a time.
/// - `BzReader::parallel(r, threads)` decodes `threads` blocks at once on worker threads
///   (plus one reading the stream in order and one reading `r`), in order.
///
/// Every block's CRC and every stream's combined CRC are checked; a bad stream is an
/// `io::Error` of kind `InvalidData` wrapping this crate's `Error`, after every byte of
/// the blocks before the bad one has been read.
pub struct BzReader<R> {
    inner: Inner<R>,
}

enum Inner<R> {
    Seq(Box<Seq<R>>),
    Par(Par),
}

struct Seq<R> {
    scanner: Scanner<ReadSource<R>>,
    kernels: Kernels,
    scratch: Scratch,
    syms: Vec<u16>,
    out: Vec<u8>,
    at: usize,
    done: bool,
    failed: Option<Error>,
}

struct Par {
    dec: Arc<StreamDecoder>,
    feeder: Option<JoinHandle<()>>,
    read_error: Arc<Mutex<Option<io::Error>>>,
}

impl<R: Read> BzReader<R> {
    /// Decode on the calling thread, with the fastest kernels this CPU has
    pub fn new(r: R) -> Self {
        Self::with_kernels(r, Kernels::detect())
    }

    /// Decode on the calling thread, with the kernels given (`Kernels::scalar()`: the
    /// portable ones)
    pub fn with_kernels(r: R, kernels: Kernels) -> Self {
        let src = ReadSource {
            r,
            buf: Vec::new(),
            base: 0,
            released: 0,
            eof: false,
        };
        BzReader {
            inner: Inner::Seq(Box::new(Seq {
                scanner: Scanner::new(src),
                kernels,
                scratch: Scratch::default(),
                syms: Vec::new(),
                out: Vec::new(),
                at: 0,
                done: false,
                failed: None,
            })),
        }
    }
}

impl<R: Read + Send + 'static> BzReader<R> {
    /// Decode on `threads` threads (at least 1), with at most 2 x `threads` blocks in
    /// flight, spent as `StreamDecoder::new` plans them (a block a worker, or teams of 2
    /// sharing each block on 6 or more physical cores: `Plan::for_threads`)
    pub fn parallel(r: R, threads: usize) -> Self {
        Self::parallel_with(r, threads, 2 * threads.max(1), Kernels::detect())
    }

    /// `parallel`, with the blocks in flight (`slots`, at least `threads`; scaled to the
    /// teams where it plans teams) and the kernels given
    pub fn parallel_with(r: R, threads: usize, slots: usize, kernels: Kernels) -> Self {
        Self::on(r, StreamDecoder::new(threads, slots, kernels))
    }

    /// Decode as `plan` says (production's workers, or cooperative teams: `Plan`), with
    /// `slots` blocks in flight and the kernels given
    pub fn parallel_planned(r: R, plan: Plan, slots: usize, kernels: Kernels) -> Self {
        Self::on(r, StreamDecoder::planned(plan, slots, kernels))
    }

    fn on(mut r: R, dec: StreamDecoder) -> Self {
        let dec = Arc::new(dec);
        let read_error = Arc::new(Mutex::new(None));
        let (d, e) = (dec.clone(), read_error.clone());
        let feeder = std::thread::Builder::new()
            .name("mibz2-read".into())
            .spawn(move || {
                loop {
                    let mut buf = vec![0u8; CHUNK];
                    let n = match read_full(&mut r, &mut buf) {
                        Ok(n) => n,
                        Err(err) => {
                            *e.lock().unwrap_or_else(|p| p.into_inner()) = Some(err);
                            d.cancel();
                            return;
                        }
                    };
                    if n == 0 {
                        break;
                    }
                    buf.truncate(n);
                    if d.write(buf).is_err() {
                        return;
                    }
                }
                d.close();
            })
            .expect("spawn the bz2 reader thread");
        BzReader {
            inner: Inner::Par(Par {
                dec,
                feeder: Some(feeder),
                read_error,
            }),
        }
    }
}

/// Fill `buf` as far as `r` goes (fewer bytes only at its end)
fn read_full<R: Read>(r: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        match r.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => n += k,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(n)
}

impl<R: Read> Read for BzReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            Inner::Seq(s) => s.read(buf),
            Inner::Par(p) => match p.dec.read(buf) {
                Ok(n) => Ok(n),
                Err(e) => {
                    if let Some(err) = p
                        .read_error
                        .lock()
                        .unwrap_or_else(|x| x.into_inner())
                        .take()
                    {
                        return Err(err);
                    }
                    Err(e.into())
                }
            },
        }
    }
}

impl<R: Read> Seq<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            if let Some(e) = &self.failed {
                return Err(e.clone().into());
            }
            if self.at < self.out.len() {
                let n = buf.len().min(self.out.len() - self.at);
                buf[..n].copy_from_slice(&self.out[self.at..self.at + n]);
                self.at += n;
                return Ok(n);
            }
            if self.done || buf.is_empty() {
                return Ok(0);
            }
            self.out.clear();
            self.at = 0;
            match self.scanner.next(std::mem::take(&mut self.syms)) {
                Ok(Event::Block {
                    index,
                    bit,
                    symbols,
                    ..
                }) => {
                    if let Err(kind) =
                        decode_block(&symbols, self.kernels, &mut self.scratch, &mut self.out)
                    {
                        self.failed = Some(Error {
                            kind,
                            block: Some(index),
                            bit: Some(bit),
                        });
                    }
                    self.syms = symbols.syms;
                }
                Ok(Event::Done) => self.done = true,
                Err(e) => {
                    if let ErrorKind::Io(kind, msg) = &e.kind {
                        return Err(io::Error::new(*kind, msg.clone()));
                    }
                    self.failed = Some(e);
                }
            }
        }
    }
}

impl<R> Drop for BzReader<R> {
    fn drop(&mut self) {
        if let Inner::Par(p) = &mut self.inner {
            p.dec.cancel();
            if let Some(h) = p.feeder.take() {
                // (the feeder's next write fails once the decoder is cancelled; a feeder
                // blocked in its source's read is left to finish that read)
                if h.is_finished() {
                    let _ = h.join();
                }
            }
        }
    }
}

/// A `Read` as a scanner's input, held from the block being read on
struct ReadSource<R> {
    r: R,
    buf: Vec<u8>,
    base: u64,
    released: u64,
    eof: bool,
}

impl<R: Read> Source for ReadSource<R> {
    fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
        while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
            let drop_n = (self.released - self.base) as usize;
            if drop_n >= COMPACT {
                self.buf.drain(..drop_n);
                self.base = self.released;
            }
            let len = self.buf.len();
            self.buf.resize(len + CHUNK, 0);
            match read_full(&mut self.r, &mut self.buf[len..]) {
                Ok(n) => {
                    self.buf.truncate(len + n);
                    self.eof = n < CHUNK;
                }
                Err(e) => {
                    self.buf.truncate(len);
                    return Err(ErrorKind::Io(e.kind(), e.to_string()));
                }
            }
        }
        let i = (from - self.base) as usize;
        Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
    }

    fn release(&mut self, upto: u64) {
        self.released = self.released.max(upto);
    }
}

/// A block of a .bz2, as `scan` finds it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSpan {
    /// counted from the input's first block, across its streams
    pub index: u64,
    /// its first bit (its header's magic) and its end (the next item's first bit)
    pub start_bit: u64,
    pub end_bit: u64,
    /// the CRC it stores, of its decoded bytes
    pub crc: u32,
    /// its stream, counted from 0, and that stream's level (1-9)
    pub stream: u64,
    pub level: u32,
}

/// The blocks of a .bz2 in memory, found by reading each block's header and Huffman
/// symbols (about a tenth of a decode's work; no block is decoded further, so no block
/// CRC is checked here, but each stream's combined CRC is checked against its blocks'
/// stored CRCs). Each span can then be decoded by `decode_blocks`, on any thread.
pub fn scan(data: &[u8]) -> Scan<'_> {
    Scan {
        scanner: Scanner::new(SliceSource(data)),
        syms: Vec::new(),
        done: false,
    }
}

pub struct Scan<'a> {
    scanner: Scanner<SliceSource<'a>>,
    syms: Vec<u16>,
    done: bool,
}

impl Iterator for Scan<'_> {
    type Item = Result<BlockSpan, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.scanner.next(std::mem::take(&mut self.syms)) {
            Ok(Event::Block {
                index,
                bit,
                end,
                stream,
                level,
                symbols,
            }) => {
                let span = BlockSpan {
                    index,
                    start_bit: bit,
                    end_bit: end,
                    crc: symbols.crc,
                    stream,
                    level,
                };
                self.syms = symbols.syms;
                Some(Ok(span))
            }
            Ok(Event::Done) => {
                self.done = true;
                None
            }
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}
