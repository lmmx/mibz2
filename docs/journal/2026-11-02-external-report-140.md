# External research report 140: the researcher on the failed walker probe: sudo strips the control FIFO's variables, and perf's ACK carries a trailing NUL (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the new independent researcher session (as in report 138).
- **In reply to:** the walker counter probe of 2026-11-02 00:06 UTC (lmmx/mibz2#131 at a245a45), whose JSON it inspected.

**Evidence cutoff:** 2 November 2026, after the failed probe. Source inspection and small transport and parser reproductions on its own machine: no timed capture, repository change or workstation-setting change.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
\*\*I found a concrete launcher bug that explains the all-false result, and a second FIFO compatibility bug that should be fixed before another probe.\*\* The immediate failure occurred before counter collection: the walker never initialized its control connection, so perf’s counters remained disabled.\
\
The guardian’s diagnosis was right. We can now identify the likely cause much more precisely.\
\
\## 1. What the failed probe actually recorded\
\
I inspected walker-probe-20261102T000606Z.json[the probe JSON]\(sandbox:/workspace/scratch/eb4b5d8c4a8d/upload/walker-probe-20261102T000606Z.json), including all the raw perf output.\
\
\| Observation | Result |\
\|---|---:|\
\| Candidate configurations | 30 |\
\| Perf invocations, including control-only runs | 60 |\
\| Raw outputs containing \`Events disabled\` | 60 |\
\| Raw outputs containing \`Events enabled\` | \*\*0\*\* |\
\| Event-result lines reporting \`\<not counted>\` | \*\*210 out of 210\*\* |\
\| Event-result lines reporting \`\<not supported>\` | \*\*0\*\* |\
\| Event-result lines with zero counter running time | \*\*210 out of 210\*\* |\
\| Retained walker records reporting \`controlled: false\` | \*\*30 out of 30\*\* |\
\
Every raw output begins like this:\
\
\`\`\`text\
Events disabled\
\<not counted>,,cycles,0,100.00,,\
\<not counted>,,instructions,0,100.00,,\
\`\`\`\
\
The important combination is \*\*\`controlled: false\`, no enable message, and zero counter running time\*\*. This run provides no usable evidence about whether the proposed groups can be scheduled together.\
\
\## 2. The immediate bug: control variables cross the \`sudo\` boundary incorrectly\
\
I checked the exact revision you ran, \*\*\`a245a4550fc44d6d3f7884a6d966ed369a66acff\`\*\*.\
\
The launch sequence has an asymmetry:\
\
\- Perf receives its FIFO paths through \*\*command-line arguments\*\*.\
\- The walker receives the same paths through \*\*environment variables\*\*.\
\- Those environment variables have to survive \`sudo\`.\
\
Here is the causal chain.\
\
\### The recipe invokes \`sudo perf\`\
\
The [recipe at that revision]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/.just/bench.just#L223) selects:\
\
\`\`\`bash\
WALKER_PERF="sudo perf" python3 bench/walker_ladder.py probe "$f"\
\`\`\`\
\
The Python [\`command()\` function]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/bench/walker_ladder.py#L142) puts \`WALKER_CTL\` and \`WALKER_ACK\` into the environment supplied to that outer command. The resulting command has this structure:\
\
\`\`\`text\
sudo perf stat --control fifo:CTL,ACK -D -1 ... -- taskset ... walker_ladder ...\
\`\`\`\
\
Sudo’s default policy creates a new, minimal environment. Custom variables survive only when the policy permits them. Consequently, setting these variables in Python’s \`subprocess\` environment does not ensure that the walker receives them. [GitHub]\(https\://github.com/sudo-project/sudo/blob/main/docs/TROUBLESHOOTING.md?utm_source=chatgpt.com)\
\
\### The walker silently proceeds without control when either variable is missing\
\
In Rust, [\`Ctl::open()\`]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/examples/walker_ladder.rs#L414) returns \`None\` if either environment lookup fails. It does not treat that as an error, because running without perf is also a supported mode.\
\
The subsequent code sends \`enable\` and \`disable\` only when that controller exists. The final JSON’s \`controlled\` field is literally derived from \`ctl.is_some()\`. Thus, \*\*for this source revision, \`controlled: false\` means the walker never established the control client\*\*. An error opening a named FIFO would instead trigger the code’s \`expect\` failure. [Control commands and reporting]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/examples/walker_ladder.rs#L522).\
\
Perf, meanwhile, successfully receives \`-D -1\`, which tells it to start with events disabled. Without an enable command, it stays that way. That behavior is documented in the [perf 6.8.12 source documentation]\(https\://github.com/gregkh/linux/blob/v6.8.12/tools/perf/Documentation/perf-stat.txt#L225).\
\
\*\*My conclusion:\*\* sudo’s environment filtering is the strongly supported explanation for the missing control variables. The JSON does not include the child’s actual environment or your sudo policy, so it does not independently prove that last link. But the launcher has exactly the defect needed to produce every observed symptom.\
\
\### I reproduced that launch-boundary failure locally\
\
Using the actual Python command constructor, I substituted an inert child and a wrapper that removes \`WALKER\_\*\` variables. No hardware counters or walker timings were involved.\
\
\| Construction | FIFO variables reach child | Barrier variable reaches child |\
\|---|---|---|\
\| Current launcher through the filtering wrapper | No | No |\
\| Explicit child assignments after perf’s \`--\` | Yes | Yes |\
\
That verifies the mechanism and a repair approach without another workstation run.\
\
\## 3. Fixing the environment alone is likely to expose a second bug: ACK framing\
\
This is the additional finding I would make sure Claude sees \*\*before\*\* returning a corrected probe.\
\
Your JSON reports \*\*perf 6.8.12\*\*. In upstream source for that exact version:\
\
\- The ACK text is defined as \`"ack\n"\`.\
\- Perf writes \`sizeof\` that string literal.\
\- That means it sends \*\*five bytes\*\*, including the terminating NUL: \`ack\n\0\`.\
\
These are visible in [the ACK definition]\(https\://github.com/gregkh/linux/blob/v6.8.12/tools/perf/util/evlist.h#L404) and [the ACK writer]\(https\://github.com/gregkh/linux/blob/v6.8.12/tools/perf/util/evlist.c#L2051).\
\
The walker’s current [\`Ctl::cmd()\`]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/examples/walker_ladder.rs#L427) reads one newline-terminated string and checks that it starts with \`"ack"\`.\
\
A newline reader consumes through \`\n\`; it leaves the following NUL for the next read. That is the documented behavior of Rust’s \`read_line\`. [Rust]\(https\://dev-doc.rust-lang.org/stable/std/io/trait.BufRead.html?utm_source=chatgpt.com)\
\
Consequently, two successive acknowledgments behave as follows:\
\
\| Response | What the current line reader receives | Current check |\
\|---|---|---|\
\| Enable ACK | \`"ack\n"\` | Passes |\
\| Disable ACK | \`"\0ack\n"\` | \*\*Fails\*\* |\
\
I also reproduced this byte-stream behavior locally.\
\
\*\*This is a latent failure identified from source inspection.\*\* The uploaded run failed earlier, before sending either command. Once the environment handoff works, the current reader would reject the second ACK with the upstream 6.8.12 framing.\
\
The repair should tolerate the NUL framing while validating the ACK text. Its regression test should include \*\*two successive ACKs\*\*, both with ordinary \`ack\n\` framing and with perf’s \`ack\n\0\` framing. Testing only one acknowledgment would miss this.\
\
\## 4. Why \`multiplexed: False\` appeared beside completely invalid counts\
\
The \`100.00\` percentages are real text printed by perf, but they do not establish successful counting.\
\
I checked [perf 6.8.12’s CSV formatter]\(https\://github.com/gregkh/linux/blob/v6.8.12/tools/perf/util/stat-display.c#L77). It initializes the displayed percentage to 100 and recalculates it only when running time differs from enabled time.\
\
When both are zero, it therefore prints:\
\
\`\`\`text\
0,100.00\
\`\`\`\
\
Separately, perf marks zero-running-time results invalid and prints \`\<not counted>\`. That explains the apparently contradictory output.\
\
The Python parser correctly preserves \`\<not counted>\`. \*\*This is not principally a CSV-column parsing failure.\*\* The reporting problem is that [\`probe()\`]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/bench/walker_ladder.py#L268) computes its multiplexing Boolean from the percentage without first requiring a valid counted interval.\
\
For this result, the appropriate reporting is:\
\
\- \*\*Control path:\*\* failed.\
\- \*\*Counts:\*\* unavailable.\
\- \*\*Multiplexing:\*\* unassessed.\
\- \*\*Event-group availability:\*\* unestablished.\
\
A legitimate numeric count of zero must still be accepted when the event actually ran. That is different from \`\<not counted>\` with zero running time.\
\
\## 5. The smallest useful repair\
\
I would keep the repair focused on the launcher, ACK reader and probe validation.\
\
\### Pass the required settings explicitly to the workload\
\
A robust command structure is:\
\
\`\`\`text\
sudo perf stat ... --control fifo:CTL,ACK -D -1 ... --\
&#x20;   env WALKER_CTL=CTL WALKER_ACK=ACK\
&#x20;   taskset -c CPU target/release/examples/walker_ladder ...\
\`\`\`\
\
This is an illustration of the command constructor’s intended output, \*\*not another command for you to run\*\*. Python should construct it as an argument list.\
\
The required values then reach the workload after the sudo boundary. This needs no change to workstation policy.\
\
\*\*Include \`WALKER_BARRIER\` in the same repair.\*\* The current [\`four()\` implementation]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/bench/walker_ladder.py#L170) adds that variable to the pre-sudo environment too. Fixing only the FIFO variables would leave the four-worker synchronization path with the same defect.\
\
\### Validate the ACK protocol against the actual perf framing\
\
Handle both four-byte and five-byte ACK frames, including successive responses. A stand-in should emit successful counts only after receiving the expected enable/disable commands and acknowledging them.\
\
\### Make a failed control path an explicit probe failure\
\
The probe currently constructs a record but does not pass it through the shared validity function. It also retains the control-only run’s raw output and cycle count while discarding its structured \`controlled\`, status and running-percentage checks. [Probe implementation]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/bench/walker_ladder.py#L258).\
\
Both halves of each probe pair need validation. A good 20-walk run paired with a broken zero-walk control is still invalid evidence.\
\
A shared control failure should be recorded clearly, with its raw output preserved, before the probe stops. Repeating that failure across every candidate group provides no additional availability information.\
\
\## 6. Why the existing tests missed this\
\
The [current unit tests]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/bench/test_walker_ladder.py) primarily create synthetic records—defaulting to \`controlled=True\`—and exercise the validity rules. They do not exercise the actual launcher through an environment-filtering wrapper.\
\
Their uncounted CSV fixture also uses \`0.00%\`, whereas your real output contains the significant \`0,100.00\` combination.\
\
The archived stand-in results show that the tested arrangement worked. They do not establish coverage of sudo’s environment filtering or the exact ACK bytes emitted by perf 6.8.12.\
\
The focused regression set should therefore cover:\
\
1\. \*\*The actual command constructor through an environment-filtering wrapper\*\*, checking that the control paths reach the child.\
2\. \*\*The four-worker barrier setting through that same wrapper\*\*, using inert children.\
3\. \*\*Two consecutive ACKs with the real five-byte framing\*\*, plus the documented four-byte form.\
4\. \*\*Your exact failed CSV fixture\*\*, which must remain invalid despite \`100.00%\`.\
5\. \*\*A successful measured run paired with a failed control-only run\*\*, which must be rejected.\
\
All of these checks can run without PMU access or a timed walker capture.\
\
\## 7. One separate issue to retain for the eventual full-capture review\
\
While tracing the four-worker path, I found another measurement issue worth recording now.\
\
Python launches deadline-based workers with \`reps=1\`. Rust consequently reserves space for only two result entries, then repeatedly pushes results until the common deadline. Once that capacity is exceeded, A/B can grow their result vector inside the measured window; P does not use that result-push path. [Python launch]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/bench/walker_ladder.py#L178), [Rust reservation and loop]\(https\://github.com/lmmx/mibz2/blob/a245a4550fc44d6d3f7884a6d966ed369a66acff/examples/walker_ladder.rs#L470).\
\
That should be resolved before a full capture so that result-storage growth does not enter the bookkeeping comparison. It does not need to expand the immediate counter-probe repair.\
\
\*\*The next deliverable should be a corrected probe with the environment handoff and ACK framing covered by regression tests.\*\* Preserve this failed probe as evidence, leave the event groups unfrozen, and retain the existing authorization boundary. I ran only source inspection and small transport/parser reproductions; no timed walker capture, repository changes or workstation-setting changes were made.
`````
