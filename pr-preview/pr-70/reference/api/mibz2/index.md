# mibz2

mibz2: a fast bzip2 decoder, block-parallel and multi-chain within each block.

- `decompress`: a whole .bz2 in memory, decoded.
- `open`: a .bz2 file (a path or a binary file object) read as a buffered binary file, decoded as it is read.

Every block's CRC is checked before any of that block's bytes are returned, and each stream's combined CRC is checked at the stream's end. Bytes read from a stream before its end come from blocks whose own CRCs matched, but the stream as a whole is verified only when a read reaches its end: until then, treat what has been read as provisional (see `BZ2Reader.verified`).

## DecodeError

Bases: `ValueError`

The input is not valid bzip2 data.

## decompress

```python
decompress(data: bytes | bytearray, threads: int | None = None) -> bytes
```

Decode a whole .bz2 (its streams one after another), every CRC checked.

It runs on every CPU this process may use, or on `threads`.

Raises:

| Type | Description |
| --- | --- |
| `DecodeError` | When the input is not valid bzip2 data. |

## BZ2Reader

```python
BZ2Reader(source: Source, threads: int | None = None)
```

Bases: `RawIOBase`

A .bz2 file's decoded bytes, read in order (an unbuffered binary file).

Parameters:

| Name | Type | Description | Default |
| --- | --- | --- | --- |
| `source` | `Source` | A path, or a binary file object open for reading (its `read` is called). | *required* |
| `threads` | `int \| None` | Decoding threads: `1` decodes on the reading thread; more decode that many blocks at once on worker threads. `None` (the default) uses every CPU this process may use. | `None` |

Raises:

| Type | Description |
| --- | --- |
| `DecodeError` | From a read, when the input is not valid bzip2 data: a block whose CRC does not match (none of its bytes are returned), a stream whose combined CRC does not match (raised by the read that reaches the stream's end), or a malformed or truncated stream. The error says which block and bit it was found at, which is where decoding failed and need not be where the input was damaged. |

### readable

```python
readable() -> bool
```

True: a reader is readable.

### readinto

```python
readinto(b) -> int
```

Read up to `len(b)` decoded bytes into `b`; 0 at the end (when verified).

### verified

```python
verified: bool
```

True once a read has reached the end and every block's and stream's CRC matched.

Until then, the bytes read so far have each passed their block's CRC, but the stream's combined CRC has not yet been checked.

### close

```python
close() -> None
```

Stop decoding (its worker threads stop) and close the reader.

## open

```python
open(
    source: Source, threads: int | None = None, buffer_size: int = 1 << 20
) -> BufferedReader[BZ2Reader]
```

Open a .bz2 file for reading, decoded as it is read.

Parameters:

| Name | Type | Description | Default |
| --- | --- | --- | --- |
| `source` | `Source` | A path, or a binary file object open for reading. | *required* |
| `threads` | `int \| None` | Decoding threads, as for `BZ2Reader`. | `None` |
| `buffer_size` | `int` | The buffered reader's buffer, in bytes. | `1 << 20` |

Returns:

| Type | Description |
| --- | --- |
| `BufferedReader` | A buffered binary file over a `BZ2Reader` (its `raw`), whose `raw.verified` says whether the whole input has been read and verified. |

Examples:

```pycon
>>> import bz2, mibz2, tempfile, os
>>> path = os.path.join(tempfile.mkdtemp(), "a.bz2")
>>> with bz2.open(path, "wb") as f:
...     _ = f.write(b"line one\nline two\n")
>>> with mibz2.open(path) as f:
...     lines = f.readlines()
...     verified = f.raw.verified
>>> lines, verified
([b'line one\n', b'line two\n'], True)
```
