//! Benchmark-only sequential adapter, matching BzReader's input/output buffering.
//! Kept here so adding the cells arm does not change any existing reader or team path.
use mibz2::stream::{Event, Scanner, Source};
use mibz2::{bwt::Cells, decode_block, Error, ErrorKind, Kernels, Scratch};
use std::io::{self, Read};
const CHUNK: usize = 4 << 20;
const COMPACT: usize = 4 << 20;
pub struct CellsReader<R> {
    scanner: Scanner<ReadSource<R>>,
    kernels: Kernels,
    scratch: Scratch<Cells>,
    syms: Vec<u16>,
    out: Vec<u8>,
    at: usize,
    done: bool,
    failed: Option<Error>,
    #[cfg(test)]
    blocks: usize,
}

impl<R: Read> CellsReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            scanner: Scanner::new(ReadSource {
                r,
                buf: Vec::new(),
                base: 0,
                released: 0,
                eof: false,
            }),
            kernels: Kernels::detect(),
            scratch: Scratch::default(),
            syms: Vec::new(),
            out: Vec::new(),
            at: 0,
            done: false,
            failed: None,
            #[cfg(test)]
            blocks: 0,
        }
    }
}
impl<R: Read> Read for CellsReader<R> {
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
                    #[cfg(test)]
                    {
                        self.blocks += 1;
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the seven hash-verified inputs; explicit untimed pre-benchmark check"]
    fn cells_pinned_readers_byte_parity() {
        fn verify(reader: &mut dyn Read, expected: &[u8], name: &str) {
            let mut offset = 0;
            let mut buffer = vec![0; 4 << 20];
            loop {
                let count = reader.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                assert_eq!(
                    &buffer[..count],
                    &expected[offset..offset + count],
                    "{name} at {offset}"
                );
                offset += count;
            }
            assert_eq!(offset, expected.len(), "{name}");
        }
        let root = std::path::PathBuf::from(std::env::var_os("CELLS_PINNED_INPUTS").unwrap());
        for name in [
            "wikidata", "onebyte", "pattern", "random", "records", "runs", "text",
        ] {
            let path = root.join(format!("{name}.bz2"));
            let mut expected = Vec::new();
            bzip2::read::MultiBzDecoder::new(std::fs::File::open(&path).unwrap())
                .read_to_end(&mut expected)
                .unwrap();
            verify(
                &mut mibz2::BzReader::new(std::fs::File::open(&path).unwrap()),
                &expected,
                name,
            );
            let mut cells = CellsReader::new(std::fs::File::open(&path).unwrap());
            verify(&mut cells, &expected, name);
            let counts = cells.scratch.cells_counts();
            let attempted =
                counts.admitted + counts.density + counts.floor + counts.budget + counts.invalid;
            let column_fallback = cells.blocks - attempted;
            eprintln!(
                "CELLS_ROUTE {name} blocks={} column_fallback={column_fallback} {counts:?}",
                cells.blocks
            );
        }
    }

    #[test]
    fn cells_arm_matches_reader_bytes_and_errors() {
        use crate::common::{compress, craft, Spec};
        let mut bad = Spec::of(vec![1; 101], 1);
        bad.crc = Some(123);
        let cases = [
            compress(&b"abacaba".repeat(20_000), 9),
            [compress(b"first", 1), compress(b"second", 9)].concat(),
            craft(9, &[Spec::of(vec![0; 25], 1), bad], None),
            b"BZh9truncated".to_vec(),
        ];
        fn collect(mut r: impl Read, chunk: usize) -> (Vec<u8>, Option<String>) {
            let mut out = Vec::new();
            let mut buf = vec![0; chunk];
            loop {
                match r.read(&mut buf) {
                    Ok(0) => return (out, None),
                    Ok(n) => out.extend_from_slice(&buf[..n]),
                    Err(e) => return (out, Some(format!("{e:?}"))),
                }
            }
        }
        for z in cases {
            for chunk in [1, 17, 4 << 20] {
                assert_eq!(
                    collect(CellsReader::new(&z[..]), chunk),
                    collect(mibz2::BzReader::new(&z[..]), chunk)
                );
            }
        }
        assert!(!crate::parallel("mibz2-cells"));
        assert!(crate::parallel("mibz2"));
    }
}
