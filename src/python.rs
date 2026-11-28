//! The Python extension module (feature `python`): `mibz2._mibz2`, wrapped by the
//! `mibz2` package (python/mibz2/)

use std::fs::File;
use std::io::{self, BufRead, Read};
use std::sync::{Arc, Mutex};

use pyo3::create_exception;
use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::pybacked::PyBackedBytes;
use pyo3::types::{PyBytes, PyList};

use crate::{BzReader, Error, ErrorKind};

create_exception!(
    _mibz2,
    DecodeError,
    PyValueError,
    "The input is not valid bzip2 data"
);

/// The worker threads for `threads`: all the CPUs this process may use when `None`
fn threads_or_all(threads: Option<usize>) -> usize {
    threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()))
        .max(1)
}

fn decode_err(e: Error) -> PyErr {
    match e.kind {
        ErrorKind::Io(_, msg) => PyOSError::new_err(msg),
        _ => DecodeError::new_err(e.to_string()),
    }
}

/// Safe 8-byte SWAR newline search.
#[inline]
fn find_newline(haystack: &[u8]) -> Option<usize> {
    const NL_MASK: u64 = 0x0a0a_0a0a_0a0a_0a0a;
    const LO_BITS: u64 = 0x0101_0101_0101_0101;
    const HI_BITS: u64 = 0x8080_8080_8080_8080;

    let chunks = haystack.chunks_exact(8);
    let rem = chunks.remainder();
    let mut off = 0usize;
    for chunk in chunks {
        let word = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        let x = word ^ NL_MASK;
        let has_zero = x.wrapping_sub(LO_BITS) & !x & HI_BITS;
        if has_zero != 0 {
            return Some(off + (has_zero.trailing_zeros() as usize >> 3));
        }
        off += 8;
    }
    for (i, &b) in rem.iter().enumerate() {
        if b == b'\n' {
            return Some(off + i);
        }
    }
    None
}

/// Retry `fill_buf` across `Interrupted` (non-blocking Python sources returning `None`),
/// returning the number of available bytes (`0` at EOF).
fn fill_buf_len<R: Read>(reader: &mut BzReader<R>) -> io::Result<usize> {
    loop {
        match reader.fill_buf() {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Ok(avail) => return Ok(avail.len()),
            Err(e) => return Err(e),
        }
    }
}

/// A Python binary file object (its `read`) as a `Read`. A Python exception it raises is
/// kept, and raised again in place of the `io::Error` it became
struct PyFile {
    obj: Py<PyAny>,
    raised: Arc<Mutex<Option<PyErr>>>,
}

impl Read for PyFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let res = self
                .obj
                .call_method1(py, intern!(py, "read"), (buf.len(),))
                .and_then(|chunk| {
                    let chunk = chunk.bind(py);
                    if chunk.is_none() {
                        // a non-blocking file with nothing to read yet: read again
                        return Ok(None);
                    }
                    let data: PyBackedBytes = chunk.extract()?;
                    let n = data.len();
                    if n > buf.len() {
                        return Err(PyValueError::new_err(format!(
                            "read({}) returned {n} bytes",
                            buf.len()
                        )));
                    }
                    buf[..n].copy_from_slice(&data);
                    Ok(Some(n))
                });
            match res {
                Ok(Some(n)) => Ok(n),
                Ok(None) => Err(io::Error::new(io::ErrorKind::Interrupted, "no data yet")),
                Err(e) => {
                    *self.raised.lock().unwrap_or_else(|x| x.into_inner()) = Some(e);
                    Err(io::Error::other("the source's read raised an exception"))
                }
            }
        })
    }
}

type Source = Box<dyn Read + Send>;

struct ReaderState {
    reader: BzReader<Source>,
    /// Number of unread bytes currently buffered inside `reader` from the last
    /// `fill_buf`. While `avail_rem > 0`, `reader.fill_buf()` returns those bytes
    /// immediately without I/O, decoding, or blocking, so the GIL need not be detached.
    avail_rem: usize,
}

/// The decoder behind `mibz2.BZ2Reader`: a .bz2 file's decoded bytes, read in order
#[pyclass(module = "mibz2._mibz2")]
struct Reader {
    inner: Mutex<Option<ReaderState>>,
    raised: Arc<Mutex<Option<PyErr>>>,
    finished: Mutex<bool>,
}

impl Reader {
    fn map_io_err(&self, e: io::Error) -> PyErr {
        if let Some(err) = self.raised.lock().unwrap_or_else(|x| x.into_inner()).take() {
            return err;
        }
        match e.into_inner().map(|inner| inner.downcast::<Error>()) {
            Some(Ok(err)) => decode_err(*err),
            Some(Err(other)) => PyOSError::new_err(other.to_string()),
            None => PyOSError::new_err("reading the input failed"),
        }
    }

    #[inline]
    fn ensure_buf(&self, py: Python<'_>, state: &mut ReaderState) -> PyResult<usize> {
        if state.avail_rem > 0 {
            return Ok(state.avail_rem);
        }
        let n = py
            .detach(|| fill_buf_len(&mut state.reader))
            .map_err(|e| self.map_io_err(e))?;
        state.avail_rem = n;
        if n == 0 {
            *self.finished.lock().unwrap_or_else(|x| x.into_inner()) = true;
        }
        Ok(n)
    }

    fn readline_inner<'py>(
        &self,
        py: Python<'py>,
        state: &mut ReaderState,
        limit: Option<usize>,
    ) -> PyResult<Option<Bound<'py, PyBytes>>> {
        if limit == Some(0) {
            return Ok(Some(PyBytes::new(py, &[])));
        }
        let avail_len = self.ensure_buf(py, state)?;
        if avail_len == 0 {
            return Ok(None);
        }
        // Fast path: line ends (or hits `limit`) inside the currently decoded block.
        let avail = state.reader.fill_buf().map_err(|e| self.map_io_err(e))?;
        let max_take = limit.map_or(avail.len(), |l| avail.len().min(l));
        if let Some(pos) = find_newline(&avail[..max_take]) {
            let take = pos + 1;
            let bytes = PyBytes::new(py, &avail[..take]);
            state.reader.consume(take);
            state.avail_rem -= take;
            return Ok(Some(bytes));
        }
        if limit == Some(max_take) {
            let bytes = PyBytes::new(py, &avail[..max_take]);
            state.reader.consume(max_take);
            state.avail_rem -= max_take;
            return Ok(Some(bytes));
        }

        // Slow path: line spans a decoded block boundary.
        let mut line = Vec::with_capacity(max_take.saturating_mul(2).min(1 << 20));
        line.extend_from_slice(&avail[..max_take]);
        state.reader.consume(max_take);
        state.avail_rem = 0;

        let mut rem_limit = limit.map(|l| l - max_take);
        loop {
            let next_len = self.ensure_buf(py, state)?;
            if next_len == 0 {
                break;
            }
            let avail = state.reader.fill_buf().map_err(|e| self.map_io_err(e))?;
            let max_take = rem_limit.map_or(avail.len(), |l| avail.len().min(l));
            if let Some(pos) = find_newline(&avail[..max_take]) {
                let take = pos + 1;
                line.extend_from_slice(&avail[..take]);
                state.reader.consume(take);
                state.avail_rem -= take;
                break;
            }
            line.extend_from_slice(&avail[..max_take]);
            state.reader.consume(max_take);
            state.avail_rem -= max_take;
            if let Some(ref mut r) = rem_limit {
                *r -= max_take;
                if *r == 0 {
                    break;
                }
            }
        }
        Ok(Some(PyBytes::new(py, &line)))
    }
}

#[pymethods]
impl Reader {
    /// `source`: a path (str, from os.fspath) or a binary file object. `threads`: 1
    /// decodes on the reading thread; more, that many blocks at once on worker threads
    #[new]
    #[pyo3(signature = (source, threads=None))]
    fn new(source: &Bound<'_, PyAny>, threads: Option<usize>) -> PyResult<Self> {
        let raised = Arc::new(Mutex::new(None));
        let src: Source = match source.extract::<String>() {
            Ok(path) => {
                Box::new(File::open(&path).map_err(|e| PyOSError::new_err(format!("{path}: {e}")))?)
            }
            Err(_) => Box::new(PyFile {
                obj: source.clone().unbind(),
                raised: Arc::clone(&raised),
            }),
        };
        let threads = threads_or_all(threads);
        let reader = if threads == 1 {
            BzReader::new(src)
        } else {
            BzReader::parallel(src, threads)
        };
        Ok(Reader {
            inner: Mutex::new(Some(ReaderState {
                reader,
                avail_rem: 0,
            })),
            raised,
            finished: Mutex::new(false),
        })
    }

    /// Up to `size` decoded bytes from at most one decoded block; empty at EOF, once
    /// every stream's CRC has been checked
    fn read<'py>(&self, py: Python<'py>, size: usize) -> PyResult<Bound<'py, PyBytes>> {
        let mut guard = self.inner.lock().unwrap_or_else(|x| x.into_inner());
        let state = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("I/O operation on closed file"))?;
        if size == 0 {
            if state.avail_rem > 0 {
                return Ok(PyBytes::new(py, &[]));
            }
            let res = py.detach(|| loop {
                match state.reader.read(&mut []) {
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    other => break other,
                }
            });
            return match res {
                Ok(_) => Ok(PyBytes::new(py, &[])),
                Err(e) => Err(self.map_io_err(e)),
            };
        }
        let avail_len = self.ensure_buf(py, state)?;
        if avail_len == 0 {
            return Ok(PyBytes::new(py, &[]));
        }
        let avail = state.reader.fill_buf().map_err(|e| self.map_io_err(e))?;
        let n = size.min(avail.len());
        let bytes = PyBytes::new(py, &avail[..n]);
        state.reader.consume(n);
        state.avail_rem -= n;
        Ok(bytes)
    }

    /// Read up to `size` decoded bytes, looping across blocks until `size` bytes are
    /// gathered or EOF is reached (matching `io.BufferedReader.read(size)` semantics)
    fn read_full<'py>(&self, py: Python<'py>, size: usize) -> PyResult<Bound<'py, PyBytes>> {
        let mut guard = self.inner.lock().unwrap_or_else(|x| x.into_inner());
        let state = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("I/O operation on closed file"))?;
        if size == 0 {
            if state.avail_rem > 0 {
                return Ok(PyBytes::new(py, &[]));
            }
            let res = py.detach(|| loop {
                match state.reader.read(&mut []) {
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    other => break other,
                }
            });
            return match res {
                Ok(_) => Ok(PyBytes::new(py, &[])),
                Err(e) => Err(self.map_io_err(e)),
            };
        }
        let avail_len = self.ensure_buf(py, state)?;
        if avail_len == 0 {
            return Ok(PyBytes::new(py, &[]));
        }
        if avail_len >= size {
            let avail = state.reader.fill_buf().map_err(|e| self.map_io_err(e))?;
            let bytes = PyBytes::new(py, &avail[..size]);
            state.reader.consume(size);
            state.avail_rem -= size;
            return Ok(bytes);
        }

        // Cross-block read: gather up to `size` bytes in one GIL-detached loop.
        let (out, eof, rem_avail) = py
            .detach(|| -> io::Result<(Vec<u8>, bool, usize)> {
                let mut out = Vec::with_capacity(size.min(8 << 20));
                let mut eof = false;
                let mut rem_avail = 0usize;
                while out.len() < size {
                    let avail = match state.reader.fill_buf() {
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Ok(a) => a,
                        Err(e) => return Err(e),
                    };
                    if avail.is_empty() {
                        eof = true;
                        rem_avail = 0;
                        break;
                    }
                    let want = (size - out.len()).min(avail.len());
                    rem_avail = avail.len() - want;
                    out.extend_from_slice(&avail[..want]);
                    state.reader.consume(want);
                }
                Ok((out, eof, rem_avail))
            })
            .map_err(|e| self.map_io_err(e))?;

        state.avail_rem = rem_avail;
        if eof {
            *self.finished.lock().unwrap_or_else(|x| x.into_inner()) = true;
        }
        let bytes = PyBytes::new(py, &out);
        py.detach(|| drop(out));
        Ok(bytes)
    }

    /// Return buffered bytes without advancing the stream position (at least 1 byte
    /// unless EOF)
    #[pyo3(signature = (size=0))]
    fn peek<'py>(&self, py: Python<'py>, size: usize) -> PyResult<Bound<'py, PyBytes>> {
        let _ = size;
        let mut guard = self.inner.lock().unwrap_or_else(|x| x.into_inner());
        let state = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("I/O operation on closed file"))?;
        let avail_len = self.ensure_buf(py, state)?;
        if avail_len == 0 {
            return Ok(PyBytes::new(py, &[]));
        }
        let avail = state.reader.fill_buf().map_err(|e| self.map_io_err(e))?;
        Ok(PyBytes::new(py, avail))
    }

    /// Read and return one line (ending in `\n`), or up to `size` bytes if `size >= 0`
    #[pyo3(signature = (size=None))]
    fn readline<'py>(&self, py: Python<'py>, size: Option<isize>) -> PyResult<Bound<'py, PyBytes>> {
        let mut guard = self.inner.lock().unwrap_or_else(|x| x.into_inner());
        let state = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("I/O operation on closed file"))?;
        let limit = match size {
            None => None,
            Some(n) if n < 0 => None,
            Some(n) => Some(n as usize),
        };
        match self.readline_inner(py, state, limit)? {
            Some(line) => Ok(line),
            None => Ok(PyBytes::new(py, &[])),
        }
    }

    /// Read and return a list of lines; stop once at least `hint` bytes have been read if
    /// `hint > 0`. Scans each decoded block in a tight inner loop (one consume per block)
    #[pyo3(signature = (hint=None))]
    fn readlines<'py>(&self, py: Python<'py>, hint: Option<isize>) -> PyResult<Bound<'py, PyList>> {
        let mut guard = self.inner.lock().unwrap_or_else(|x| x.into_inner());
        let state = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("I/O operation on closed file"))?;
        let limit_bytes = match hint {
            Some(n) if n > 0 => Some(n as usize),
            _ => None,
        };
        let list = PyList::empty(py);
        let mut total = 0usize;
        let mut carry: Vec<u8> = Vec::new();
        loop {
            let avail_len = self.ensure_buf(py, state)?;
            if avail_len == 0 {
                if !carry.is_empty() {
                    list.append(PyBytes::new(py, &carry))?;
                }
                break;
            }
            let avail = state.reader.fill_buf().map_err(|e| self.map_io_err(e))?;
            let mut pos = 0usize;
            let mut hit_hint = false;
            while pos < avail.len() {
                if let Some(rel) = find_newline(&avail[pos..]) {
                    let end = pos + rel + 1;
                    let line_len;
                    if carry.is_empty() {
                        let slice = &avail[pos..end];
                        line_len = slice.len();
                        list.append(PyBytes::new(py, slice))?;
                    } else {
                        carry.extend_from_slice(&avail[pos..end]);
                        line_len = carry.len();
                        list.append(PyBytes::new(py, &carry))?;
                        carry.clear();
                    }
                    pos = end;
                    total = total.saturating_add(line_len);
                    if let Some(h) = limit_bytes {
                        if total >= h {
                            hit_hint = true;
                            break;
                        }
                    }
                } else {
                    carry.extend_from_slice(&avail[pos..]);
                    pos = avail.len();
                    break;
                }
            }
            state.reader.consume(pos);
            state.avail_rem -= pos;
            if hit_hint {
                break;
            }
        }
        Ok(list)
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyBytes>>> {
        let mut guard = self.inner.lock().unwrap_or_else(|x| x.into_inner());
        let state = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("I/O operation on closed file"))?;
        self.readline_inner(py, state, None)
    }

    /// True once the reader has reached the end and every block's and stream's CRC
    /// matched
    #[getter]
    fn verified(&self) -> bool {
        *self.finished.lock().unwrap_or_else(|x| x.into_inner())
    }

    /// Stop decoding (its worker threads stop) and drop the source
    fn close(&self, py: Python<'_>) {
        let state = self.inner.lock().unwrap_or_else(|x| x.into_inner()).take();
        py.detach(|| drop(state));
    }
}

/// A whole .bz2 (its streams one after another) decoded, every block's and stream's CRC
/// checked: on all the CPUs this process may use, or on `threads`
#[pyfunction]
#[pyo3(signature = (data, threads=None))]
fn decompress<'py>(
    py: Python<'py>,
    data: PyBackedBytes,
    threads: Option<usize>,
) -> PyResult<Bound<'py, PyBytes>> {
    let input: &[u8] = &data;
    let threads = threads_or_all(threads);
    let out = py
        .detach(|| {
            if threads == 1 {
                crate::decode(input)
            } else {
                crate::decode_parallel(input, threads)
            }
        })
        .map_err(decode_err)?;
    let bytes = PyBytes::new(py, &out);
    py.detach(|| drop(out));
    Ok(bytes)
}

#[pymodule]
fn _mibz2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decompress, m)?)?;
    m.add_class::<Reader>()?;
    m.add("DecodeError", m.py().get_type::<DecodeError>())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::find_newline;

    #[test]
    fn find_newline_swar_matches_scalar_all_bytes_and_alignments() {
        assert_eq!(find_newline(b""), None);
        assert_eq!(find_newline(b"\n"), Some(0));
        assert_eq!(find_newline(b"abcdefg"), None);
        assert_eq!(find_newline(b"abcdefgh"), None);

        // No non-newline byte produces a false positive in any lane (including 0x00, 0x09,
        // 0x0b, 0x8a, 0xff and adjacent pairs that exercise SWAR borrows).
        let non_nl: Vec<u8> = (0u8..=255).filter(|&b| b != b'\n').collect();
        for start in 0..16 {
            assert_eq!(find_newline(&non_nl[start..]), None);
        }

        // Exact position across every offset 0..32 with varied prefix/suffix bytes.
        for len in 1..=33 {
            for pos in 0..len {
                for fill in [0x00u8, 0x09, 0x0b, 0x7f, 0x80, 0x8a, 0xff] {
                    let mut buf = vec![fill; len];
                    buf[pos] = b'\n';
                    assert_eq!(
                        find_newline(&buf),
                        Some(pos),
                        "len={len} pos={pos} fill={fill:#x}"
                    );
                    // Second newline after `pos` still returns first `pos`.
                    if pos + 1 < len {
                        buf[len - 1] = b'\n';
                        assert_eq!(find_newline(&buf), Some(pos));
                    }
                }
            }
        }
    }
}
