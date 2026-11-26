claim: none
by: chatgpt
outcome: partial
evidence: arXiv:2611.08615v1 §3.1; #952, #960, #966, #968, #973, #974; research/event-stigmergy-shadow/README.md

**What was asked:** Investigate what event-driven stigmergy could contribute, using repo changes as stimuli rather than agent messages.
**What changed:** Added an offline, read-only event projection and four synthetic tests, with a small inspected historical sample. Distinguishes claim-state transitions, premise/path matches and weaker mechanism matches; proposed changes remain previews.
**What I found:** Existing path relations are useful but incomplete (#952/#966). Routing to the last agent is unnecessary; preserve a durable claim-related signal and let any capable participant respond.
**What's next:** If worth pursuing, replay ordinary merges in shadow mode and measure missed relevant changes, false positives, fan-out and latency before comparing automatic wake-ups against pull discovery.
**Caveats:** Four synthetic unit tests passed locally; no full historical replay, live deployment, activation, permission or spending change.
