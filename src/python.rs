//! The Python extension module (feature `python`): `mibz2._mibz2`, wrapped by the
//! `mibz2` package (python/mibz2/)

use std::fs::File;
use std::io::{self, Read};
use std::sync::{Arc, Mutex};

use pyo3::create_exception;
use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::pybacked::PyBackedBytes;
use pyo3::types::PyBytes;

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

/// A Python binary file object (its `readinto`, else its `read`) as a `Read`. A Python
/// exception it raises is kept, and raised again in place of the `io::Error` it became
struct PyFile {
    obj: Py<PyAny>,
    raised: Arc<Mutex<Option<PyErr>>>,
}

impl Read for PyFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let res = self
                .obj
                .call_method1(py, "read", (buf.len(),))
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

/// The decoder behind `mibz2.BZ2Reader`: a .bz2 file's decoded bytes, read in order
#[pyclass(module = "mibz2._mibz2")]
struct Reader {
    inner: Mutex<Option<BzReader<Source>>>,
    raised: Arc<Mutex<Option<PyErr>>>,
    finished: Mutex<bool>,
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
            inner: Mutex::new(Some(reader)),
            raised,
            finished: Mutex::new(false),
        })
    }

    /// Up to `size` decoded bytes; empty at the end, once every stream's CRC has been
    /// checked
    fn read<'py>(&self, py: Python<'py>, size: usize) -> PyResult<Bound<'py, PyBytes>> {
        let mut buf = vec![0u8; size];
        let mut guard = self.inner.lock().unwrap_or_else(|x| x.into_inner());
        let reader = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("I/O operation on closed file"))?;
        let res = py.detach(|| loop {
            match reader.read(&mut buf) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                other => break other,
            }
        });
        match res {
            Ok(n) => {
                if n == 0 && size > 0 {
                    *self.finished.lock().unwrap_or_else(|x| x.into_inner()) = true;
                }
                Ok(PyBytes::new(py, &buf[..n]))
            }
            Err(e) => {
                if let Some(err) = self.raised.lock().unwrap_or_else(|x| x.into_inner()).take() {
                    return Err(err);
                }
                match e.into_inner().map(|inner| inner.downcast::<Error>()) {
                    Some(Ok(err)) => Err(decode_err(*err)),
                    Some(Err(other)) => Err(PyOSError::new_err(other.to_string())),
                    None => Err(PyOSError::new_err("reading the input failed")),
                }
            }
        }
    }

    /// True once the reader has reached the end and every block's and stream's CRC
    /// matched
    #[getter]
    fn verified(&self) -> bool {
        *self.finished.lock().unwrap_or_else(|x| x.into_inner())
    }

    /// Stop decoding (its worker threads stop) and drop the source
    fn close(&self, py: Python<'_>) {
        let reader = self.inner.lock().unwrap_or_else(|x| x.into_inner()).take();
        py.detach(|| drop(reader));
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
    Ok(PyBytes::new(py, &out))
}

#[pymodule]
fn _mibz2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decompress, m)?)?;
    m.add_class::<Reader>()?;
    m.add("DecodeError", m.py().get_type::<DecodeError>())?;
    Ok(())
}
