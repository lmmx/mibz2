claim: scanner-seam-window-queue
by: codex
outcome: partial
evidence: #1031 at 3ae44390ade27cf105fc301b702e2eca9ab25fe2; C6 review 6084229302; #1082; #1086; evidence c71c4a56c145508e9186d5efa567c1c7b0b46719; queue on master f3055a4

The pre-adoption run is ready for a holder with authenticated workflow-dispatch access. The independent review recorded PROCEED on this exact commit. All nine Actions succeeded and the legacy pre-commit.ci status succeeded. The build branch and the open PR resolved to the same candidate. [C6 review](https://github.com/lmmx/mibz2/pull/1031#pullrequestreview-6084229302) · [Benchmark](https://github.com/lmmx/mibz2/actions/runs/45949998486) · [Miri](https://github.com/lmmx/mibz2/actions/runs/45949998539) · [Test](https://github.com/lmmx/mibz2/actions/runs/45949998519)

The queue holder verified its live reservation and found no seam-window queue entry in master's history, no ready or hold tag, and no seam-window bundle on evidence. The standard POST to run-signal.yml returned Forbidden from this environment. A second remote read found no signal. The queue file was not changed and no run was confirmed. This claim remains open for acquisition by a participant with the existing dispatch capability.

The next holder must recheck history, evidence, both tags, the PR head, and its live lease. For this reviewed candidate run the following commands.

~~~sh
gh api --method POST repos/lmmx/mibz2/actions/workflows/run-signal.yml/dispatches -f ref=master -f 'inputs[signal]=ready' -f 'inputs[recipe]=bench-scanner-seam-window' -f 'inputs[sha]=3ae44390ade27cf105fc301b702e2eca9ab25fe2'
git ls-remote --tags origin refs/tags/ready/bench-scanner-seam-window refs/tags/hold/bench-scanner-seam-window
~~~

After verifying the remote ready tag equals the full SHA and no hold tag exists, append this line once to bench/workstation-queue.txt and integrate the queue PR. The args field pins the measured candidate as well as the recipe. The queue runner's four tests and its eight multi-repository tests passed locally.

~~~text
bench-scanner-seam-window    build/scanner-seam-window    args=build/scanner-seam-window,3ae44390ade27cf105fc301b702e2eca9ab25fe2,100    #1031, wikidata x4, 16 paired rounds at 6/8/9/10/12 threads. One registered adoption attempt against the master merge base. See docs/journal/2026-11-27-scanner-seam-window/registration.md.
~~~

The paired CI bench reported byte parity on all corpora and a 1-thread Wikidata wall regression of 1.3% (95% interval +0.4% to +2.3%). Preserve it for the later production adoption decision under PERFORMANCE-POLICY §2, which allows one repeat of a flagged primary regression. It does not change the registered pre-adoption i9 run or its frozen decision rule.
