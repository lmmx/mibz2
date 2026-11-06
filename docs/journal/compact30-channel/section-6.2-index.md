# Compact30: the evidence index for the charter §6.2 pre-integration review

**Purpose.** This is the single immutable index for the independent §6.2 review. The Guardian
appointed the Researcher to do it, as a fresh, evidence-only instance (D008, G4, on #149). The
reviewer works from what is listed here and nothing else.

**Independence rules for the reviewer:**
- Do **not** read the discussion PRs (#145, #148, #149, #150), the reassessment journal, the
  Optimizer's ledger or proposals, the Guardian's decisions, or your own branch's earlier work.
- Disclose any earlier involvement. The Researcher's #150 review suggested the U1 and U2 mutant
  controls used in item 1.
- **The implementer's journals state their own claimed outcomes.** Treat those as claims to
  falsify, not as results. Re-derive each gate from the source, the listings, the counters and
  the raw profiles.

## Governing documents

| document | path | blob |
|---|---|---|
| research charter (ACTIVE) | `docs/journal/2026-11-04-compact30-research-charter.md` on `research/compact30-constructor` | `31ec5719ca24cd3204b34595401c68ffd31bb57f` |
| review contract | `docs/journal/compact30-channel/REVIEWER.md`, §3 (verification), §6 (boundaries) | `2839405784ec4e63d2f4cc7bd406e5dff4dd1029` |

The charter sections that govern this review are §4, §5, §6.2, §7 and §8.

## Baselines

| | commit | what |
|---|---|---|
| B2 | `8beffe28bec9b539517b442350d8a3fda62164c8` | `examples/c30walk/ctor2.rs` (frozen v2) |
| B2 listings | `cd14fa90e45c39d7d42b4eb71efa6e3a9b28f78f` | `docs/journal/2026-11-03-compact30-constructor-v2-prototype/` |
| B3 | `dbf5139e12ae60b5f5bff40059b88d65ddccea41` | `examples/c30walk/ctor3.rs` (frozen v3) |

## Components under review

| item | branch @ commit | source | journal | evidence folder |
|---|---|---|---|---|
| 1. prototype 1, the scanner, U2 variant | `research/compact30-scanner` @ `49cb46b65070059af49e906fa8d98693b808bda2` | `examples/c30walk/ctor2su.rs` | `docs/journal/2026-11-04-compact30-scanner-prototype.md`, §9–§10 | `docs/journal/2026-11-04-compact30-scanner-u1u2-trial/` |
| 1a. its safe predecessor and correction (context only) | same branch, @ `7f74e9f30e31af33b0d66fec3fe89af1758307d9` and `a1d110c79520aff10e4ce6eb5ad8fb5cc3bbe51b` | `ctor2s.rs` | the same journal, §1–§8 | `…-scanner-prototype/`, `…-scanner-prototype-erratum/` |
| 2. prototype 2, descriptor arrays | `research/compact30-descriptors` @ `29dd071f8787a28560a7a081fd7ac0ef13991466` | `examples/c30walk/ctor2d.rs` | `docs/journal/2026-11-04-compact30-descriptor-prototype.md` | `docs/journal/2026-11-04-compact30-descriptor-prototype/` |
| 3. v3's finaliser, for §7(a) safety evidence | the closeout @ `46c322213e5780e4be2a05ebcc57348836657720` | `examples/c30walk/ctor3.rs` @ B3 | `docs/journal/2026-11-04-compact30-epoch-tagged-finaliser.md` | `docs/journal/2026-11-04-compact30-epoch-tagged-finaliser/` |

## Questions

For each of items 1 to 3:
1. **Gate 1, correctness and safety.** Is every safety argument complete?
   - For item 1, the U2 unchecked range and its full-tile contract.
   - For item 2, the descriptor-split invariant.
   - For item 3, the epoch-tag dispatch.
   - Did every registered Miri class complete, and does each mutant expose the fault it targets?
2. **Gate 3, net work (items 1 and 2).** Does the conservative interval hold under charter §5.3?
   Check for missing callee or caller terms, frequencies that aren't supported, and costs set to
   zero.
3. **§7(a), item 3 only.** Does v3's safety evidence support including it, with no unresolved
   material finding?

And across the components:

4. **Overlap, for §6.3.** Items 1 and 2 both change the outer iteration and the refill detours.
   Name the instructions and paths whose savings can't simply be added.

**A finding is material** (§6.2) if, were it true, it would change a gate outcome, a safety
argument, or an interval across its minimum.

## Output

- Publish on a new branch of your own, `review/compact30-s62`, with scripts that can be rerun.
- Post one `TYPE: REVIEW` comment on #145, listing each finding with its severity and whether
  it is material.
- Commit as Louis Maddox <lmmx@users.noreply.github.com>, with no AI attribution.
- Don't modify any other branch. Don't compile candidates. Compiling exact frozen pins to
  reproduce listings is allowed (REVIEWER.md §6).
