from typing import Any

class DecodeError(ValueError):
    """The input is not valid bzip2 data."""

def decompress(data: bytes | bytearray, threads: int | None = None) -> bytes:
    """Decode a whole .bz2 (its streams one after another), every CRC checked.

    It runs on every CPU this process may use, or on ``threads``: a block a worker,
    or teams of 2 sharing each block on 6 or more physical cores.

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
