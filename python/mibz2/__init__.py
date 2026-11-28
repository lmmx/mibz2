"""mibz2: a fast bzip2 decoder, block-parallel and multi-chain within each block.

- [`decompress`][mibz2.decompress]: a whole .bz2 in memory, decoded.
- [`open`][mibz2.open]: a .bz2 file (a path or a binary file object) read as a
  buffered binary file, decoded as it is read.

Every block's CRC is checked before any of that block's bytes are returned, and each
stream's combined CRC is checked at the stream's end. Bytes read from a stream before
its end come from blocks whose own CRCs matched, but the stream as a whole is verified
only when a read reaches its end: until then, treat what has been read as provisional
(see [`BZ2Reader.verified`][mibz2.BZ2Reader.verified]).
"""

from __future__ import annotations

import io
import os
from collections.abc import Iterator
from typing import IO, TYPE_CHECKING, Any, NoReturn, Union, cast

from ._mibz2 import DecodeError, Reader, decompress

__all__ = ["BZ2Reader", "DecodeError", "decompress", "open"]

#: What a reader reads from: a path, or a binary file object open for reading
Source = Union[str, "os.PathLike[str]", IO[bytes], io.RawIOBase, io.BufferedIOBase]

#: Chunk size for whole-stream ``readall`` (1 MiB, above the 900 kB max bzip2 block).
_READALL_CHUNK = 1 << 20


def _copy_into(b: Any, data: bytes) -> int:
    """Copy ``data`` into writable buffer ``b`` and return the number of bytes copied."""
    n = len(data)
    if n == 0:
        return 0
    if isinstance(b, memoryview) and b.ndim == 1 and b.format == "B":
        b[:n] = data
    elif isinstance(b, bytearray):
        memoryview(b)[:n] = data
    else:
        memoryview(b).cast("B")[:n] = data
    return n


def _readall_from(reader: Reader) -> bytes:
    """Drain all remaining decoded bytes from ``reader`` in 1 MiB chunks."""
    chunks: list[bytes] = []
    read = reader.read
    while chunk := read(_READALL_CHUNK):
        chunks.append(chunk)
    if not chunks:
        return b""
    if len(chunks) == 1:
        return chunks[0]
    return b"".join(chunks)


def _detached(*_args: Any, **_kwargs: Any) -> NoReturn:
    """Raise ``ValueError`` when a detached buffered reader is used."""
    raise ValueError("raw stream has been detached")


class BZ2Reader(io.RawIOBase):
    """A .bz2 file's decoded bytes, read in order (an unbuffered binary file).

    Parameters
    ----------
    source
        A path, or a binary file object open for reading (its ``read`` is called).
    threads
        Decoding threads: ``1`` decodes on the reading thread; more decode blocks at
        once, as the Rust crate's ``Plan::for_threads`` spends them: worker threads, a
        block each (past the physical cores on 8 to 10 of them, one a core; on the
        i9-10900K, one a core but one), or teams of 2 sharing each block (from 11 threads
        on 11 or more cores, or past the cores on 6 or 7).
        ``None`` (the default) asks for every CPU this process may use.

    Raises
    ------
    DecodeError
        From a read, when the input is not valid bzip2 data: a block whose CRC does not
        match (none of its bytes are returned), a stream whose combined CRC does not
        match (raised by the read that reaches the stream's end), or a malformed or
        truncated stream. The error says which block and bit it was found at, which is
        where decoding failed and need not be where the input was damaged.
    """

    def __init__(self, source: Source, threads: int | None = None) -> None:
        super().__init__()
        if isinstance(source, (str, os.PathLike)):
            source = os.fspath(source)
        reader = Reader(source, threads)
        self._reader = reader
        # Bind hot C methods directly on the instance to avoid a Python call hop per
        # line while preserving ValueError on closed readers (enforced in Rust).
        self.__dict__["readline"] = reader.readline
        self.__dict__["readlines"] = reader.readlines

    def readable(self) -> bool:
        """True: a reader is readable."""
        return True

    def readinto(self, b: Any) -> int:
        """Read up to ``len(b)`` decoded bytes into ``b``; 0 at the end (when verified)."""
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return _copy_into(b, self._reader.read(len(b)))

    def readall(self) -> bytes:
        """Read and return all remaining decoded bytes until EOF."""
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return _readall_from(self._reader)

    def read(self, size: int | None = -1) -> bytes:
        """Read up to ``size`` bytes, or all remaining bytes if ``size`` is negative or ``None``."""
        if size is None or size < 0:
            return self.readall()
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return self._reader.read(size)

    def readline(self, size: int | None = -1) -> bytes:
        """Read and return one line, or up to ``size`` bytes if ``size >= 0``."""
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return self._reader.readline(size)

    def readlines(self, hint: int = -1) -> list[bytes]:
        """Read and return a list of lines; stop after ``hint`` bytes if ``hint > 0``."""
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return self._reader.readlines(hint)

    def __iter__(self) -> Iterator[bytes]:
        """Iterate decoded lines directly over the Rust block buffer."""
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return self._reader

    def __next__(self) -> bytes:
        """Return the next decoded line, or raise ``StopIteration`` at EOF."""
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return next(self._reader)

    @property
    def verified(self) -> bool:
        """True once a read has reached the end and every block's and stream's CRC matched.

        Until then, the bytes read so far have each passed their block's CRC, but the
        stream's combined CRC has not yet been checked.
        """
        return self._reader.verified

    def close(self) -> None:
        """Stop decoding (its worker threads stop) and close the reader."""
        if not self.closed:
            self._reader.close()
        super().close()


if TYPE_CHECKING:
    _BufferedBase = io.BufferedReader[BZ2Reader]
else:
    _BufferedBase = io.BufferedReader


class _BZ2BufferedReader(_BufferedBase):
    """Buffered reader delegating directly to Rust's block-buffered ``Reader``.

    Preserves ``isinstance(..., io.BufferedReader)``, ``.raw``, ``.raw.verified``,
    ``detach()``, and ``io.TextIOWrapper`` compatibility without double-buffering
    through ``io.BufferedReader``'s C buffer.
    """

    def __init__(self, raw: BZ2Reader, buffer_size: int = 1 << 20) -> None:
        if buffer_size <= 0:
            raise ValueError("buffer size must be strictly positive")
        super().__init__(raw, 1)
        reader = raw._reader
        self._reader: Reader | None = reader
        self._buf_size = buffer_size
        self.__dict__["readline"] = reader.readline
        self.__dict__["readlines"] = reader.readlines
        self.__dict__["peek"] = reader.peek

    def _active_reader(self) -> Reader:
        """Return the underlying Rust ``Reader`` or raise if detached."""
        reader = self._reader
        if reader is None:
            raise ValueError("raw stream has been detached")
        return reader

    def detach(self) -> BZ2Reader:
        """Separate and return the underlying raw ``BZ2Reader``."""
        self._reader = None
        self.__dict__["readline"] = _detached
        self.__dict__["readlines"] = _detached
        self.__dict__["peek"] = _detached
        return cast(BZ2Reader, super().detach())

    def read(self, size: int | None = -1) -> bytes:
        """Read up to ``size`` bytes, or all remaining bytes if ``size`` is negative or ``None``."""
        reader = self._active_reader()
        if size is None or size < 0:
            return _readall_from(reader)
        return reader.read_full(size)

    def read1(self, size: int = -1) -> bytes:
        """Read up to ``size`` bytes from at most one decoded block."""
        reader = self._active_reader()
        if size < 0:
            size = self._buf_size
        return reader.read(size)

    def readinto(self, b: Any) -> int:
        """Read decoded bytes into ``b`` until full or EOF; return bytes read."""
        reader = self._active_reader()
        if isinstance(b, memoryview) and b.ndim == 1 and b.format == "B":
            mv = b
        elif isinstance(b, bytearray):
            mv = memoryview(b)
        else:
            mv = memoryview(b).cast("B")
        cap = len(mv)
        if cap == 0:
            reader.read(0)
            return 0
        total = 0
        read = reader.read
        while total < cap:
            chunk = read(cap - total)
            n = len(chunk)
            if n == 0:
                break
            mv[total : total + n] = chunk
            total += n
        return total

    def readinto1(self, b: Any) -> int:
        """Read bytes from at most one decoded block into ``b``; return bytes read."""
        reader = self._active_reader()
        return _copy_into(b, reader.read(len(b)))

    def peek(self, size: int = 0) -> bytes:
        """Return buffered bytes without advancing the stream position."""
        return self._active_reader().peek(size)

    def readline(self, size: int | None = -1) -> bytes:
        """Read and return one line, or up to ``size`` bytes if ``size >= 0``."""
        return self._active_reader().readline(size)

    def readlines(self, hint: int = -1) -> list[bytes]:
        """Read and return a list of lines; stop after ``hint`` bytes if ``hint > 0``."""
        return self._active_reader().readlines(hint)

    def __iter__(self) -> Iterator[bytes]:
        """Iterate decoded lines directly over the Rust block buffer."""
        reader = self._active_reader()
        if self.closed:
            raise ValueError("I/O operation on closed file")
        return reader

    def __next__(self) -> bytes:
        """Return the next decoded line, or raise ``StopIteration`` at EOF."""
        return next(self._active_reader())


def open(  # noqa: A001
    source: Source,
    threads: int | None = None,
    buffer_size: int = 1 << 20,
) -> io.BufferedReader[BZ2Reader]:
    r"""Open a .bz2 file for reading, decoded as it is read.

    Parameters
    ----------
    source
        A path, or a binary file object open for reading.
    threads
        Decoding threads, as for [`BZ2Reader`][mibz2.BZ2Reader].
    buffer_size
        The buffered reader's buffer, in bytes.

    Returns
    -------
    io.BufferedReader
        A buffered binary file over a [`BZ2Reader`][mibz2.BZ2Reader] (its ``raw``), whose
        ``raw.verified`` says whether the whole input has been read and verified.

    Examples
    --------
    >>> import bz2, mibz2, tempfile, os
    >>> path = os.path.join(tempfile.mkdtemp(), "a.bz2")
    >>> with bz2.open(path, "wb") as f:
    ...     _ = f.write(b"line one\nline two\n")
    >>> with mibz2.open(path) as f:
    ...     lines = f.readlines()
    ...     verified = f.raw.verified
    >>> lines, verified
    ([b'line one\n', b'line two\n'], True)
    """
    return _BZ2BufferedReader(BZ2Reader(source, threads), buffer_size)
