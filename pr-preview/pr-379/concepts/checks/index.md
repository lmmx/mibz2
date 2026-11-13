# What is checked, and when

## Block CRCs, before the block's bytes

Every block's CRC is checked before any of that block's bytes are returned. A block whose CRC does not match returns none of its bytes: the error comes instead. The bytes returned before an error are exactly the stream's up to the bad block.

## Stream CRCs, at the stream's end

Each stream's combined CRC is checked at the stream's end. The bytes of a stream's blocks are returned as each block is verified, before the stream's own CRC has been read. So a streaming caller has the whole stream verified only once a read reaches its end without an error: until then, what it has read is provisional.

In Python, `BZ2Reader.verified` turns true only when a read returns the end with every block's and stream's CRC matched.

## Where an error was found

An error says where decoding failed: in Rust `Error { kind, block, bit }`, in Python the `DecodeError` message. That is which block, and the bit it starts at. A damaged bit can be found later than where it lies, since the input is only known to be wrong once something it encodes fails a check.

The error kinds (`ErrorKind`):

- the input: `Empty`, `NotBz2`, `Truncated`, `TrailingData`;
- each of libbzip2's block checks: `BadBlockMagic`, `BadOrigPtr`, `NoSymbols`, `BadGroups`, `BadSelectors`, `BadSelector`, `BadCodeLength`, `BadHuffmanCode`, `SelectorOverrun`, `RunTooLong`, `BlockTooLong`, `RunUnfinished`;
- `BlockCrc` and `StreamCrc`, with the stored and computed values;
- this decoder's refusals: `Randomised`, `BlockTooLarge`;
- `Io` (from a `BzReader`'s source), `RangeOvershoot`, `Cancelled`, and `Internal` (a caught panic: a bug).

## What is decoded, and what is refused

The format is libbzip2 1.0.8's, with every check it makes: bzip2 1.0 and later output. Four choices differ from it, and each has its own error:

| input | libbzip2 | mibz2 |
| --- | --- | --- |
| randomised blocks (written only by bzip2 0.9.0 and earlier, before 1999) | decoded | refused (`Randomised`) |
| bytes after the last stream that do not begin another stream | ignored | an error (`TrailingData`): nothing after a stream is silently dropped |
| an oversubscribed Huffman code pointing past the alphabet | a stale table entry read | an error |
| a block whose compressed form passes 64 MiB (a real block is at most about 4.2 MB) | decoded | an error (`BlockTooLarge`) |
