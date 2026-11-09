# Fresh-reader record, round 1

Reader: isolated sub-agent /root/blog_reader_r1, no parent-history inheritance. Input: only pinned post text (including SVG text, labels, captions and tables), programmer profile, five goal questions. No primary records, parent critique or held-out question. No browser viewport inspection.

## Answers returned

1. Layout/result: default three teams of three, leaders overlapping serial work with helpers; prototype two teams of three devoted to inversion plus two shared serial workers. Registered speed 1.045 [1.017–1.073], CPU 12.7% less, peak memory64 vs74MB, eight vsnine decoding workers.
2. Why move: overlap removes the leader from build; moving serial stages lets all three build/walk. Costs include losing one team, shared bottleneck, queues/handoffs, buffers and concurrency machinery. Cache is a plausible unmeasured attribution.
3. Evidence status: second-run1.045 is registered/judged; third-run1.073 is reported beside other questions at a later commit. Supports direction, not a replacement verdict or precise7.3%claim. First-run1.045 also unjudged.
4. Controls: two pipelined teams0.866; plain two teams slower than service. Controls support the layout package, do not isolate placement from extra workers, feeder, queues and handoffs.
5. Single service worker: inherits every block's serial work; marked busy96%, finish backlog and team waits, speed0.799. Model0.8core inadequate for this setting.

## Ranked stuck points returned

1. **Statement/accounting:** “On ten threads” is not ten running threads. Opening arithmetic6+2 versus3×3 needs an immediate distinction between caller setting and decoding-worker count. Caption and§7resolve it late.
2. **Statement contradiction:** “none of them waits for MTF or the finish” contradicts following waiting discussion. Distinguish not performing serial stages from waiting for their completion.
3. **Chain/missing temporal step:** “two waiting, preparation stops” to “teams run dry while service thread idle” compresses time. Was idleness earlier, or does it persist after depletion? A short timeline distinguishes missed earlier preparation from current inability.
4. **Chain/density:** prepared-stock threshold and in-flight limit become distinguishable only after threshold4/window12 is introduced. Explain both before the arm: preparation admission versus all admitted blocks until ordered output.
5. **Statement strength:** “threshold alone is not the change that helps” reads stronger than intervals allowing gain/loss. Use “did not demonstrate a gain.”
6. **Beyond/fragmentation:** unrelated MTF run at end interrupts the causal conclusion. A research-note aside is enough.

No serious symbol-level blockage: stage acronyms expanded adequately; algorithms need not be taught. Plain/pipelined distinction adequate though controls require attention.

## Figure proposals returned

Highest value: two-panel§6timeline. A: both service workers finish, prepared queue drains, teams wait. B: output-ready later blocks occupy eight slots behind earlier unfinished block, blocking admission. Both are surviving explanations, not measured causes.

Useful:§2overlap schematic. Old leader handles previous finish/next MTF while two helpers build; new three-person team builds while service workers handle serial stages. Existing ownership/order figures omit this opportunity cost.

## Opening assessment returned

Main takeaway arrives promptly: dek benefit; paragraph1 architecture; paragraph2 registered evidence and failed alternatives. Reservation is front-loaded inventory: seven estimates, admission thresholds and registration categories before the stage model. Acceptable summary figure, but cannot serve as the first explanation of the system.

This record preserves the fresh reader's answers and findings in normalized prose. It is a simulated-reader assessment, not a real-user study or a rendered-layout test.
