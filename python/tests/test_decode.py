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


@pytest.mark.parametrize("threads", [1, 4])
def test_bz2reader_readall_and_read_modes(threads):
    # readall(), read(-1), and read(None) all return full data and set verified=True
    for call in (
        lambda r: r.readall(),
        lambda r: r.read(-1),
        lambda r: r.read(None),
        lambda r: r.read(),
    ):
        with mibz2.BZ2Reader(io.BytesIO(COMPRESSED), threads=threads) as r:
            assert not r.verified
            assert call(r) == DATA
            assert r.verified
            assert r.readall() == b""
            assert r.read(-1) == b""
            assert r.read(0) == b""
            assert r.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_bz2reader_partial_read_then_readall(threads):
    # Direct BZ2Reader partial read followed by readall()
    with mibz2.BZ2Reader(io.BytesIO(COMPRESSED), threads=threads) as r:
        assert r.read(0) == b""
        assert not r.verified
        head = r.read(100)
        assert head == DATA[:100]
        assert not r.verified
        rest = r.readall()
        assert head + rest == DATA
        assert r.verified

    # Buffered mibz2.open() partial read followed by read()
    with mibz2.open(io.BytesIO(COMPRESSED), threads=threads) as f:
        head = f.read(100)
        assert head == DATA[:100]
        rest = f.read()
        assert head + rest == DATA
        assert f.raw.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_bz2reader_readall_empty_single_block_and_multistream(threads):
    # Empty bzip2 stream
    with mibz2.BZ2Reader(io.BytesIO(bz2.compress(b"")), threads=threads) as r:
        assert not r.verified
        assert r.readall() == b""
        assert r.verified

    # Single small block (exercises len(chunks) == 1 zero-join fast path)
    small = b"hello world" * 100
    with mibz2.BZ2Reader(io.BytesIO(bz2.compress(small)), threads=threads) as r:
        assert r.readall() == small
        assert r.verified

    # Multi-stream concatenation
    two = COMPRESSED + bz2.compress(b"second-stream-tail")
    with mibz2.BZ2Reader(io.BytesIO(two), threads=threads) as r:
        assert r.readall() == DATA + b"second-stream-tail"
        assert r.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_bz2reader_readall_closed_and_corrupt(threads):
    r = mibz2.BZ2Reader(io.BytesIO(COMPRESSED), threads=threads)
    r.close()
    with pytest.raises(ValueError, match="closed"):
        r.readall()
    with pytest.raises(ValueError, match="closed"):
        r.read(-1)
    with pytest.raises(ValueError, match="closed"):
        r.read(None)
    with pytest.raises(ValueError, match="closed"):
        r.read(0)
    with pytest.raises(ValueError, match="closed"):
        r.read(16)

    # Corrupt block CRC propagates DecodeError and leaves verified=False
    bad = bytearray(COMPRESSED)
    bad[len(bad) * 3 // 4] ^= 0x10
    with mibz2.BZ2Reader(io.BytesIO(bytes(bad)), threads=threads) as r:
        with pytest.raises(mibz2.DecodeError):
            r.readall()
        assert not r.verified

    # Truncated stream propagates DecodeError and leaves verified=False
    with mibz2.BZ2Reader(io.BytesIO(COMPRESSED[:-10]), threads=threads) as r:
        with pytest.raises(mibz2.DecodeError):
            r.readall()
        assert not r.verified


# Multi-block line corpus (100 kB blocks, no trailing newline)
_LINE_RAW = b"alpha\nbeta\ngamma" * 70_000
_LINE_COMPRESSED = bz2.compress(_LINE_RAW, compresslevel=1)


@pytest.mark.parametrize("threads", [1, 4])
def test_readline_limits_cross_block_and_eof(threads):
    # Test both mibz2.open() and raw BZ2Reader with size=0, small limit, exact limit,
    # large limit, negative/None, no-trailing-newline tail, and EOF persistence.
    for opener in (
        lambda: mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads),
        lambda: mibz2.BZ2Reader(io.BytesIO(_LINE_COMPRESSED), threads=threads),
    ):
        with opener() as f:
            raw_obj = f.raw if isinstance(f, io.BufferedReader) else f
            assert not raw_obj.verified
            assert f.readline(0) == b""
            assert f.readline(3) == b"alp"
            assert f.readline(3) == b"ha\n"
            assert f.readline(5) == b"beta\n"
            assert f.readline(100) == b"gammaalpha\n"
            assert f.readline(-1) == b"beta\n"
            assert f.readline(None) == b"gammaalpha\n"
            out = bytearray(b"alpha\nbeta\ngammaalpha\nbeta\ngammaalpha\n")
            while line := f.readline():
                out += line
            assert bytes(out) == _LINE_RAW
            assert raw_obj.verified
            assert f.readline() == b""
            assert f.readline(10) == b""
            assert f.readline(0) == b""


@pytest.mark.parametrize("threads", [1, 4])
def test_readline_very_long_cross_block_line(threads):
    # A single line spanning >2 compressed blocks (250 kB with no newline until the end)
    long_line = (b"x" * 250_000) + b"\n" + b"tail-no-nl"
    comp = bz2.compress(long_line, compresslevel=1)
    with mibz2.open(io.BytesIO(comp), threads=threads) as f:
        assert f.readline(150_000) == b"x" * 150_000
        assert f.readline() == (b"x" * 100_000) + b"\n"
        assert f.readline() == b"tail-no-nl"
        assert f.readline() == b""
        assert f.raw.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_readlines_and_hint(threads):
    expected = _LINE_RAW.splitlines(keepends=True)
    for opener in (
        lambda: mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads),
        lambda: mibz2.BZ2Reader(io.BytesIO(_LINE_COMPRESSED), threads=threads),
    ):
        with opener() as f:
            raw_obj = f.raw if isinstance(f, io.BufferedReader) else f
            part = f.readlines(1234)
            assert sum(len(x) for x in part) >= 1234
            assert len(part) < len(expected)
            assert not raw_obj.verified
            rest = f.readlines(-1)
            assert part + rest == expected
            assert raw_obj.verified
            assert f.readlines() == []
            assert f.readlines(0) == []

    # Empty stream readlines
    with mibz2.open(io.BytesIO(bz2.compress(b"")), threads=threads) as f:
        assert f.readlines() == []
        assert f.raw.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_iter_and_next_raw_and_buffered(threads):
    expected = _LINE_RAW.splitlines(keepends=True)
    for opener in (
        lambda: mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads),
        lambda: mibz2.BZ2Reader(io.BytesIO(_LINE_COMPRESSED), threads=threads),
    ):
        with opener() as f:
            raw_obj = f.raw if isinstance(f, io.BufferedReader) else f
            first = next(f)
            assert first == expected[0]
            rest = list(f)
            assert [first, *rest] == expected
            assert raw_obj.verified
            with pytest.raises(StopIteration):
                next(f)


@pytest.mark.parametrize("threads", [1, 4])
def test_buffered_reader_contract_peek_readinto_detach_and_text(threads):
    with pytest.raises(ValueError, match="positive"):
        mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads, buffer_size=0)

    # isinstance(io.BufferedReader), peek non-advancing, read(size) cross-block, read1
    with mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads) as f:
        assert isinstance(f, io.BufferedReader)
        assert isinstance(f.raw, mibz2.BZ2Reader)
        p1 = f.peek(10)
        p2 = f.peek(10)
        assert p1 == p2 and p1.startswith(b"alpha\n")
        assert f.read(0) == b""
        # Cross-block bounded read (250 kB > 100 kB block)
        head = f.read(250_000)
        assert len(head) == 250_000
        assert head == _LINE_RAW[:250_000]
        # read1 returns up to one block
        chunk1 = f.read1(10_000)
        assert 0 < len(chunk1) <= 10_000
        rest = f.read()
        assert head + chunk1 + rest == _LINE_RAW
        assert f.raw.verified
        assert f.peek(10) == b""

    # readinto / readinto1 with bytearray, memoryview, non-byte cast view, and 0-length
    with mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads) as f:
        assert f.readinto(bytearray(0)) == 0
        assert f.readinto1(bytearray(0)) == 0
        buf16 = memoryview(bytearray(200)).cast("H")
        n_cast = f.readinto1(buf16)
        assert 0 < n_cast <= 200
        out = bytearray(buf16.cast("B")[:n_cast])
        buf_big = bytearray(250_000)
        while n := f.readinto(buf_big):
            out += buf_big[:n]
        assert bytes(out) == _LINE_RAW
        assert f.raw.verified

    # detach() returns BZ2Reader cleanly and invalidates detached wrapper
    f_det = mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads)
    first = f_det.readline()
    assert first == b"alpha\n"
    raw = f_det.detach()
    assert isinstance(raw, mibz2.BZ2Reader)
    for op in (
        lambda x: x.read(10),
        lambda x: x.read1(10),
        lambda x: x.readinto(bytearray(10)),
        lambda x: x.readinto1(bytearray(10)),
        lambda x: x.readline(),
        lambda x: x.readlines(),
        lambda x: x.peek(),
        lambda x: list(x),
        lambda x: next(x),
    ):
        with pytest.raises(ValueError, match="detached"):
            op(f_det)
    rest_lines = list(raw)
    assert first + b"".join(rest_lines) == _LINE_RAW
    assert raw.verified
    raw.close()

    # io.TextIOWrapper interoperability
    with mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads) as f:
        text_out = io.TextIOWrapper(f, encoding="ascii").read()
        assert text_out == _LINE_RAW.decode("ascii")
        assert f.raw.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_line_and_buffered_closed_and_corrupt_errors(threads):
    f = mibz2.open(io.BytesIO(_LINE_COMPRESSED), threads=threads)
    f.close()
    for op in (
        lambda x: x.read(10),
        lambda x: x.read1(10),
        lambda x: x.readinto(bytearray(10)),
        lambda x: x.readinto1(bytearray(10)),
        lambda x: x.readline(),
        lambda x: x.readlines(),
        lambda x: x.peek(),
        lambda x: list(x),
        lambda x: next(x),
    ):
        with pytest.raises(ValueError, match="closed"):
            op(f)

    r = mibz2.BZ2Reader(io.BytesIO(_LINE_COMPRESSED), threads=threads)
    r.close()
    for op in (
        lambda x: x.readline(),
        lambda x: x.readlines(),
        lambda x: x.readinto(bytearray(10)),
        lambda x: list(x),
        lambda x: next(x),
    ):
        with pytest.raises(ValueError, match="closed"):
            op(r)

    bad = bytearray(_LINE_COMPRESSED)
    bad[len(bad) * 3 // 4] ^= 0x10
    for mode in ("iter", "readline", "readlines", "read_full"):
        with mibz2.open(io.BytesIO(bytes(bad)), threads=threads) as bf:
            with pytest.raises(mibz2.DecodeError):
                if mode == "iter":
                    for _ in bf:
                        pass
                elif mode == "readline":
                    while bf.readline():
                        pass
                elif mode == "readlines":
                    bf.readlines()
                else:
                    while bf.read(250_000):
                        pass
            assert not bf.raw.verified


@pytest.mark.parametrize("threads", [1, 4])
def test_reader_nonblocking_lines_and_read_full(threads):
    class NonBlocking(io.RawIOBase):
        def __init__(self, payload: bytes) -> None:
            super().__init__()
            self.payload = payload
            self.pos = 0
            self.calls = 0

        def readable(self) -> bool:
            return True

        def read(self, size: int = -1) -> bytes | None:
            self.calls += 1
            if self.calls % 3 == 1 and self.pos < len(self.payload):
                return None
            cap = 4096 if size < 0 else min(size, 4096)
            chunk = self.payload[self.pos : self.pos + cap]
            self.pos += len(chunk)
            return chunk

    with mibz2.open(NonBlocking(_LINE_COMPRESSED), threads=threads) as f:
        assert f.readlines() == _LINE_RAW.splitlines(keepends=True)
        assert f.raw.verified

    with mibz2.open(NonBlocking(_LINE_COMPRESSED), threads=threads) as f:
        out = bytearray()
        while chunk := f.read(250_000):
            out += chunk
        assert bytes(out) == _LINE_RAW
        assert f.raw.verified

