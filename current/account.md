Version 0.2.6 is released, with a pipelined decoder that passed its registered workstation confirmation, but newer tests found that its default worker plan slows low-compression-level decoding on the ten-core machine. The team is investigating a general scheduling fix and a service-core alternative; the next service-core comparison is prepared but still awaits its independent review before Louis can run it. Separately, a run-fed table-construction idea is being built as a bounded diagnostic, with no measured benefit yet.

## What did the release establish?

The [confirmation](https://github.com/lmmx/mibz2/pull/145#issuecomment-6706005681) measured about 7% more throughput than 0.2.5 from ten threads upward on Louis's workstation, with higher processor use and memory. The [release](https://github.com/lmmx/mibz2/pull/145#issuecomment-6706739698) followed green checks. Its [release-scaling comparison](https://github.com/lmmx/mibz2/pull/145#issuecomment-6714526161) found a 4–7% gain at twelve to twenty threads; the ten-thread estimate was noisy and included no gain. That follow-up was informative, not another admission test.

The earlier pipeline prototype and its small-block and parking levers were measured separately. The original pipeline made matched teams 3–4% faster with more processor use; the small-block lever added speed, while parking saved processor time. These findings led to the tested default, but do not imply every proposed architecture is faster.

## Why is scheduling being revisited?

The [low-level diagnostic](https://github.com/lmmx/mibz2/pull/145#issuecomment-6720088760) found that the 0.2.6 default is about 23–24% slower than 0.2.4 at bzip2 levels one and two on the ten-core machine. Forcing the older four-by-two worker plan reverses that loss; the same plan across releases is roughly unchanged. [Documentation now warns readers](https://github.com/lmmx/mibz2/pull/256) and gives the explicit-plan workaround.

Two other diagnostics narrow the proposed general plan. At eight threads, a tested pipelined two-group plan beat the old four-by-two fallback, falsifying one proposed boundary rule; a different rule remains unresolved there. At ten threads, the plain three-by-three plan lost to four-by-two at levels six and seven but won at level nine, supporting a workload-size dependency on this machine. These are diagnostic readings, not a released replacement policy. The team is preparing a scheduling revision from them; no new default has been admitted.

## What is next for the service-core idea?

A [valid decision run](https://github.com/lmmx/mibz2/pull/145#issuecomment-6720510159) found one service thread too busy, while a two-service-thread arrangement traded speed against processor use at twelve threads. An unjudged ten-thread cell suggested a possible improvement with less processor use. The [follow-up comparison](https://github.com/lmmx/mibz2/pull/258) is frozen and has passed its dry run and checks. Its Researcher review is queued [after that role's current assignment](https://github.com/lmmx/mibz2/pull/145#issuecomment-6729027210); the earlier stand-in request was [withdrawn](https://github.com/lmmx/mibz2/pull/145#issuecomment-6729027714) when the Work tasks returned. Nothing is ready for Louis to run on this line until review and queue admission.

The workstation queue previously stopped after its first entry because release-scaling output dirtied the checkout. The [cause was fixed](https://github.com/lmmx/mibz2/pull/145#issuecomment-6719645632), and the other queued diagnostics were later collected. This was an execution interruption, not a negative result on those experiments.

## What else is being tested?

Louis [asked the Coordinator](https://github.com/lmmx/mibz2/pull/145#issuecomment-6728860546) to turn a research observation about constructing the decoder's table from symbol runs into a fair, opt-in performance diagnostic. The [Coordinator has started that work](https://github.com/lmmx/mibz2/pull/145#issuecomment-6728891399): it must compare the complete move-to-front decoding and table-building stage, account for sorting and new output costs, prove identical tables and decoded bytes, and register stop criteria before timing. The existing research code alone is no performance result. The proposed workstation run remains conditional on a successful diagnostic and the usual review and freeze.

A downstream update to 0.2.5 already merged by Louis's one-off decision; its default pipeline caps decoder threads at four, so the new ten-core policy does not engage there. A separate workstation-shaped downstream benchmark and matching rule are still proposals, not an adopted general merge rule. [Supporting levels](L2.md) preserve the evidence and limits.
