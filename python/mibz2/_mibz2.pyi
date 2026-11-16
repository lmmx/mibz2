from typing import Any

class DecodeError(ValueError):
    """The input is not valid bzip2 data."""

def decompress(data: bytes | bytearray, threads: int | None = None) -> bytes:
    """Decode a whole .bz2 (its streams one after another), every CRC checked.

    It runs on every CPU this process may use, or on ``threads``, as the Rust crate's
    ``Plan::for_threads`` spends them: a block a worker (past the physical cores on 8
    to 10 of them, one a core; on the i9-10900K, one a core but one), or teams of 2
    sharing each block (from 11 threads on 11 or more cores, or past the cores on 6 or 7).

    Raises
    ------
    DecodeError
        When the input is not valid bzip2 data.
    """

class Reader:
    def __init__(self, source: str | Any, threads: int | None = None) -> None: ...
    def read(self, size: int) -> bytes: ...
    @property
    def verified(self) -> bool: ...
    def close(self) -> None: ...
