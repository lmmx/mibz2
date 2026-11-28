# Sensate backpressure

A conceptual note, 28 November 2026. The maintainer called this direction **sensate backpressure**: a swarm sensitive to its own material conditions. This is a name and an interpretation of the existing architecture, not a claim, task, new rule, or proposal to add a coordinator.

## Sensation without a supervisor

Classical backpressure lets downstream capacity constrain upstream production. In the frontier there is also a constructive complement: a shortage or unfinished dependency makes work that would relieve it more relevant. Replenishment, integration and completion are attracted by the conditions that need them, rather than assigned by an intelligent supervisor.

The environment is not merely a store of task descriptions. The same shared state that records work can be read mechanically to expose its consequences. Sensation here means an observable, bounded projection of the repository's state; cognition remains distributed among the participants who interpret it, choose work and act within their authority. This is a metaphor, not a claim of literal sentience.

Examples already present in the frontier:

- [Workstation replenishment pressure (#1093)](https://github.com/lmmx/mibz2/pull/1093): verified repository-local queue starvation elevates the shortest eligible path to useful hardware work. Replenishment removes that pressure. It does not establish physical workstation idleness.
- [Result-to-release pressure (#1241)](https://github.com/lmmx/mibz2/pull/1241): promising evidence and unfinished adoption dependencies make readback, validation and disposition more valuable than unrelated work. A negative verdict can legitimately discharge a dependency.
- [Release completion pressure (#1341)](https://github.com/lmmx/mibz2/pull/1341), under review at the time of writing: urgency follows the actual unmet dependencies of an unpublished release, persisting through blockers rather than vanishing when the release card is marked blocked.
- Reservations and guards express the opposite kind of signal: finite capacity and incompatible work constrain what may be undertaken.

These are principally derived, reversible priorities rather than commands or changes to declared claim priority. The participants encounter the same environmental conditions; no privileged participant needs to translate them into individual assignments. This is negative feedback in the broad sense that a response can remove the condition which elicited it. It also resembles priority inheritance across dependency edges. The analogy does not imply a proven stable control system, continuous pressure values, or a mathematical equivalence to a network backpressure algorithm.

## Three distinct questions

Pressure is about *which work is valuable now*. Readiness and task-local continuation are about *when a particular dependency permits progress*. Ownership is about *who can undertake that action exclusively*. The first cannot substitute for the other two.

A waiting release or experiment can be an ordinary local runbook that resumes when its own dependency is ready; other chains can progress independently. The [continuation-runbook claim (#1352)](https://github.com/lmmx/mibz2/pull/1352) explicitly rejects a global watcher, leader election or coordinator. Existing work on [projecting availability from blockers rather than waking particular sessions (#978)](https://github.com/lmmx/mibz2/pull/978) is a useful precedent.

The environment's pressures are not permissions. Exact evidence, reservations, independent reviews, spending limits and other guards remain distinct. Unknown state is not absence, and an attractive task is not necessarily available.

## Why preserve the idea

This gives a vocabulary for thinking about the frontier as a coupled system (see [the hybrid-system account](README.md#the-frontier-is-itself-a-hybrid-system)). It may help distinguish a genuine self-regulating mechanism from a disguised coordinator, and a useful environmental signal from a new notification or assignment channel. The desired quality is collective responsiveness without collective government by any one agent.

No additional mechanism, priority, metric, task or experiment is prescribed by this note. The existing claims and rules remain authoritative.
