# Seven-family Cells eligibility and memory screen

Frozen before the first corpus check, 13 November 2026. Implements Coordinator
queue 6850979995 under the owner's Cells shipping mission; this is not a timing
experiment or a replacement for the completed 120+9 diagnostic.

Question: do all seven pinned inputs decode identically, which blocks reach the
existing Cells writer, and does a native child memory measurement pass controls?
Decision: byte/length/CRC mismatch, changed source/input identity, timeout, missing
record or failed memory control stops the screen; retain every partial result.
Integration consequence: use actual writer eligibility to select the next bounded
candidate with MONODROMY and Coordinator; no speedup or default decision here.

## Immutable inputs and execution inventory

Decoder d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2; existing live-reader preparation
basis cb73bf909397e2fff8c90f0a00437061ad06478e. Rust 1.99.1 x86_64 Linux, release
fat LTO, one codegen unit. Preserve decoder operations and the live harness's
4 MiB drain/hash. Diagnostic additions use checked slices and counters, no clocks.

Generator and text tree 57f51bbc0ca62ac3ee1cc148a1010842eea6222a, scale 4,
bundled libbzip2 1.0.8 level 9. Generate from an isolated exact tracked tree;
record ordered text paths, lengths and SHA-256s. Use canonical Wikidata compressed
bytes (windows pin a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a), never recompress them.
All seven compressed identities are checked against `inputs.json` before launch.

Exactly 23 full-input/control child executions: touched 1 MiB then 64 MiB native
controls; then, in order, wikidata, onebyte, pattern, random, records, runs, text,
with Segmented, Cells and independent bundled libbzip2 once each. No warmups,
replacement runs or measured timings. Each child has a 30 second timeout, entire
screen a 15 minute deadline. Input preparation/build timeout is 30 minutes.
Small fixture unit/dry-run tests are separate correctness tests, not full-corpus
replays. CI executes the same frozen inventory once per reviewed harness head.

Each decoder writes its complete output; compare byte-for-byte with the oracle,
length and SHA-256, then retain compressed output files and hashes. Retain logs,
source/input/build identities and failed output on failure. Four previously
unknown decoded hashes are observations, never replacements for compressed pins.
Cells records block n, capped run-record r, period p, budget B, route/refusal,
small/long unmarked spine seeds, maximum seed length, reachable unmarked expansion
depth, root depth and diagnostic visits. Post-induction eligibility is not an
early dispatch feature. Diagnostic scratch and traversal are explicitly reported.

The tiny native launcher forks/execs a child, waits for that exact PID with wait4,
and records exit/signal plus ru_maxrss. The Rust child separately reports its PID,
post-exec startup/end VmRSS and VmHWM from /proc/self/status, also on ordinary
errors. Controls must touch every allocated byte and stay alive through sampling;
64 MiB minus 1 MiB VmHWM must be at least 32 MiB. PID and wait4/native HWM must
agree (within 4 MiB for exit-time sampling effects). Any absent or implausible
memory evidence blocks subsequent performance admission. The parent's RSS is
never substituted. Counter-instrumented memory is descriptive, not hooks-off
production memory qualification.

## Preserved boundaries

Existing CRC/error, concatenated-stream, forced-route and F01 regression tests
remain required. No original unsafe operation, fallback, admission threshold,
writer, decoder default, team path or experimental gate is changed. Independent
B5 review is not self-certified. No i9 run, ready tag, queue entry, merge, release,
140+21+2 timing programme or warm20+20 is admitted by this preparation.
