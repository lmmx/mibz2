# Compact30 instruction research and verification

Standing reviewer branch: `review/compact30-instruction-ledger`, draft into
`research/compact30-constructor`; never merged. Constructor implementation remains
with Claude. This branch carries static research, reproducible evidence reviews,
and proposals, not authority to extend the sequence.

Start with the [canonical instruction ledger](instruction-ledger.md),
[research brief](research-brief.md), and [review cursor](review-cursor.json).
Replay the baseline with `python docs/journal/compact30-research/recount_baseline.py .`
from a checkout containing the pinned evidence. It reads files only; it neither
compiles nor executes a constructor. [Archived output](baseline-replay.json).

## Governing pins

| Document | Commit | Git blob |
|---|---|---|
| Requested REVIEWER.md | `b302c5db47f4104093ed5f408dcbe5a59720ac7c` | `f5941d00db4c212ef3dab61230a9a4038e4708f1` |
| ACTIVE revision-4 charter | `2f15e513bdb9808a2c7504e3f01b59ebce65f221` | `31ec5719ca24cd3204b34595401c68ffd31bb57f` |
| Current PROTOCOL.md | same base commit | `01d5391874e756ff90bbd9a822913b201cd9dece` |
| Current REVIEWER.md, also inspected | same base commit | `2839405784ec4e63d2f4cc7bd406e5dff4dd1029` |

The current reviewer document adds branch ownership and the canonical knowledge
base/ledger. It does not change implementation authority. The initial branch base
is exactly the commit above; later movement of Claude's base is not silently adopted.

## Frozen evidence

| Evidence | Immutable source |
|---|---|
| B2 source | `8beffe28bec9b539517b442350d8a3fda62164c8`, `examples/c30walk/ctor2.rs`, blob `168c2bac9ad0d8da4d9f82788f009d32f419608e` |
| B2 listings | `cd14fa90e45c39d7d42b4eb71efa6e3a9b28f78f`, `docs/journal/2026-11-03-compact30-constructor-v2-prototype/` |
| B3 source | `dbf5139e12ae60b5f5bff40059b88d65ddccea41`, `examples/c30walk/ctor3.rs`, blob `01f91419904a605b721dc1742ba8087fe79f74cf` |
| B3 closeout | `46c322213e5780e4be2a05ebcc57348836657720`, epoch-tagged-finaliser journal and evidence |
| Scanner handoff | `25d7eb5943dbd4d4cb5119f4aeafd23f0948c6ed`; current blob `b6500f12c757c6d6fc8a2cadd2366f48163bbfb1` |
| Ultra reports | base commit above, `2026-11-03-compact30-constructor-v2-reassessment.md`, blob `afc2a5cb9251445666473c7d3c3cf38181874fd3` |
| Ultra independent audit | same base, `2026-11-04-compact30-ultra-audit-evidence/report.txt`, blob `0ff2f0e594d9459243de9313e7941267f8476229` |
| Scanner checkpoint | PR #146, `7f74e9f30e31af33b0d66fec3fe89af1758307d9` |

Compiler contract: rustc 1.97.0 (`2d8144b78`), LLVM 22.1.6, recorded
`--locked --release` flags. No constructor was compiled for this review. Locally
materialized baseline files were checked against their Git blob IDs; this does
not reproduce the compiler. The text connector could not retrieve binary capture archives; an explicitly
network-enabled Git transport subsequently obtained them. The per-block scanner/refill bounds are now independently replayed in
[structural findings](structural-findings.md). Original timing verdicts are unchanged.

## Coordination and provenance

GitHub authenticated login is `lmmx`; repository metadata reports push/admin and
the collaborator-permission endpoint reports admin. The account profile is
`Louis Maddox <louismmx@gmail.com>`. This verifies access, not a message's role.
Commit author and committer are checked before publication.

At intake, PR #145 contained comment IDs `6628984134` (dummy T1), `6629044041`
(CI explanation), and `6629821793` (scanner checkpoint). No T2–T6 completion or
end-to-end pilot PASS was present. The protocol describes coordinator wake-up;
it does not establish that this Goal subscribes or wakes. No bridge, credential,
scheduled task, or ongoing polling is installed. A real checkpoint notification
or user nudge naming PR/SHA/evidence is the next review trigger.

Preserved outcomes: v1 STOP; v3 compiler STOP; both construction captures INVALID;
traversal VALID ADVANCE. Instructions, memory operations, bytes, dependencies,
cycles and elapsed time are different units. No conversion between them is claimed.

## Completed reviews and new research

- [PR #146 at 7f74e9f](reviews/scanner-146/report.md), [portable replay](reviews/scanner-146/README.md). Evidence digest `aa4f295b5ec152494e999eacf381a92602de329b03e2f76aecd9d6b8c2484753`.
- [V3 closeout at 46c3222](reviews/v3/report.md), with `replay.py --repo CHECKOUT`. Evidence-manifest digest `18e8de1ca5a97cf81ef489ce7e3e24a8f01f42272d1dd05707d24c0689868e85`.
- [Direct-refill proposal](proposals/refill-boundary/proposal.md) and [descriptor gate arithmetic](proposals/refill-boundary/descriptor-note.md). No new prototype authorized.
- [Structural findings](structural-findings.md): independent archive hashes, per-block frequencies and critical-worker exposure.

State: WAITING after reviewing currently published implementation checkpoints. Next
trigger is a new immutable prototype/correction/rebuttal or the §6.2 closeout index,
delivered by an actual notification or user nudge. PR #147 is a proposed shadow
guardian specification, not an implementation checkpoint or additional authority.
