# F01: asserted reach of the cells long-word writer

Assignment [#363/6782519694](https://github.com/lmmx/mibz2/issues/363#issuecomment-6782519694),
answering the [independent Researcher finding](https://github.com/lmmx/mibz2/blob/9693874f0795083d4dabb5caec9a6b28af6a928e/state/reports/cells-b5-6782111848/report.md).
Parent: `57f51bbc0ca62ac3ee1cc148a1010842eea6222a` on PR #365. The original source and
benchmark evidence remain at that commit and in their existing journal directories.
No production operation, admission rule, default, team route or benchmark was changed.

## Correction

`Kernel` has a `cfg(test)` per-call reach record. Increments sit inside the real writer
branches, after position seeding/propagation and copies; they compile out of production.
The new tests assert positive reach, exact output against the shipped Plain walk and
the Researcher's expected cell/period counts. Output is complemented before each call.
Both tests then decode a smaller block with the same arena, position and output
allocations, assert parity and retained capacities, and verify per-call reach reset.

| Witness | Cells / period | Seeds | Descending / propagation | Expansion | Long-factor copies | Final recopy | Repetition |
|---|---:|---:|---:|---:|---:|---:|---:|
| `1^16 0^15`, p=16 | 17 / 31 | 1 | 7 / 6 | 7 | 0 | 0 | 0 |
| BWT of `(a^9 b^9)^3 c`, p=0 | 37 / 55 | 2 | 2 / 1 | 2 | 0 | 1 | 0 |
| Same column, p=3 | 38 / 55 | 2 | 6 / 5 | 6 | 1 | 0 | 0 |
| Each column byte repeated ten times, p=0 | 57 / 55 | 2 | 2 / 1 | 2 | 0 | 1 | 4 |

The last witness calls ordinary `Cells::invert_runs`, with n=550, R=8, seven marked-split
letters and B=88. It asserts admission, no refusal, and the selected period 55. It is
not forced. All proposed witness counts reproduced; no discrepancy was suppressed.
The earlier short-cycle Miri tests remain unchanged. The workflow checks both new names
are selected, so a rename cannot silently remove their coverage.

The new `missing-spine-recopy` mutant removes only
`copy_long(dst.add(d), dst.add(f), lx)`, leaving the reach increment intact. Thus the new
fixture must fail its byte-parity assertion, not merely a missing counter. The existing
five mutations and original evidence are retained. SAFETY now distinguishes asserted
long-rule coverage from the old fixtures and explicitly records F02: the implemented
floor is marked-split `letters - 1`, not union-of-cuts `r2 - 1`. No admission redesign.

## Validation

Results and exact source/tool identities are recorded in `evidence/`. The initial Miri
setup failed before executing tests because its default cache was read-only; the retry
sets `XDG_CACHE_HOME` to writable workspace storage. That failed setup log is retained.

- Full selected Miri suite: **39 passed, zero failures**, complete in 1155.64 seconds.
  Both new tests print the reach counts above. Expected caught panics in existing canary
  and worker-recovery tests are retained in the log, not test failures.
- Native cells unit tests: **7/7**; real-decoder differential tests: **5/5**, including
  the new long-writer witnesses with successful libbzip2 decoding, scalar/detected
  kernels, CRC parity and asserted ordinary admission.
- Mutation campaign: baseline passes; **6/6 compiling mutants killed**. The new recopy
  mutant fails byte parity in both new writer tests, leaving complemented bytes in
  output. It does not merely fail a reach assertion. No timeout or build failure counted.
- Kernel SHA-256: `42e377d8a980c94d95a4b162b0f7ba6466ff6b3844cbc8b10fce933fd25692b2`.
  Miri and the final native/mutation checks used this exact source. The initial native
  probe preceded the addition of the explicit period counter; it is not substituted
  for final-source evidence.
- Clippy (all features, library/tests/ab/fuzz) and Rust formatting pass. Workflow
  selection was checked with the actual Miri `--list` output.

`miri.raw.log.gz` preserves exact stdout/stderr bytes; `miri.log` is the same text with
trailing whitespace removed for repository formatting. Both are hashed. Other logs are
copied unchanged. The previous fuzz and benchmark evidence remains applicable to unchanged
production code; neither was rerun locally for this coverage-only correction.

Commands use the installed toolchains with `CARGO_BUILD_JOBS=4`, and one heavy job at a
time. Native and mutation checks use Rust 1.99.1; Miri uses the installed nightly.

```sh
cargo +nightly miri test --lib -- miri_ --nocapture
cargo +1.99.1 test --lib cells::tests -- --nocapture
cargo +1.99.1 test --test cells
RUSTUP_TOOLCHAIN=1.99.1 python3 docs/journal/monodromy/2026-11-10-cells-slice-one/mutations.py WORK_DIR
cargo +1.90.0 clippy --lib --tests --example ab --example fuzz --all-features -- -D warnings
cargo +1.90.0 fmt --all --check
```

No new performance measurement or workstation run is requested or performed. Automatic
CI after this substantive correction is separate evidence and cannot erase prior
benchmark outcomes. The prior incomplete macOS jobs remain recorded as such.

## Independent decision still required

This is the implementer's targeted correction, not B5 self-certification. The Coordinator
must route the exact commit, source digest, reach/mutation/Miri evidence and any CI limits
to the Researcher for the required targeted recheck. F01 remains open until that independent
acceptance. The queued occupancy-ledger repair is a separate assignment and PR.
