# D020 reconciliation

Trigger: [6644051935](https://github.com/lmmx/mibz2/pull/145#issuecomment-6644051935).
Read [D020 at 8d6a716d](https://github.com/lmmx/mibz2/blob/8d6a716d0dbf75bc5c6a2ff3e4a652f6e7df1662/docs/journal/compact30-guardian-shadow/decisions.md#d020--b2-provenance-resolved-claimed-validation-mixes-runs-and-kernels).

The independently prepared [per-copy assessment](copy-predictions-6643992749/report.md) reaches the same run, kernel and identifiability corrections. It directly read both the current benchmark table and 75b13 source; the earlier table was retained from the prior Researcher inspection. No raw timing-round replay is claimed.

The useful additions for the requested predictions are:

- A common latency increase does not change B/A in the uncapped window regime when issue and window work are both proportional to U. Loss of gain needs a changed limiting regime, asymmetric penalty, or other changed cost.
- A separate-core pair and an SMT-sibling pair must be separate cells. Four workers on two cores is not a substitute for the two-worker SMT cell.
- The current phase harness times elapsed seconds with Instant. Under concurrency, off-CPU wait can erase an apparent elapsed-stage gain without identifying a hardware shared limit. Pair elapsed and CPU boundaries before making that inference.
- A stable isolated-walk ratio does not locate the production critical path; altered overlap can change the walk's environment in the real decoder.

D020's comparison corrections are supported; no new disagreement on those findings. This note neither authorises nor executes instrumentation, changes a gate, nor adopts any broader authority inferred from GitHub comments.
