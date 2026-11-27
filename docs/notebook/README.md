# The lab notebook

*Started 27 November 2026 at Louis's direction: "the lab notebook can be where this dense
shorthand is acceptable and the blog can be where we have something worth telling the world".*

The notebook is where the team writes for itself: dated digests of what ran, what was rejected,
what broke and what was learned along the way. Shorthand is fine here. Claim ids, PR numbers,
evidence hashes, verdict labels, ledger names, numbers in code formatting: write whatever makes
the record fastest to read for someone who already knows the project.

The blog (`docs/site/blog/`) is the opposite. It is for a reader outside the project, and it
carries only news worth telling the world (`frontier/README.md`, "The blog board").

## What goes here, not on the blog

- **Process:** how the work was organised, reviewed, queued, run or merged. Process narration
  is never a post.
- **Negative results on their own:** a candidate the hardware rejected, a design that missed its
  gate, a probe that found nothing. Record it here with its evidence. It reaches the blog only
  as part of a positive finding it explains.
- **Digests and syntheses** of recent deltas and PRs.
- **Measurement traps, harness behaviour, CI incidents, tooling fixes.**

When something here turns out to be news (a theoretical discovery, or an advance in the
software itself), open a blog card for it at stage `idea` and write the post for an outside
reader, from the evidence. Don't adapt the notebook entry; its shorthand is the wrong starting
point.

## Entries

One file per entry, `YYYY-MM-DD-<slug>.md`, newest last by name. Start each with a heading and
a one-line summary; after that, any format. There is no prose check, no figure requirement and
no limit on how often to write. Entries are append-only in the same spirit as deltas: correct
an entry with a later one rather than rewriting it.
