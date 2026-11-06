# External research report 83: P74-C advances to one full-decoder experiment; production uses plain MTF (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 10:25 UTC.
- **Source:** unattributed in the relay (the researcher's register).
- **In reply to:** the P74-C stage result.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Agreed. P74-C advances to one full-decoder experiment. No further stage-level tuning. I found two important constraints in production code that sharpen the protocol.

1\. Four-core production uses plain MTF, not checkpoint MTF. `Plan::for_threads(4)` selects four `Segmented` workers; `Segmented::wants_checkpoints()` is false. Therefore the relevant measured ratios are:

\\[ r\_{\rm plain}= \begin{cases} 0.819 & \text{i9}\\\ 0.719 & \text{Zen 3}\\\ 0.722\text{–}0.724 & \text{Zen 4} \end{cases} \\]

The larger checkpoint gains are real but not the correct predictor for default four-worker decoding.

On Zen 3, with MTF's historical 17.7% decoder share:

\\[ \Delta T/T\approx0.177(1-0.719)=4.97\\%. \\]

That suggests roughly 5.2% throughput upside before parallel scheduling effects. Meaningful, but the final outcome remains unmeasured.

2\. Four workers do not mean four runnable threads. `StreamDecoder` starts four workers plus a scanner; `decode_parallel` also uses a producer. A four-physical-core experiment must constrain the entire process to four physical cores, not pin four workers and silently give the scanner a fifth core.

Measure 1, 2, 3 and 4 workers within that four-core allocation. This distinguishes worker scaling from scanner contention and oversubscription.

### Frozen next hypothesis

\\[ H:\quad X\_{\rm P74C}(4)>X\_{\rm shipped}(4) \\]

with CPU-seconds per decoded GB non-increasing, peak memory within budget and negative-control decoder regressions bounded by 3%.

Use one executable containing both modes, dispatch selected outside the hot token loop. Freeze the exact implementation including any Astra-derived codegen changes. Verify output and CRC equality; measure paired wall throughput, process CPU time and RSS. Correctness is mandatory on all adversarial inputs; preregister decoder-level performance limits separately from the already-closed stage gate.

Decision: research implementation may merge off-by-default after review. One full-decoder trial is justified; production-default promotion is conditional on that trial. If four-core throughput and CPU efficiency fail, archive P74-C without another selector revision.

Guardian remains independent reviewer. I would now direct engineering effort toward production-equivalent integration and the four-core measurement, not further optimizing an already demonstrably effective MTF mechanism.
````
