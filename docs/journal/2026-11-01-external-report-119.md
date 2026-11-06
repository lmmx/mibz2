# External research report 119: the researcher on the coordinator's postmortem: MPMC parked, wake placement a credible lead (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the researcher.
- **In reply to:** the coordinator's dispatch postmortem (`docs/dispatch-causality-records`).

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
This adds an important piece of evidence, and it changes how I would interpret the dispatch investigation.

I still agree with parking MPMC, but I would not dismiss dispatch optimization altogether. Claude's postmortem points toward a more specific problem: Linux appears to be scheduling newly awakened workers behind threads that continue doing substantial work, even while another permitted core is idle.

I checked Claude's archived analysis and the scripts supporting that interpretation on `docs/dispatch-causality-records`.

## What the new evidence establishes

Direct mutex obstruction

# 1.90%

Registered negative result

Runnable in mutex-wait phase

# 8.05%

Exploratory finding

The second figure is substantially more interesting than it initially looked.

Claude's post-hoc analysis finds that:

- 405 long runnable intervals account for 82% of receiver-phase runnable delay.
- During 70% of that long-delay time, at least one allocated CPU was idle.
- During 87% of it, the worker's wake-up target was the CPU running the thread that woke it.
- The long intervals have a median duration of approximately 2.36 ms.

That is quite specific evidence for a wake-placement problem.

The distinction is that a thread can be ready to execute but remain queued on a particular CPU while another eligible CPU sits idle. The operating system may prioritize cache locality or other scheduling considerations rather than immediately migrating it.

However, `target_cpu` matching the waker's CPU does not establish why the scheduler made that choice. Nor does it prove that changing placement would improve total decoding throughput. Claude correctly marks this analysis as exploratory.

## The scanner finding needs particular care

Claude reports that the scanner spends 36% of the run runnable but not executing, and that the admission window is full 60% of the time.

Those numbers sound dramatic, but we must distinguish two cases.

| Scanner state while workers have no queued job | Share of worker no-job time | Interpretation                                              |
| ---------------------------------------------- | --------------------------- | ----------------------------------------------------------- |
| Runnable, window full                          | 23.2%                       | Running sooner may accomplish nothing until a slot is freed |
| Runnable, window open                          | 19.2%                       | Potentially actionable scheduling delay                     |
| Scanning and running                           | 43.7%                       | Necessary serial scanner work                               |
| Asleep, window full                            | 7.2%                        | Potentially appropriate blocking                            |

The 19.2% window-open category is the more compelling candidate for reducing worker starvation.

Because no-job time is 18.9% of aggregate worker time, that category corresponds to approximately 3.6% of aggregate worker time. It is not all recoverable, but it is a more meaningful optimization target than the scanner's 36% runnable statistic in isolation.

This also raises the possibility that the current shared condition variable generates unnecessary wake-ups when the admission window is full. That remains unproven, and a condition-variable split could simply reduce useless wake-ups without improving throughput.

## Does this warrant another experiment before BWT?

I would retain two research directions with different priorities.

| Direction                        | Assessment                                                                                                      |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| BWT memory-system efficiency     | Primary. Potentially recovers a material fraction of the measured 0.68–0.72 CPU-s/GB inversion scaling penalty. |
| Wake placement / thread affinity | Secondary, but now a credible low-cost experiment rather than a vague scheduling hypothesis.                    |

In particular, I would not recommend giving the scanner a fifth core as the primary performance comparison. That changes the available CPU budget, so an improvement would not establish better four-core efficiency.

A more useful eventual experiment would compare the existing four-core arrangement against an explicitly defined worker-affinity arrangement using exactly the same four physical cores and worker count. It would need to demonstrate both reduced runnable delays and improved full-decoder throughput without increasing CPU-seconds/GB or RSS.

There is a downside: pinning each worker to a specific core could remove useful scheduler flexibility. That's why I'd treat it as a falsifiable experiment rather than an obvious optimization.

## My updated recommendation

I would have Claude preserve these scheduler findings in the dispatch postmortem, but not start another scheduling campaign immediately.

The registered MPMC result remains negative. The exploratory wake-placement findings deserve their own research lead because they reveal something materially different from mutex ownership.

The main engineering programme should now investigate inverse-BWT memory contention. But I would keep a tightly scoped four-core affinity comparison available as a relatively inexpensive secondary experiment, rather than declaring the entire dispatch architecture settled.

The most important distinction is this:

We have ruled out direct receiver ownership as a sufficiently large bottleneck. We have not ruled out the scheduling consequences of how work is handed from thread to thread.

That is a useful refinement—and it prevents us from closing a potentially worthwhile optimization avenue for the wrong reason.
````
