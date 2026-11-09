# 1. Scaling and plan policy

- **Doing:** revise the general worker-plan policy against recent diagnostics, in order to serve 0.
- **Status:** research and preparation; no new default qualified.

## Completed pipeline and release

The revised knee journal [accepted seven Researcher objections](https://github.com/lmmx/mibz2/pull/145#issuecomment-6699649478): serial-time attribution, memory saturation, buffer labels, predicted gain and falsifiers were narrowed. H1's predicted gain could disappear under plausible added table contention. The first registered pipeline run was valid: matched teams improved by about 3–4%, with 8–11% more processor use ([result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6702315743)). Its parking and small-block levers gave separate effects; the combined arm was reported, not qualified by that lever run ([result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6705082379)).

A subsequent registered confirmation of the intended default passed: 1.072 (95% interval 1.054–1.089) at ten threads versus 0.2.5, and roughly 1.07 at eleven through twenty ([LOG](https://github.com/lmmx/mibz2/pull/145#issuecomment-6706005681)). The Coordinator admitted and [released 0.2.6](https://github.com/lmmx/mibz2/pull/145#issuecomment-6706739698). Informative release scaling against 0.2.5 recorded 1.037 (0.984–1.093) at ten, 1.046–1.065 at twelve through twenty, with higher CPU and memory ([LOG](https://github.com/lmmx/mibz2/pull/145#issuecomment-6714526161)). It does not rewrite the registered confirmation.

## General plan: three diagnostics, no adoption

- **Boundary:** the eight-thread rule A fallback was falsified; a pipelined two-by-four plan returned 1.079 (1.055–1.102) against four-by-two. Rule B was unresolved at eight threads. Nine-thread pipelined three-by-three gained 1.141 (1.119–1.163) ([result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6719903152)).
- **Bytes:** plain three-by-three versus four-by-two returned 0.916 at level six, 0.957 at seven and 1.087 at nine on the i9. This supports the local bytes-per-team premise without licensing a rule for other cache sizes ([result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6719949774)).
- **Levels:** at levels one and two, 0.2.6's default returned about 0.76–0.78 against 0.2.4; forcing four-by-two restored about 1.30–1.32 against the default, while same-plan comparisons were roughly 1.008. The registered reading is a decoder-count scheduling slowdown, with the crossover between levels two and three ([result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6720088760)).
- **Next:** register and test a revised scheduling rule; the three diagnostics cannot qualify it by themselves. The README/CHANGELOG workaround is merged as #256.

Earlier team-size, policy, mechanism and control evidence remains in the [previous settled detail](https://github.com/lmmx/mibz2/blob/87b7cd8b0d67e570e62c8b8b97d291ccd36d3ee8/current/L3/1-scaling-past-8-threads.md).
