# 3. Current team machinery

- **Doing:** finishing the first Oracle account after the machinery cutover.
- **In order to:** 0, keep current authority, purposes and recoverable state aligned with whole-decoder work.
- **Status:** the migration, #225 merge and four-task re-pin are done as intended; this version supplies the first rebuild and drift test. The result of Louis's Oracle verification is not established.
- **Known:** [Louis's recorded decision](https://github.com/lmmx/mibz2/blob/3421fe600f544724425db665caf06fa89dd25f06/docs/journal/decisions.md) records approval of the full rules and MODE choices; [#225](https://github.com/lmmx/mibz2/pull/225) merged at `282fcbb0dd273196052fc9637a7d7516918e684c`; [6686470751](https://github.com/lmmx/mibz2/pull/145#issuecomment-6686470751) records read-back and reopened dispatch. This run independently verified the Oracle's old head and migration digest before committing its receipt. Other roles' migration verification is the Coordinator's recorded report, not an independent audit here.
- **Next:** Louis assesses the configured Oracle; #220's result handoff remains his decision. Cosmetic old task names do not revive old authority.
- **Drift test:** see [weak chain](../weak.md). An applicability question was justified on 4 November, without treating any unauthenticated comment as authority to retire a rule.
