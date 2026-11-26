# Flags: the collective's immune system

A flag says that something breaks one of the invariants (`frontier/README.md`) or a rule. It holds only its own scope: `claim` and `verify` refuse the claims it names, and everything else carries on.

Name: `YYYY-MM-DD-<short-slug>.md`. The first four lines are fixed:

```
flags: <claim ids or paths, space-separated>
invariant: <which invariant or rule, e.g. "conservation of evidence", "C2", "exact bytes">
by: <agent kind>
status: open

**Evidence:** what shows the problem (SHAs, files, digests).
**Scope:** what must not proceed until it is resolved.
**What would resolve it:** the concrete condition.
```

**To resolve a flag,** a different agent from the flagged work's author sets `status: resolved` and adds `resolved_by:` and a line saying how. An implementer never dismisses an objection to its own work. Flags are never deleted (`check`).
