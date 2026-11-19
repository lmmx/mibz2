# Reproducing the mixed-step study

The [journal](../2026-11-18-cells-mixed-steps.md) gives the two-cut identity, the surviving-site invariant and the cost model. The programs here use Python 3.10+ and the repository's `bench/irec.py`. Run with assertions enabled.

## Published traces and small examples

From the root of this checkout, extract the pinned data-only branch. Its Wikidata ordinals count blocks of at least 65,536 rows. The record producer at the time was `2da2a24944e16700475c572166695cc47f60c233`. [Data README](https://github.com/lmmx/mibz2/blob/2e4b34e086ac95e8218f1ee822a4652c8db5bb19/README.md)

```sh
study="$PWD/docs/journal/2026-11-18-cells-mixed-steps"
tmp=$(mktemp -d)
git fetch origin refs/heads/data/induction-records
git archive 2e4b34e086ac95e8218f1ee822a4652c8db5bb19 wikidata prose | tar -x -C "$tmp"
python3 "$study/verify.py" --small
python3 "$study/geometry.py" --small
python3 "$study/verify.py" "$tmp/wikidata" "$tmp/prose" > "$tmp/sample-audit.jsonl"
python3 "$study/geometry.py" "$tmp/wikidata/0000.irec.xz"
python3 "$study/summary.py"
```

The small cycle check tries every choice of marked row on all binary columns of lengths 1–9 and all ternary columns of lengths 1–6. The two alphabets are separate suites. The independent oracle walks the inverse stable-sort permutation from the mark. Over 14,209 cases it finds 1,630 direct TE substitutions, 413 TB and 746 shared products. The local-order check tries all bottom permutations on three through six sites with distinct $A,B,D$, widths $b,d$ from 1 through 3 and all $1\le s\le d$. All 10,188 final order/width triples equal those from two scalar cuts.

Four primitive columns expose the hypotheses. Indices and steps are zero-based. All witnesses occur at step 0.

| Column $L$ | Mark $p$ | What goes wrong with a stronger claim |
|---|---:|---|
| `100` | 0 | The old bottom predecessor is $A$ itself. T inserts $X$ after $A$. The actual next opponent is $X$ with width 1. Reading old $A$ with width 2 predicts the wrong second event. |
| `11100` | 1 | The two bottom letters both spell `1`, but only the second is marked. Comparing plain bytes would erase the distinguished occurrence. |
| `11200` | 0 | Distinct unmarked bottom literals `2` and `1` meet the width test. The boundary substitution is valid and both words must be made. |
| `101000` | 3 | Both products spell `01`. The new cells occupy original top sites 0 and 2, with the descendant of site 1 between them. |

For the frozen trace the first TE hit is step 1 with $A,B,D=(51183,51090,51088)$. The first TB hit is step 84 with $(51103,6556,6553)$. The order checker reports each example as `[step,A,B,D,a,b,d,byte(D),product_length]`. SHA-256 of the sixteen-line `sample-audit.jsonl` is `7c4701a87139c31143fab52eb60264c6fc5e995ff4b9d7cda8aaff5022e5ad66`.

## From the compressed Wikidata stream

The corpus SHA-256 is `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. The repository's `bench/corpus.sh` reproduces it from the pinned Wikidata windows. The six prose records use corpus SHA-256 `5dcc4817845a94d62556716f9a8305ee95bd3776653f0a48a20247cfb13d6357`. The published branch includes that smaller compressed file. Run the existing recorder from the study's code baseline. The local run used rustc 1.90.0 and Python 3.12.14. The repository pins rustc 1.99.1 for its performance measurements. Its own checks replay the kernel's hooks and compare the marked cycle against the kernel's output before writing each file. The 233 uncompressed records occupy about 1.7 GiB. [Corpus construction](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/bench/corpus.sh) · [Recorder](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/examples/induction_trace.rs)

```sh
wiki="$PWD/corpora/wikidata.bz2"
code=642be1bc3efc7c702773e85e8a3a696b97a1821d
git worktree add --detach "$tmp/kernel" "$code"
cargo build --manifest-path "$tmp/kernel/Cargo.toml" --target-dir "$tmp/target" \
  --release --locked --features induction-trace --example induction_trace
"$tmp/target/release/examples/induction_trace" "$wiki" "$tmp/all" > "$tmp/recorder.jsonl"
python3 "$study/census.py" "$tmp/all" "$tmp/prose" > "$tmp/blocks.csv"
cmp "$tmp/blocks.csv" "$study/blocks.csv"
python3 "$study/verify.py" "$tmp/all" > "$tmp/all-audit.jsonl"
python3 "$study/geometry.py" "$tmp/all" > "$tmp/all-geometry.jsonl"
sha256sum "$tmp/all-audit.jsonl" "$tmp/all-geometry.jsonl"
```

Expected SHA-256 values in the last command are `b5d7e47937a466d8fd11facf2ceb1fa70faf1630dbecf1505d6efa4af1d45c01` and `5f8157a38f6827b93dcbdbc8c08f066d5330850ec108c19342bbf9a76d73c757`, respectively. The order census returns 650,507 separated and zero gluable pairs. All arrays in each of the ten published Wikidata records equal those from regeneration. At ordinal 144 the older header has `admitted=false` and the study revision has `admitted=true`. The other nine headers agree. The census CSV uses the study revision's flag for the 233 Wikidata rows and the published flags for prose. A forced trace describes the induction on a block even when the production admission gate would choose another path.

## What each program checks

- [census.py](census.py) is the offline count. It reads recorded steps, freshness, word kinds and uses. Pair counts exclude closures. A mixed immediate child is the first cell appearing as an operand of the second. A single-use child has exactly one recorded use in the full grammar.
- [verify.py](verify.py) is the constructive test. `Machine.run` needs only the initial per-site lengths and bytes, stable bottom order and marked site. Its scalar and joint modes use the shipped convention of paired front slots, a shared entry tag, two reverse cursors and a free-pair pool. For readability it uses original names for stream headers. The shipped top cursor locates the same entry through the byte's existing end cursor. An extra letter ID in each test slot supports independent assertions. Neither ID equality nor future trace events recognize a pair.
- The two runs compare complete cycle bytes and mark position, all width decisions, actual number of word constructions, short copies, arena advances in bytes, rule count, both orders' route/header/slot visits, pool takes/gives, peak slots, final cursors, free list and tags. The joint run also checks every logical event and closure against the external trace. It descends the independent trace grammar to locate the mark and expands the two final words separately. These expansions and comparisons are test costs.
- [geometry.py](geometry.py) tests the closed formula on permutations and searches for primitive counterexamples using the two literal orders. On recordings it uses the repository's separate `irec.Orders` implementation to expose old neighbors before the cut. The test reads the top successor of $D$ to determine whether the two new bands could be adjacent, independently of the lazy-slot transducer.
- [summary.py](summary.py) reduces the [per-block CSV](blocks.csv) to the journal table and the priced bound. Splits by admission, the original ten-block sample and the frozen observation are overlapping subsets of the 233.

## Direct slot update and recognition charges

In the source loop let $f$ be $A$'s top front slot, $k$ its promised bottom site, $g$ the bottom front slot of $B$, and $t$ the promised top site of $B$. Let $h$ be the previous link at $k$. The mandatory settlement of fresh $D$ obtains a top/bottom slot pair $(u,j)$ and tags its source entry with $u$. Its bottom predecessor links back to the stream. The following final assignments suffice. Here a top payload holds width, word descriptor and its opposite slot, and a bottom record holds previous link and opposite slot.

```text
top[t].payload = (b, wordX, k)
top[u].payload = (s, wordY, g)
bottom[k] = (g, t)
bottom[g] = (h, u)

s = d:  top[u].previous = 0. Give back the pair (f,j).
        Resume the whole-front reads at the old A predecessor and the bottom stream.

s < d:  top[f].previous = 0.  top[u].previous = f.
        Keep D at its bottom front j, with remaining width d-s and promised top site f.
        Read the next top front at the old A predecessor, after the assignments.
```

The top previous link at $t$ is retained. The two fields of `bottom[g]` may be written separately, as in the scalar path. The ledger takes no credit for packing them. The first T in the scalar loop sets `bs = g` and reduces $A$'s width in registers. The direct form has no reason to enter a loop with those intermediate fronts.

The priced control flow has three separate successors for a width comparison. Existing bounds checks, read modes and allocations follow the same sources. The static continuations in this sketch make the placement of the comparisons explicit.

```text
known T on A,B:
    X = ordinary join(A.word, B.word)
    s = a-b
    dispatch the mandatory next-bottom source using B.previous
      linked slot or tagged entry:
          apply ordinary T, then load that slot's current payload
          enter ordinary width decision on A's remainder and the next bottom front
      untagged entry D:
          read the literal width d and its packed word once
          use the next logical cell's width comparison, compare(s,d)
            greater: settle D, finish ordinary T, enter known T on the remainder and D
            equal:   settle D, apply the joint TE assignments
            less:    settle D with its top previous link deferred, apply the joint TB assignments

on either mixed branch:
    Y.word = X.word if B.word == D.word and X is an unmarked short word
    otherwise Y.word = ordinary join(A.word, D.word)
    install the two final descriptors at their own sites
```

The last three lines supply the words used by the assignments. A budgeted version uses the ordinary second check/debit when its decision moves forward and goes directly to the already-debited arm. If only one cell is available it finishes the ordinary first cut and takes the usual refusal. Exactly one debit is required for each logical cell.

Moving the dispatch past or before a group of scalar writes changes live ranges. On a linked/tagged fallback there is no need to keep a second front's payload alive. On an untagged entry the additional live data are bounded by one entry/front and scratch registers. The unit model prices one further control transfer for every such entry, both recipe predicates for every mixed candidate (including long words and mismatches), and a descriptor selection for every mixed candidate. The full false-probe population is included in these charges. Compiler register allocation, instruction-cache effects and multi-worker timings are outside the measurements.
