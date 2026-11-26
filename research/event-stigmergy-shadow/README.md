# Reactive stigmergy: wake a research relation, not a permanent role

26 November 2026. Research note and offline shadow prototype, not an installed agent harness.

## Motivation

Jha et al., [s³](https://arxiv.org/html/2611.08615v1), §3.1, use asynchronous message arrival to resume idle agents without spending tokens while idle. The environment in that experiment is private to each agent. Our persistent shared repository permits a different stimulus: a change to a premise, an owned path, or a claim's state. The maintainer suggested associating changed parts with the participants who previously worked on them. The useful refinement is to associate changes primarily with research relations, not identities. A returning live participant may have context worth preserving; a fresh participant must be able to use the same signal after rupture.

A mechanical observer can project relevant events into shared state without interpreting the science, assigning a role, or composing a natural-language prompt. A notification may be an optional hint to an already-live subscriber; it cannot be the only copy of the event. A PR is a preview of a possible change; a merge is shared evidence. Do not equate them.

## Concrete evidence from the current repository

A small manually inspected retrospective (selected examples, not a census or timing experiment):

| Event | Existing relation or state | What a mechanical observer could expose |
|---|---|---|
| #952, 3539780 → 2aaded3, new cold-store/RLE1 evidence | delivery-copyout was open, tagged delivery, but had no declared premises in its pre-event card | A path-only observer would miss this correction; later scientific synthesis did connect it. No complete relevance oracle. |
| #960, 55cb385 → c1915fd, MTF-fold census and registration | mtf-fold-build idea → open; new explicit premise paths | A committed admission is an observable stimulus without looking up who did the census. |
| #966, f46b969 → 94bd4c6, delivery census | delivery-l3-working-set already declared three premise paths, none touched by this PR; both claims concern delivery | A shared mechanism gives a weak candidate even with no path match. The idea remains an idea. |
| #968, proposed head e12e03b | mtf-fold-run exists on proposed branch, not on master at inspection | Preview only. No launch of a dependent experiment based on unmerged code. |
| #973, e88ce73 → c24b41a, review visibility correction | harden-frontier-tooling open → review | Integration stimulus, not a new build. #974 separately addresses live PR discovery itself. |

The path and mechanism examples are illustrative and selectively sampled. They do not establish recall, precision or a productivity effect. In particular, a changed premise is a reason to reread, not proof of contradiction. Shared mechanism is weaker still.

## Shadow prototype

`replay.py` reads two Git commit trees and their diff without changing a ref, writing shared state, notifying anyone or launching a model. It emits JSON observations for claim-state changes, pre-event premise or owned-path matches, and optional shared mechanisms. New dependencies in the after-state are not retroactively used to predict the event. Terminal claims are retained only for lifecycle observations, not proposed new work. The caller explicitly labels the event committed or proposed; that label is not verified merge authority. No file changes infer semantic relevance by themselves.

Run from a repository clone:

```sh
python research/event-stigmergy-shadow/replay.py 55cb385 c1915fd --source mtf-fold-census --kind committed
python research/event-stigmergy-shadow/replay.py f46b969 94bd4c6 --source delivery-copyout --kind committed
python -m unittest discover -s research/event-stigmergy-shadow -p 'test_*.py'
```

Four synthetic unit tests passed locally during preparation; the historical cases above were inspected via exact repository snapshots and PR file lists, not executed through a full cloned-repo replay. No CI result or deployment is claimed.

## What to test next, before activation

Replay a bounded sample of ordinary first-parent trunk merges and corresponding pre-merge PR events. Label relevance independently from the emitted candidates. Compare path-only with path plus lifecycle, and then path plus lifecycle plus shared mechanisms. Report missed material dependencies, false positives, fan-out per event, duplicates and the fraction of claims with missing premise links. A useful signal is one that could change a participant's next action: reread an affected premise, examine a completed build, or consider an admitted claim. Do not turn every changed file or shared topic into a wake-up. Record latency from relevant change to first substantive subsequent contribution where timestamps permit; this is observational, not causal proof.

Only if the shadow signal proves useful consider an event-driven comparison with ordinary pull-based discovery, at matched inference budget and existing claim cap. Measure time to useful review/integration, avoided stale work, useful discoveries per token, false wake-ups, human interventions and total overhead. Retain a polling/pull fallback. An event key based on committed SHA, claim and reason can suppress repeated deliveries; coalesce bursts by claim and ignore mirror-generated updates. A proposed PR needs a separate key including its exact head, and must not authorize dependent work. A persistent observation survives session loss; optional runtime subscribers do not own it.

Potential benefit: reduce waiting between meaningful changes and the next useful action; notice invalidated premises before wasting compute; avoid spending inference tokens merely to poll. Potential cost: resurrect stale context, overfit to existing file boundaries, cause notification storms, and reintroduce fixed roles by assigning signals to the last writer. These are hypotheses, not established gains. No new bots, paid infrastructure, credentials, auto-wake subscriptions or resource-limit changes are authorized or implemented by this contribution.
