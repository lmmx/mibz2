# Reproduce the ownership-schedule study

The [research record](../2026-11-18-cells-owner-schedules.md) contains the proofs and conclusion. The scripts below consume induction records and study the *finished* W16 grammar. They audit spans against supplied bytes. No native decoder, benchmark, admission gate or performance branch is involved.

## Inputs and checks

The ten archived Wikidata inputs are large-block ordinals 0, 24, …, 216. Ordinal 0 is the frozen 899,931-row block with primary 264,045. Use Python 3 with assertions enabled. From the checkout root run the following commands.

```sh
study="$PWD/docs/journal/2026-11-18-cells-owner-schedules"
tmp=$(mktemp -d)
git fetch origin refs/heads/data/induction-records
git archive 2e4b34e086ac95e8218f1ee822a4652c8db5bb19 wikidata | tar -x -C "$tmp"
python3 "$study/small.py"
python3 "$study/schedules.py" --check "$tmp/wikidata" > "$tmp/wikidata.csv"
python3 "$study/first_visit.py" --check "$tmp/wikidata" > "$tmp/first-visit.csv"
cmp "$study/wikidata.csv" "$tmp/wikidata.csv"
cmp "$study/first-visit.csv" "$tmp/first-visit.csv"
python3 "$study/summary.py"
```

Both trace scripts reuse Study 5's ordinary grammar transcription, marked rotation and independent initialized-output oracle. Each archived trace also checks the cyclic spelling against dense stable-LF inversion from the recorded column. A checksum alone would miss uninitialized reads of equal bytes. The coverage oracle checks every source before the copy, disjointness, exactly one write to each destination and the marked singleton's position. It runs in birth order, in the offline wave-order control and during the immediate first-occurrence walk. The chosen rule edges and positions are computed without the oracle. No result depends on a decoded spelling supplied to an algorithm. [Trace schema](https://github.com/lmmx/mibz2/blob/2e4b34e086ac95e8218f1ee822a4652c8db5bb19/README.md) · [Study 5 transcriber and byte oracle](../2026-11-18-cells-live-reuse/positions.py)

The whole-corpus count uses Study 5's 233 unmodified recorder traces of the Wikidata compressed input with SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. With the all-block directory produced by the [Study 5 reproduction](../2026-11-18-cells-live-reuse/README.md), run the two commands below. A stored child forces its parent to be stored too. Counting its left and right operand identifiers in the induction record therefore recovers exactly the stored grammar's left and right rule-edge counts. The 233-row census needs no output positions or traversal. Other measurements have ten-block coverage. These populations include forced grammars, independently of the production gate. Both writers being compared receive the same grammar. All the measured periods equal their block lengths.

```sh
python3 "$study/summary.py" --grammar-census "$tmp/all" > "$tmp/counts.csv"
cmp "$study/counts.csv" "$tmp/counts.csv"
```

| Evidence | SHA-256 |
|---|---|
| [counts.csv](counts.csv), whole-corpus algebraic count | `b5588809f5dcf17cff6a82726cdc1f4399de4ed812244ce2c13606af1ea683c0` |
| [wikidata.csv](wikidata.csv), five tree choices | `e604a0160bc3e5483a82c9114928aa72dd68cdcdf805a68dd303c7314fc90414` |
| [first-visit.csv](first-visit.csv), paired traversal evidence | `2f15ae3dc7f4ea20879bdf85524f77f908074c905d14bec8bafcf49842a724b1` |

[small.py](small.py) tests every binary column through length nine at every primary, 8,194 cases. For each of the 976 full-period cases it enumerates *every* tree. This gives 1,207 trees at the rule-heavy cutoff one and 976 at cutoff 16. It independently derives both critical paths from physical overlaps between source spans and destination holes. The 52-, 71- and 88-row witnesses use the real cutoff with three, four and six choices respectively. The explicit left-comb family also checks worst-case stack growth. All use the scalar two-order induction and dense LF. The scope throughout is a single reachable period. The normal periodic repetition, RLE1 and CRC operations are common downstream work.

## What an ownership policy changes

[schedules.py](schedules.py) compares last parent by birth (`last`), first parent by birth (`first`), exact minimum atomic copy rounds (`rounds`), exact minimum byte-weighted atomic span (`bytes`), and a conditional minimum-distance owner (`median`). The first parent by birth differs from the first occurrence seen in text order. All birth orders here use increasing parent index in the copy phase. The two depth policies break equal priorities towards last-use. For the median, a two-offer tie keeps last-use. With more offers the first in reverse birth order within the middle interval wins. For fixed parent positions and two offers, swapping source and destination cannot change their absolute distance. Changing an ancestor can move either offer.

The `changed` column counts rules with a different owner from last-use. `early` counts commands generated before their owner's coordinate arrives in the reverse scan. `double` counts parents that defer both children. The study checks the in-place suffix capacity for every selection. The contiguous-use proof gives the same one-comparison certificate for birth-first on the previous neighboring record as for last-use on the next neighbor. Birth-first has at most one defer per parent in Cells and can carry/pipeline the operand pair. For optimal depth a realizable offline design stores a height and a chosen parent edge for each rule and processes the $E$ offers. A streaming version could collect the two largest scores in a current same-child run. It must still propagate depths and resolve ties. Median selection needs the actual offered coordinates. On the ten blocks, 7,740 children have at least three offers, with 108,132 such offers in total. Frozen has a child with 618. One can buffer only the active runs, but the coordinates and their order statistic must still be obtained. The large dictionaries and arrays used for these offline controls have no credited zero-cost native equivalent.

**An upper bound that includes every ownership tree.** Unfold all stored-rule occurrences in the root with their cyclic positions. For an incoming edge $e$ to child $v$, let $B_e$ contain every address bin in which this offer can start over all occurrences of the parent. In any coherent tree the source starts in some bin $b$. At most $|\{e:b\in B_e\}|-1$ other offers can start in that bin. Maximize over bins and sum over children. This deliberately grants each edge an independent choice of parent occurrence. The result is an upper bound for *every* coherent tree, including arrangements absent from the five test policies. At 64-byte bins the frozen maximum allowed by the relaxation is one, at 4 KiB it is 816, against zero and 462 achieved by last-use. Across ten blocks the bounds are 13 and 9,240, against eight and 5,363. A shared start bin is a spatial statistic, distinct from a cache hit. The census unfolded 100,933 stored-rule occurrences on frozen to compute these bounds.

The `aligned` field counts unordered potential joins where one copy's source and destination both end at the other copy's starts. Actual joining also needs readiness, compatible order and an admissible combined span. `adjacent` counts potential joins between neighbors of the birth-ordered stream in either address direction. Last-use has 14,784 unordered pairs across ten blocks and only 42 neighboring pairs. As a control, order commands by earliest executable round and then destination. Readiness follows from child completion heights computed in ascending birth order. This gives 7,163 neighboring forward pairs and 410,736 output-only fills with last-use, compared with 404,176 fills for the depth-minimum tree under the same ordering. The control uses a whole height pass and a positional sort of 106,902 commands. Its raw counts carry no credit for avoiding either cost. None of the comparisons assumes that one logical copy always requires one library call.

## The first-occurrence control and memory models

[first_visit.py](first_visit.py) tags the consumed first operand with the selected physical offset. A tag read on entry to every rule occurrence includes all unsuccessful reuse tests. The walk makes one 64-bit tag store on each first visit, including singleton-use and marked rules. A tag hit needs the length from the incoming descriptor. The ordinary mark walk happens before any tag is written. Bytes and addresses fit the shipped 32-bit fields. Grammar restoration costs zero under Study 5's disposable-grammar contract. Keeping the grammar for another consumer would require paying for separate writable state. The script keeps an immutable original and records selected edge IDs only to audit against the alternative schedules. [W16 operand and writer layout](https://github.com/lmmx/mibz2/blob/50612a8f7b141e4999645669cd67dce95c2dc0ad/src/cells.rs)

Let $P$ be the count of long left operands, $H$ the maximum number of simultaneously pending right operands and $T=R+1$ the number of emitted factors. A nonterminal left needs a push of its right sibling, even when the left turns out to be a memo hit. The ledger charges those failed opportunities. A short left is handled inline. Both first and repeated rules follow ordinary child references. A tag needs neither a hash probe nor a per-rule slot to be cleared. The $8H$ bytes of live pending descriptors can be charged as a cold doubling stack. The field totals in the main table are steady traversal totals. Starting from zero, final capacity $C$ is the smallest power of two at least $H$ for $H>0$. Charge $2(C-1)$ additional field accesses for relocation and $1+\log_2 C$ allocation calls per nonempty block, with a capacity test on every push. All ten traces stop at capacity 32. This is 620 relocation fields and 60 allocation calls across the sample. The model gives the baseline its existing position storage and command slots at zero setup cost.

The following explicit source-level price compares scalar bookkeeping. An addressed read or write includes indexed addressing. One predicate includes its conditional transfer. A declared add, mask, shift, move or cursor advance is one unit. Word extraction and the actual byte-copy operations common at the same factor counts and lengths cancel. Register assignment to the two carried neighbor operands is free for the baseline. The marked-position walk, common block admission and physical output capacity are excluded on both sides.

| Scalar operations | Last-use | First occurrence |
|---|---:|---:|
| Identify child types | $2R$ | $2R+1$ including root |
| Decide owner or memo hit | $E=R-1+K$ | $R+K$ tag tests |
| Pack tag or recover source from high half | 0 | $R+K$ |
| Form destinations | $3R+W$ for left-length mask, add, range test and $W$ actual wraps | $2T+1$ for cursor add, end test and one reset |
| Reverse scan advance and loop test | $2R$ | 0 |
| Command cursors and forward loop test | $3K$ | 0 |
| Push/pop cursor, capacity test, nonempty test | 0 | $4P+1$ |
| Transfer to the next operand after opening a rule | 0 | $R$ |

A short/long dispatch can enter the already-known-long path after testing the left operand. A literal implementation that tests it again adds $P$ predicates. Fixed base-pointer setup and allocator internals have no assigned latency. [summary.py](summary.py) prices the table, growth fields and one control unit per allocation call. Its unit field price treats one addressable field like one scalar operation. Its price of two tests sensitivity to greater memory cost. The ten-block results are 0.66% and 5.23% less structural work. At field price one, charging the second type test on each pushed left reverses the advantage. These scenarios exclude copy payload costs common in count. They cannot be read as percentages of elapsed time.

**Line model.** The two traces start with empty, fully associative 32 KiB LRU data caches, 64-byte lines, write allocation and zero-mod-64-aligned regions. A short write reads the advertised byte span from the short arena and writes its destination. A copy references all its source lines then all its destination lines. The baseline reads the two W16 children and position, writes owner positions or a packed 12-byte command in a consumed 16-byte rule slot, then reads the command, chosen position and output. First-occurrence reads or tags the rule in place and uses an eight-byte stack entry. The combined cache includes all these regions. Separate output-only and rule-only caches isolate the two access patterns and are never added together. Dirty writebacks include final draining. The packed study arena, cold initial state, line alignment and instruction-independent line order are modelling choices. Mark-walk warming, stack reallocation, allocator internals, prefetch, conflicts and memcpy's native instruction order are outside this cache trace.

Across the ten blocks, under one unit per fill, saving 40,844 fills tolerates a premium of 0.238 on each of the 171,943 net additional rule fills. If both rule and short-arena fills are dearer, the break-even premium is 0.157. Crediting each avoided dirty writeback at a full fill unit gives 0.610 and 0.404 respectively. These are sensitivity calculations on the same combined-cache counters. They do not equate an L1 fill with a DRAM access. The largest uncertainty is whether the increased random rule and short-arena references are more exposed to latency than the references they replace.
