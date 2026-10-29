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
from typing import IO, Union

from ._mibz2 import DecodeError, Reader, decompress

__all__ = ["BZ2Reader", "DecodeError", "decompress", "open"]

#: What a reader reads from: a path, or a binary file object open for reading
Source = Union[str, "os.PathLike[str]", IO[bytes], io.RawIOBase, io.BufferedIOBase]


class BZ2Reader(io.RawIOBase):
    """A .bz2 file's decoded bytes, read in order (an unbuffered binary file).

    Parameters
    ----------
    source
        A path, or a binary file object open for reading (its ``read`` is called).
    threads
        Decoding threads: ``1`` decodes on the reading thread; more decode blocks at
        once on worker threads, a block each, or, on 6 or more physical cores, in teams
        of 2 sharing each block (as many teams as keep their tables in the CPU's last
        cache level, or one a core pair for a stream of level 1-3, whose blocks are
        small). ``None`` (the default) uses every CPU this process may use.

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
        self._reader = Reader(source, threads)

    def readable(self) -> bool:
        """True: a reader is readable."""
        return True

    def readinto(self, b) -> int:
        """Read up to ``len(b)`` decoded bytes into ``b``; 0 at the end (when verified)."""
        data = self._reader.read(len(b))
        n = len(data)
        memoryview(b).cast("B")[:n] = data
        return n

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
    return io.BufferedReader(BZ2Reader(source, threads), buffer_size)
