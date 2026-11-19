# A reversible in-place MTF-to-maximal-run log

The AVX2 front already computes the true width of each maximal BWT run. Retain the temporal separation between MTF and the inverse constructor by writing that run to a contiguous log. The log fits inside the already-consumed u16 MTF symbols for every run prefix, including an opening run that has no literal. Its construction reads the original literal before overwriting its slot. The decoder's symbol bitmaps retain RUNA/B values and the positions of future literals.

## Representation and prefix contraction

A maximal run of byte c and positive width w≤254 takes one word c⟨8⟩+w, where c occupies the high byte. For 255≤w≤65535 use low-byte tag 0 and one u16 payload. For larger w use low-byte tag 255 and two payload words, least-significant first. The occupied space is at most ceil(w/127) for every w. This is a word-count statement independent of canonical run order.

A regular literal with t following run symbols and little-endian RUNB mask u has width 2ᵗ+u and consumes 1+t input words. A nonempty opening span has width 2ᵗ−1+u and consumes t input words. A regular span with t=0 uses one word. A nonempty opening has t≥1 and takes one word through width 254. To reach 255 at least seven run symbols are necessary; to exceed 65535 at least sixteen are necessary. Thus every local output width in machine words is at most its span of input symbols. Summing this inequality proves the invariant at every run boundary. Writing forward to the front of a mutable input slice cannot overwrite a later literal. This removes a separately allocated record buffer without coupling MTF to label-indexed row queues.

## Restoring an overwritten prefix

The map is injective on every valid completed run prefix when its ordered active alphabet is fixed and contains distinct bytes. If the first log byte is the initial MTF head, its span was the opening run. Otherwise the first span has a literal. For every regular span its literal's old rank is recovered by moving the stored byte to the front of the original evolving alphabet. Its original rank is positive, since two consecutive maximal run bytes differ. For a regular span take v=w; for the opening take v=w+1. The number of run symbols is floor(log₂ v), and their low-order RUNB bits are the low floor(log₂ v) bits of v.

Let T be the number of packed words and U the number of original symbols covered. A log scan recovers U from the widths and the opening flag. Move T words to the right end of this U-word prefix. Expand forward using only a 256-byte MTF list and fixed scalars. After any number of decoded records, the accumulated expansion is at most the final U−T, by local nonnegative expansion. The output cursor never passes the end of the just-read log record. Read the current record completely before writing its restored span. The original suffix, including any later malformed symbols or EOB, was never touched. Thus a late error or rejected admission may restore the exact original symbol buffer before invoking the stock worker and its normal error ordering. The rollback cost includes a log-size move, two log passes, and MTF rank lookup during expansion. It can be expensive on a rejected block.

The scalar oracle checked every input word and every interruption position through eight packed-source alphabet-4 symbols (844,686 prefix restores), 34 escape boundaries including 65535 and 65536, and 11 failing partial writes. All 431 real blocks matched stock maximal runs exactly and restored every input symbol and suffix. The 233 large blocks contained 24,970,771 MTF input words, 16,320,934 stock cap records and 15,258,121 packed words, including 116,519 extended u16 widths. None needed the long u32 form. This saves 6.51% of sequential log words on these blocks and makes a separate record buffer unnecessary given mutable ownership. [Scalar implementation](log.rs) · [Exhaustive and corruption checks](small.txt) · [All real blocks](real.csv) · [Audit source](audit.rs)

The existing worker hook borrows immutable Symbols. Any actual implementation requires a genuinely mutable ownership entry to reuse the buffer. The scalar oracle receives a mutable slice and uses safe overlapping copy for rollback. The paid native follow-up must compare the two full phases and include in-place ownership, bitmap construction, both admission alternatives, fallback restoration, RLE1 and CRC in addition to separately measuring the sequential log conversion. Real output speed and i9 RSS remain open.

```sh
d=research/astra-e/front-fusion/packed
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc -O --edition 2021 "$d/audit.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/audit"
"$scratch/audit" small
"$scratch/audit" corpora/wikidata.bz2
```
