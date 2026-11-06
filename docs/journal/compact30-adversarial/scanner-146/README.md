# Scanner adversarial supplement: reproducible package

Start with [report.md](report.md). The reviewer-owned branch is
`review/compact30-adversarial-scanner-146`, a draft into
`research/compact30-constructor`, never master and never merged. This package is
separate from Astra's `compact30-research` index and canonical ledger.

The review concerns PR #146 at `7f74e9f30e31af33b0d66fec3fe89af1758307d9`.
Its [input manifest](input-manifest.json) has SHA-256
`e18b6ff9d866ef66a32b287b5cb0f4f81b54cffc038e2275f3513bd76a2691de`.
It identifies 46 input files, the governing document blobs, and the historical
frozen source pins. No input is silently taken from a moving branch.

With the pinned commits available in a local repository, run from that repository:

```sh
python3 docs/journal/compact30-adversarial/scanner-146/replay_all.py \
  --git-repo . --output-dir /tmp/compact30-adversarial-replay
```

The runner uses `git show` to materialize exact immutable evidence into the output
directory, verifies hashes, then runs only Python parsers and abstract integer
models. It does not check out or change another branch, compile Rust, invoke a
constructor, execute archive contents, collect performance data or access the network.
Use a fresh output directory. Git objects must already be present; the script does
not fetch them.

Alternatively, provide an already-materialized checkpoint tree:

```sh
python3 docs/journal/compact30-adversarial/scanner-146/replay_all.py \
  --evidence-root /path/to/evidence-7f74e9f \
  --output-dir /tmp/compact30-adversarial-replay
```

This verifies the 46 checkpoint files; historical/governance blob comparisons in
that mode remain the separately recorded connector reads, not locally re-executed
Git checks. Each of the four JSON results must exactly match its archived result.

Contents:

- [Accounting](accounting/ACCOUNTING.md): independent archived-profile decoder,
  complete build-body totals, normalized baseline bridge, wrapper frequencies,
  explicit memory operand classification, and U1–U3 site exposure.
- [Safety](safety/SAFETY.md): partition and phase proofs, Miri/correctness inventory,
  proposed operation-specific negative controls and counterexamples.
- [Research](phase/proposal.md): split-discriminator reuse and phase-state removal,
  with unresolved residual costs and explicit overlap exclusions.
- [Fresh critique](fresh-check/REVIEW.md): evidence-only review of this package and
  a second independently written aggregate parser.
- [Cursor](review-cursor.json) and [outgoing messages](messages.json): the reviewed
  commit/evidence identity and this reviewer's own publication records.

The release compiler and Miri were not run. Results apply to saved evidence and
finite models, with the exact limits stated in the reports. Existing safe-scanner
nonqualification and historical programme verdicts remain unchanged. The new
research earns no achieved net credit and authorizes no implementation.
