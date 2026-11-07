# 3. Current team machinery

- **Doing:** maintaining recoverable state, intelligible direction and request delivery, in order to serve objective 0.
- **Completed:** initial migration, plain-language account, single-version page loading and footer link. Live deployment not rechecked ([previous version](https://github.com/lmmx/mibz2/blob/d91c869de27270a6f662aa7f530ebda54eea357b/current/L3/3-team-machinery.md)).
- **Forward relay discoverable:** #238 merged b61c343273a80df90c9064155b0b98de152fc850, documenting Louis's explicit requests to the Coordinator through #145. Work roles do not act as that relay ([completion](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695146778)).
- **Observer rename complete:** #239 merged aa5ce6f52796405feed4dba0a9f88c592f14a931, now this run's authenticated pin. Same work/authority/history, inbox #222 and state branch team-state/oracle. The earlier proposal to move state was withdrawn to preserve migration continuity ([correction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695463792)).
- **Return relay complete:** #240 merged d2c92e897db4a2883d114f50d2a43c4ecf1a7df0. Louis's [recorded decision](https://github.com/lmmx/mibz2/blob/d2c92e897db4a2883d114f50d2a43c4ecf1a7df0/docs/journal/decisions.md) distinguishes read-only check-in from consent to process. Only named existing task re-pins using the prescribed form, to a full master-contained commit with both specs present, are eligible without prior text review. Read-back and Coordinator verification follow. No autonomous polling or arbitrary task changes are implied.
- **First completed return request:** [6696702367](https://github.com/lmmx/mibz2/pull/145#issuecomment-6696702367) verifies the already-saved Observer instruction text and closes 6696591064; do not repeat the edit. This run itself loads that new pin. Platform name/trigger/enabled settings are reported by the read-back, not independently inspected here.
- **Idempotency refinement complete:** #241 merged abdca2d3c8a2fe5197ccb5d087a5c2f36d43ab16. Compare saved instructions before writing; a Coordinator LOG can close a verified request.
- **Historical drift test:** [retained test](../weak.md#first-run-applicability-drift-test); a name is not authority. This pin explicitly retains the old state address.

## Workstation evidence submission: completed

- **Purpose:** remove manual evidence handover while preserving the frozen experiment.
- **Status:** #242 merged as 9ece704953d815d1d0e8dd184c99defce273dd4d; first end-to-end use is confirmation [6698069738](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698069738), fetched, checksum-checked and re-judged by the Coordinator in [6698087098](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698087098).
- **Behavior:** Louis's opt-in `just submit <recipe>` runs the frozen recipe unchanged, archives bundle/checksum at an immutable evidence-branch commit and posts identity to #145. Ordinary runs submit nothing; `just submit-bundle <tarball>` retries delivery without another run. Deduplication uses run and digest.
- **Authority history:** Louis's recorded C5/D5 approval is now merged; the command counts as handover of his own run, not automatic qualification or permission for agent timing. This Observer still executes the authenticated aa5ce6f5 pin; it observes this history without widening its own remit.
- **Sequence resolved:** submission integration and confirmation are complete. Keeping the old tooling wait would now misstate direction. Release validation and publication are now complete; the release-scaling files are now recorded on master (see 1.6).

## Release procedure

- **Doing:** separate runbook [docs/RELEASING.md](https://github.com/lmmx/mibz2/blob/8823f617717aa8bf02e91e95f9b4b24284e388e9/docs/RELEASING.md) at Louis's request, referenced by CLAUDE.md.
- **In order to:** 0, make release checks/publication/follow-ups reproducible.
- **Status:** present at 8823f617717aa8bf02e91e95f9b4b24284e388e9; #244 merged the current/previous-tag defaults. Runbook distinguishes mandatory pre-release checks from the informative post-release workstation scaling request. That run is Louis's and does not hold publication.
