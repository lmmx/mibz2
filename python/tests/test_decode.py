import bz2
import io
import os
import random

import pytest

import mibz2


def sample(n: int = 3_000_000, seed: int = 0) -> bytes:
    rng = random.Random(seed)
    words = [bytes(rng.choice(b"abcdefghij") for _ in range(rng.randint(1, 9))) for _ in range(500)]
    out = bytearray()
    while len(out) < n:
        out += rng.choice(words) + b" "
    return bytes(out[:n])


DATA = sample()
# 100 kB blocks, so the sample spans many blocks
COMPRESSED = bz2.compress(DATA, compresslevel=1)


@pytest.mark.parametrize("threads", [1, 4, None])
def test_decompress(threads):
    assert mibz2.decompress(COMPRESSED, threads=threads) == DATA


def test_decompress_bytearray_and_multistream():
    two = bytearray(COMPRESSED + bz2.compress(b"tail"))
    assert mibz2.decompress(two) == DATA + b"tail"


@pytest.mark.parametrize("threads", [1, 4])
def test_open_path(tmp_path, threads):
    path = tmp_path / "a.bz2"
    path.write_bytes(COMPRESSED)
    with mibz2.open(path, threads=threads) as f:
        assert not f.raw.verified
        assert f.read() == DATA
        assert f.raw.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_open_file_object(threads):
    with mibz2.open(io.BytesIO(COMPRESSED), threads=threads) as f:
        assert f.read() == DATA
        assert f.raw.verified


def test_lines(tmp_path):
    text = b"".join(b"line %d\n" % i for i in range(10_000))
    path = tmp_path / "lines.bz2"
    path.write_bytes(bz2.compress(text))
    with mibz2.open(os.fspath(path)) as f:
        assert f.readlines() == text.splitlines(keepends=True)


def test_not_bz2():
    with pytest.raises(mibz2.DecodeError):
        mibz2.decompress(b"not a bz2 stream at all")


@pytest.mark.parametrize("threads", [1, 4])
def test_corrupt_block_returns_none_of_its_bytes(threads):
    bad = bytearray(COMPRESSED)
    # a bit inside the second half of the input: a later block is damaged
    bad[len(bad) * 3 // 4] ^= 0x10
    got = bytearray()
    with mibz2.open(io.BytesIO(bytes(bad)), threads=threads) as f:
        with pytest.raises(mibz2.DecodeError):
            while chunk := f.read1(1 << 16):
                got += chunk
        assert not f.raw.verified
    # whatever was returned before the error is a prefix of the true output
    assert DATA.startswith(bytes(got))
    assert len(got) < len(DATA)


def test_truncated():
    with pytest.raises(mibz2.DecodeError):
        mibz2.decompress(COMPRESSED[:-10])
    with mibz2.open(io.BytesIO(COMPRESSED[:-10])) as f:
        with pytest.raises(mibz2.DecodeError):
            f.read()
        assert not f.raw.verified


def test_source_exception_propagates():
    class Broken(io.RawIOBase):
        def readable(self):
            return True

        def read(self, n=-1):
            raise RuntimeError("the source broke")

    with mibz2.open(Broken(), threads=1) as f:
        with pytest.raises(RuntimeError, match="the source broke"):
            f.read()


@pytest.mark.parametrize("threads", [1, 4])
def test_reader_zero_huge_and_closed(threads):
    from mibz2._mibz2 import Reader

    r = Reader(io.BytesIO(COMPRESSED), threads=threads)
    assert r.read(0) == b""
    assert not r.verified
    first = r.read(7)
    assert first == DATA[:7]
    assert r.read(0) == b""
    out = bytearray(first)
    # Huge size must cap at the lent decoded block rather than over-allocating
    while chunk := r.read(1 << 40):
        out += chunk
    assert bytes(out) == DATA
    assert r.verified
    assert r.read(0) == b""
    assert r.read(16) == b""
    r.close()
    with pytest.raises(ValueError, match="closed"):
        r.read(0)
    with pytest.raises(ValueError, match="closed"):
        r.read(16)


@pytest.mark.parametrize("threads", [1, 4])
def test_reader_nonblocking_and_empty(threads):
    from mibz2._mibz2 import Reader

    class NonBlocking:
        def __init__(self, payload: bytes) -> None:
            self.payload = payload
            self.pos = 0
            self.calls = 0

        def read(self, size: int) -> bytes | None:
            self.calls += 1
            if self.calls % 3 == 1 and self.pos < len(self.payload):
                return None
            chunk = self.payload[self.pos : self.pos + min(size, 4096)]
            self.pos += len(chunk)
            return chunk

    r = Reader(NonBlocking(COMPRESSED), threads=threads)
    out = bytearray()
    while chunk := r.read(65_536):
        out += chunk
    assert bytes(out) == DATA
    assert r.verified
    r.close()

    empty = Reader(io.BytesIO(bz2.compress(b"")), threads=threads)
    assert empty.read(0) == b""
    assert not empty.verified
    assert empty.read(1024) == b""
    assert empty.verified
    empty.close()


@pytest.mark.parametrize("threads", [1, 4])
def test_reader_corrupt_persists_on_zero_read(threads):
    from mibz2._mibz2 import Reader

    bad = bytearray(COMPRESSED)
    bad[len(bad) // 2] ^= 0xFF
    r = Reader(io.BytesIO(bytes(bad)), threads=threads)
    with pytest.raises(mibz2.DecodeError):
        while r.read(65_536):
            pass
    with pytest.raises(mibz2.DecodeError):
        r.read(0)
    r.close()
