# Singletiny through the normal parallel decoder

Putting the preserved BMI2 Huffman phrase scanner in front of the same frozen scalar-cap worker produced **no resolved local whole-decoder wall improvement**. In both cases the actual scorer built the worker pool and drained the output using the stock `BzReader::parallel_workers_with` entry point with 4W slots. The direct binary-to-binary wall ratios over 32 complete rounds were 0.9441 [0.8023, 1.1108] at W1 and 1.0122 [0.9513, 1.0770] at W4. The paired intervals grouped in consecutive sets of four rounds were [0.8099, 1.1004] and [0.9319, 1.0994]. Every one of the 128 runs decoded exactly 419,400,392 bytes from Wikidata×2. This host has four CPU-equivalents. These observations supply no i9 throughput or memory decision. [Preregistration](PLAN.md) · [Raw rounds](direct/runs.jsonl) · [All ratios](direct/summary.json)

| Workers | Median phrase / stock wall, s | Phrase / stock CPU ratio [95%] | Median phrase / stock peak, KiB | Peak RSS ratio [95%] | Maximum phrase / stock peak, KiB |
|---:|---:|---:|---:|---:|---:|
| 1 | 1.277 / 1.271 | 0.9475 [0.8207, 1.0940] | 71,106 / 71,352 | 0.9980 [0.9928, 1.0033] | 72,212 / 73,184 |
| 4 | 0.833 / 0.790 | 0.9886 [0.9291, 1.0520] | 100,312 / 99,310 | 1.0068 [0.9963, 1.0174] | 103,744 / 104,136 |

Wall, CPU and peak intervals use paired log ratios. The table's separate wall medians need not have the same ratio as the geometric mean of the paired observations. At W1 an individual stock observation reached 5.0 s, with a 1.27 s median. The full set of observations enters the registered decision. A pre-existing low-priority compile briefly overlapped the beginning before suspension. [Host receipt](direct/host.txt)

## Separately built and independently checked

The release’s Huffman parser, ported `singletiny.rs` and unchanged worker are independently identified. Both builds drew from frozen scalar-cap `d0a42980e896211906234c7679b38760ae44a46e`. The treatment uses the scanner generator preserved at `78c2fa6` and first tested at the isolated scanner entry point. In its detached checkout, BMI2-capable x86-64 processors select the new scanner from the original `src/block.rs::read_item` entry point. All other processors use the original parser. Both paths return the same original Symbols and Item types. The source change is private to that checkout.

The full scorer validated 338/338 cases in each executable, with libbzip2 byte comparisons, periodic and multiple-cycle inputs, mutation cases and CRC integrity. Both also passed the release's 4,000 generated malformed or valid normal-entry reads including end bit and overrun comparisons. Comparing rustc’s 33 checkout-local source dependencies gave a single difference, `src/block.rs`. The scalar generator writes three absolute checkout paths into module declarations. Each was normalized solely for comparing worker-source identity across the two build locations. The raw scorer and executable receipts are preserved. Fetched master was `5298b23c7c89048c8986979e19b67c840397f0a0`, scorer path `75297a1d57b2bdf262c3a19f3064848dedf2c493`, both printed `scorer v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The official binaries hashed to `f2bb49927dc688e8c910f5e6438401079a4b715881bf12d10c50466e84e63902` for stock and `5b523c365f455ff06944c7f367c990b1d99e16936aab93ca42f436e331fd6a09` for the treatment. [Gate, hashes and dependency comparison](official/verified.json) · [Source patch](official/scanner-phrase.patch)

## Scanner, handoff and worker measurements

A separate matched 8-round diagnostic added the release’s existing level-2 per-thread timeline to both executables. Each of 32 runs independently accounted for all 862 block indices from issue, enqueue, worker acquisition, decoding and publication to ordered consumption. All worker output lengths summed to the verified decoded length. These builds emit timestamps and record worker CPU and scheduler data. The uninstrumented binaries above provide the primary wall and RSS estimates. [Per-run stage totals](trace/roles.csv) · [Paired intervals and raw medians](trace/role-summary.json) · [Raw marks, one gzip file per decode](trace/)

| Median per complete corpus, phrase / stock | W1 | W4 |
|---|---:|---:|
| Scanner thread CPU, s | 0.241 / 0.219 | 0.270 / 0.258 |
| Scanner read/parse interval, including input wait, s | 0.236 / 0.217 | 0.322 / 0.394 |
| Scanner elapsed waiting for a work slot, s | 1.280 / 1.409 | 0.589 / 0.770 |
| Scanner elapsed waiting for compressed input, s | 0.0069 / 0.0025 | 0.0032 / 0.0066 |
| Mean issue-to-worker acquisition per block, ms | 4.635 / 5.069 | 5.428 / 5.600 |
| Worker decode CPU sum, s | 1.427 / 1.540 | 1.587 / 1.916 |
| Mean decode-end to ordered consumption per block, ms | 0.140 / 0.183 | 6.279 / 8.060 |

Scanner CPU ratio intervals were [0.8733, 1.3643] at W1 and [0.7484, 1.3843] at W4. The earlier isolated scanner’s 2–4% favorable direction did not resolve in this parallel pipeline. The scanner often had to wait for a slot. Some queue and per-block intervals overlap across threads, and the waiting times include scheduling. Summing those intervals would double-count wall time. The target at seven through ten workers requires independent whole-machine throughput and RSS measurements.

The scorer’s existing chaingrind region starts after symbol extraction and calls `decode_block`. Both executable builds use identical source inside that region. It excludes the scanner whose cost is at issue, so this experiment records no complete-scanner instruction, cache-transfer or dependency-chain estimate. The separate phase receipt measures thread CPU and handoff latency directly.

## Cross-built release references

A separate 12-round screen ran the scorer’s explicit LazyCells and opend references from the unmodified-scanner binary. The candidate used the phrase/scalar binary. All three arms occupied each run position four times per worker count, with a two-second settle after each process. Each reference passed all 98 quick libbzip2 cases before timing, and all 72 measured processes emitted 419,400,392 bytes. These comparisons include both the changed scanner and the different inverse worker. The direct 32-round panel above isolates the scanner change. At these low worker counts the scorer explicitly selects its reference kernels. [Frozen schedule](ANCHORS.md) · [Ordered observations](anchors/runs.jsonl) · [Measured ratios](anchors/summary.json) · [Scorer summary](anchors/scorer-summary.json)

| Workers | Reference | Phrase / reference wall [95%] | CPU [95%] | Peak RSS [95%] | Max phrase / reference, KiB |
|---:|---|---:|---:|---:|---:|
| 1 | Lazy | 0.9815 [0.8700, 1.1074] | 0.9924 [0.8919, 1.1043] | 0.8894 [0.8836, 0.8952] | 72,456 / 80,400 |
| 1 | opend | 1.0427 [0.8896, 1.2222] | 1.0353 [0.9012, 1.1894] | 0.9204 [0.9136, 0.9272] | 72,456 / 77,856 |
| 4 | Lazy | 0.7252 [0.5964, 0.8817] | 0.7761 [0.6830, 0.8818] | 0.7341 [0.7080, 0.7612] | 102,664 / 145,704 |
| 4 | opend | 0.6956 [0.3845, 1.2587] | 0.7412 [0.4613, 1.1907] | 0.8598 [0.8355, 0.8847] | 102,664 / 118,552 |

At W4 the whole phrase/scalar executable was faster than Lazy over these 12 local rounds. The independent effect of substituting the scanner remains unresolved. Every paired peak fell below its reference. The final W4 opend sample took 11.892 s versus its 0.550 s median, and the paired wall interval for opend includes one. The full interval retains every observation. The four-equivalent host and low-worker results leave the i9 throughput and peak-memory decision open.

## Replay

Use a new private work directory for each paired build. The scanner-source input is generated by the isolated `native`, `scatter` and `selectors` build commands in the parent README. It must hash to `df6ad1d33b301fa8b6b593cff6a5e965cd9e9dcbf30edb5eabb57238869ef171`. For example, from the repository root with Rust 1.99.1 and the canonical input:

```bash
d=$PWD/research/astra-e/huffman-phrases/integrated
b=/tmp/astra-huff-integration-new
e=/tmp/astra-huff-receipt-new
t=/tmp/astra-huff-traced-new
g=/tmp/astra-huffman-replay/selectors/singletiny.rs
c=$PWD/corpora/wikidata.bz2
bash "$d/gate.sh" "$b" "$c" "$g" "$e/official"
python3 -B "$d/verify.py" "$e/official" "$b" > "$e/verified.json"
bash "$d/refresh-round.sh" "$b" "$e/official"
python3 -B "$d/panel.py" run "$b" "$c" "$e/direct" --rounds 32
python3 -B "$d/panel.py" summary "$e/direct/runs.jsonl" > "$e/direct/summary.json"
bash "$d/trace-build.sh" "$t" "$g" "$(head -1 "$e/official/refresh-stock.txt")"
bash "$d/refresh-round.sh" "$b" "$e/official"
python3 -B "$d/panel.py" run "$t" "$c" "$e/trace" --rounds 8 --traced
python3 -B "$d/roles.py" "$e/trace"
bash "$d/refresh-round.sh" "$b" "$e/official"
python3 -B "$d/anchors.py" "$b" "$c" "$e/anchors"
```

The corpus's original SHA-256 is `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. At each pricing round the worktrees refresh from master and require the same scorer projection as their binary builds. If a later revision changes that projection, rerun the matching gates on all binaries. The preserved working projection allows recovery from a documented scorer-interface failure.
