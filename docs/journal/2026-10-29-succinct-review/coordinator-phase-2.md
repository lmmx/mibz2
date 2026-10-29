<!--
Provenance: the coordinator's response to the Phase II agenda, 29 October 2026, relayed by
the user into the working session to be recorded. "The coordinator" is the chatbot earlier
called the guardian chatbot, now acting as research coordinator and experimental
gatekeeper. The text below is unedited. The user retains final authority over what is
implemented, benchmarked or merged.
-->

Agreed. I'll act as the research coordinator and experimental gatekeeper, while the succinct research chatbot pushes the theory and Claude Code handles implementation and measurement.

The division of responsibilities is clear:

- Research chatbot: Explore ambitious ideas, challenge assumptions, study prior art and develop new representations. Don't constrain its theoretical ambition.
- Claude Code: Validate proposals against the repository, construct controlled experiments, inspect machine code and record results.
- Me: Evaluate the evidence, decide which experiments deserve priority, prevent duplication and scope creep, and recommend when Claude should proceed or stop.
- You: Retain final authority over what gets implemented, benchmarked or merged.

## Our current priorities

| Order    | Work                                             | Decision                                                          |
| -------- | ------------------------------------------------ | ----------------------------------------------------------------- |
| 1        | #81 cooperative-build experiment                 | Proceed next                                                      |
| 2        | #79 F-window audit and matched traversal control | Proceed after #81                                                 |
| 3        | 16-bit fixed-line Ψ hybrid                       | Elevated research candidate; validate layout and dependency costs |
| 4        | Sub-16-bit translation-first architecture        | Theoretical exploration only                                      |
| Deferred | MTF register-residency experiment                | Preserve independently                                            |

The new census—98.11% FOR eligibility, 77.6% translation eligibility and a modeled 16.12 bits per row—gives us a strong reason to investigate the hybrid. It does not justify skipping the simpler controls.

### The rule I'll enforce

Every proposed experiment should answer a question that existing results cannot already answer. Before Claude implements it, we should have a falsifiable hypothesis, a matched control, explicit acceptance criteria and a clear stopping point.

Negative results stay negative. We don't retroactively change thresholds or keep tuning until something wins.

And we'll distinguish three levels of evidence throughout: mathematically possible, empirically representable, and demonstrably faster.

One practical limitation: I can coordinate the decisions and instructions through this conversation, but I cannot independently direct Claude's separate session or schedule its work. I'll give you concise, actionable handoffs to relay.

For now, my decision is to complete #81's cooperative test before authorising any new representation prototype. Meanwhile, the research chatbot has room to explore the mathematics as aggressively as it likes.

This gives us a productive balance: unrestricted theoretical curiosity, disciplined experimental execution.
