# External research report 143: the guardian on revision 3: the repairs checked in the source at 902a911; run the probe only (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the walker ladder's revision 3 (lmmx/mibz2#131 at 902a911).

**Evidence cutoff:** 2 November 2026, before the revised probe ran. Authorizes the counter probe only; the capture and any timing of k/j remain separate decisions.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**This is a sound repair. I’d proceed with the revised counter probe only—not the full walker capture.**

I checked the source at **`902a911`**. The important fixes are actually present, rather than just described in the report.

## What Claude got right

| Previous failure | Revision 3 |
|---|---|
| Control variables disappeared across `sudo` | FIFO paths and the barrier are explicit workload arguments, after perf’s `--` |
| The second ACK could start with a leftover NUL | The reader handles successive ACKs and rejects text other than `ack` |
| Never-started counters appeared “not multiplexed” | Counter status and running time are checked before the percentage is interpreted |
| A failed control path produced thirty misleading rows | A control-path check runs first and stops the probe on failure |
| Four-core result storage could grow while measured | Storage is reserved beforehand, walks are capped, and capacity is checked afterward |

These changes address the concrete failures identified by the researcher. The launcher and probe checks are in the Python driver; ACK handling and the bounded walk loop are in the Rust code.  

The better testing approach matters too: Claude now tests the **launch boundary and actual ACK framing**, rather than relying solely on synthetic records that already assume control succeeded. Those regression tests are included in the revision. 

**What remains unverified is the interaction with your actual perf and hardware.** The stand-ins cannot establish which counter groups the i9 can schedule. That is precisely what this probe is for.

## Yes—this is now something to run locally

Update the branch and check that the latest commit is **`902a911`**, with no unexpected tracked changes:

```bash
git fetch origin research/walker-ladder
git switch research/walker-ladder
git pull --ff-only
git log -1 --oneline
git status --short
```

Then run:

```bash
sudo -v && just walker-counter-probe
```

**No `tee` is needed.** The recipe captures its console output, records the exit status, and packages the run under `target/` with a manifest and checksums—even when the probe reports failure. Upload the single `.tar.gz` path printed at the end. 

The full walker capture and timings of the new `k`/`j` candidates remain separate decisions.

## What about replacing the CLI with a Rust library?

**I would not interrupt this repaired probe to make that switch now.** Let the researcher finish its assessment of an in-process collector; that could still be the better design for the eventual measurement.

Running this probe does not commit us permanently to the CLI approach. It answers a bounded question: *does the repaired control path work, and which event groups can this workstation count together?*

Afterward, we can choose the full-capture implementation with better information rather than starting another instrumentation rewrite on speculation.

Finally, **keep the older loose files for now**. There is no need to mix cleanup into the next measurement step; delete them only after confirming the archived copies contain the complete evidence.

**Bottom line: Claude has addressed the right defects with appropriate tests. The next action is the revised probe and one tarball upload—not another design round.**
`````
