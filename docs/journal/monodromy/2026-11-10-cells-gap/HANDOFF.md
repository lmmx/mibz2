# Cells: the missing whole-decoder gain

Assignment: [#363 comment 6782141557](https://github.com/lmmx/mibz2/issues/363#issuecomment-6782141557),
reconciling Louis's direct instruction with the Acting Coordinator's queue. Branch:
`investigate/cells-end-to-end-gap`, from master
`2635f04e9155dd31388b24fd3f7248f87b3ddcda`. PR #365 stays frozen at
`57f51bbc0ca62ac3ee1cc148a1010842eea6222a` for the Researcher's independent B5 review.
No production source, existing benchmark, admission threshold, default, team path,
workstation queue or review record was changed.

**Recommendation:** implement the diagnostic only (done); defer kernel changes until
one matched warm-port/research comparison plus decode-context trace identifies a
material cost. Admission dilution is not the leading explanation supported by the
data. A source-level writer difference is concrete, but its timing cost is unmeasured.
The Optimiser's independent whole-cost pricing remains a separate assignment.

## What is now verified

Recovered the original i9 bundle from evidence commit
`d85b2074099f027d048aa7a057da10d25f2b70bb` and **verified its archive SHA-256**:
`552e26aaf3b41636afdbd5a195ad5f0ce1c66aed945ec2911a508057baed705c`.
Its recorded research source is `c79ff7d384072e5e95eda7e513227f2aa06ff3d5`.
The original three raw censuses are copied without remeasurement into `evidence/`.

Recovered CI's byte windows from bench-data commit
`a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a`. A narrow, hash-checked reconstruction
reproduces the **exact 11,854,933-byte corpus**:
`82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.
The new safe wrapper calls the frozen library's actual scanner and `decode_block`,
observing its public trait and per-kernel route counters. It reads no clock in census
mode. All **431 blocks**, **209,706,557 BWT rows**, and **209,700,196 output bytes**
agree with Segmented and independently with Python/libbzip2. Block CRCs also pass.
Complete decoded SHA-256:
`3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`.

| Actual route | Blocks | BWT rows | Output bytes |
|---|---:|---:|---:|
| cells admitted | 196 | 176,398,965 | 176,391,605 |
| density refusal | 185 | 31,507,152 | 31,508,149 |
| cell-budget refusal | 2 | 1,799,975 | 1,799,977 |
| column route | 48 | 465 | 465 |
| floor / invalid refusal | 0 | 0 | 0 |

Cells covers **84.117% of rows**. The 198 blocks omitted by the research census's
65,536-row minimum contain only **8,895 rows (0.00424%)**. Their block overhead is not
measured here, but row omission cannot account for a large missing row-weighted gain.
The remaining 233 blocks match the historical census's lengths in order, and every
production admission/refusal decision matches the rule applied to its recorded
`records`, `c_letters` and `c_cells` values. Of these large blocks, 196 admit, 35 refuse
on density and two on budget.

## Historical counterfactual, not a new performance claim

Using only the original i9 timings, substitute production's recorded inversion cost
for the blocks that current admission refuses, while retaining research cells cost
for admitted blocks:

| Repeat | Original all-cells / records inversion | Hypothetical admitted-cells + fallback / records |
|---|---:|---:|
| 1 | 0.815 | 0.773 |
| 2 | 0.827 | 0.782 |
| 3 | 0.818 | 0.775 |

This counterfactual **omits attempted-then-refused work** and assumes the research cost
survives the production port. It is not a measured production-port ratio or a decoder
forecast. It shows that the density filter removes relatively expensive research
cells blocks rather than selecting away their gains. The implied historical saving
is 70–74 ms of inversion over the sampled large rows, not a measured end-to-end saving.
Actual budget-refusal cost and the production implementation's cost remain unmeasured.

## Why the existing comparison cannot locate the loss

1. **It is not a matched i9 whole-decoder experiment.** The inversion baseline is i9;
   the final whole-decoder CI observation is EPYC 9V74. A ratio of 1.000 there does not
   establish that the i9 advantage disappears on the i9.
2. **Research and the port differ.** The research x86 small-copy helper uses masked
   SSE loads/stores; its long copy uses an explicit 16-byte loop. The production port
   uses `copy_nonoverlapping` / overlap-permitting `copy`. A release disassembly of
   the frozen port contains `memcpy` / `memmove` call sites in `write_cells`. This is
   a concrete candidate, not proof those calls dominate. The port also adds admission,
   budget checks, different bounded layouts and allocation/reuse behavior; none may
   be removed merely to recover the research number.
3. **The conditioning and cost boundary differ.** `rauzy` records the best repeated
   inversion of the same block with records already prepared, after other kernels.
   The decoder runs scanner/front/inversion/RLE1/CRC in order and hashes/copies output
   through a reader. The existing `examples/phases.rs` still times the column build
   and walk, not this actual records/cells route, so its stage shares cannot locate
   the gap. Reader-adapter overhead remains another unmeasured candidate.

For matched measurements only, if inversion occupies fraction f and is sped up to
ratio s, the no-extra-cost decoder ratio is `1 - f*(1-s)`. Added refusal, allocation or
other costs must be charged separately. Neither f for this real path nor s for this
production port is supplied by the old cross-machine comparison.

## Delivered diagnostic and next decision

[`bench/cells-gap`](../../../../bench/cells-gap/README.md) is a separate, locked crate
depending on the frozen revision, not the branch's production sources. No new unsafe
code is introduced. It supplies:

- Untimed census with actual route and row counts, per-block SHA/CRC parity, independent
  libbzip2 oracle check, and replay of the archived admission/cost counterfactual.
- Optional `trace` mode around actual `decode_block` and its inner inversion, alternating
  arm order across full-file passes; hashing/serialization outside measured spans.
- Optional `warm` mode matching the research census's minimum length and repeated
  inversion conditioning, with byte equality checked after each Cells/Segmented pair.
- An untimed CI workflow; it does **not** run either timing mode or any i9 job.

The README gives a bounded three-round diagnostic recipe and decision branches.
**Next action:** Coordinator and Optimiser assess that recipe, then arrange one matched
machine/toolchain/corpus run of frozen research versus production warm inversion,
alongside the real decode-context trace. If the warm port already loses, price one
writer/port control first. If the warm advantage survives but context does not, test
conditioning; if context inversion wins but total does not, price the remaining stages
and add a matched reader control. Do not tune all candidates or broaden rollout.
Any workstation run still needs its checked C5 recipe and Louis's runner; none ran here.

## Validation and limits

Local build/test compiler: Rust 1.99.1. Formatting and Clippy: installed Rust 1.90.0.
The diagnostic's real-decoder fixture test checks original bytes, admitted/column
reach, timed/untimed parity and the warm-mode correctness path; its incidental clock
reads are not a performance measurement. The first test incorrectly required every
fixture to admit; that assumption was corrected to assert both actual routes across
the set. No production change resulted. Final test, Clippy and formatting pass.

The exact corpus reconstruction, all-block census, independent oracle and historical
join passed locally. Two source builds gave identical untimed census JSONL. The source
and evidence manifests identify what was checked. Release writer disassembly is static
evidence only. No timed performance run, acceptance verdict, new unsafe optimisation,
B5 certification or i9 run is claimed. CI results and the separate PR are recorded in
the associated #145 REVIEW after opening it.

## Launcher reality

The enabled `mibz2⠶Coordinator⠶Monitor` automation is a Work task, not owned by this
Codex session. No Codex automation appeared in the inspected task configuration; the
merged CODEX-RELAY-PILOT explicitly says issue comments do not wake Codex. The new
#363 assignment was consumed by **this already-running session**, not a demonstrated
event restart. The Acting Coordinator can post and route the next assignment but
cannot currently restart this session through the configured event loop. A deliberate
Codex launch/resume is required. No automation was created, changed or run to simulate
delivery, and future launcher support must be saved and observed executing before it
can replace that manual step.
