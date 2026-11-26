# The zero-copy floor: what's left when every part is free

**The run:** `zero-copy-floor/run-20261126T112751Z`, recipe `bench-zero-copy-floor` (#917 at 1e27e46f; the zero-copy oracles are #921). It is evidence cc8ba137, SHA-256 10df0d89…5e34, checked.
- **Size:** 648 decodes in 16 minutes, on the i9-10900K.
- **Valid:**
  - every decode was byte-identical;
  - the scanner's pin held in 216 of 216 traces;
  - the removed parts' busy time was at most 0.15%;
  - the spelling had no calls in any F arm;
  - the memory controller's counters were live.
- **The rule:** `../../registration-floor.md` on #917, frozen before the data. The report and the table are in this folder.

The letters are the parts: S the scan, E emission, C construction, M MTF/RLE2, R RLE1/CRC/output, D delivery. F is emission with its spelling removed too (E_full). An arm's name lists its ideal parts.

## 1. The answer to the question it was built for

**The plumbing is negligible.** With every part free (spelling included) and delivery discarding its bytes, the decode takes **5.2–5.7 ms at every worker count**: Residual B, SFCMRD. That is 2.7% of the shipped 195 ms at 8 workers.
- **The queues, hand-offs and locks between the parts don't need attention.**
- **The design analysis's prediction** (14–19 ms at 1 worker) is scored a MISS: the plumbing is smaller than predicted.

## 2. What each part costs when it alone is real (8 workers)

Each wall is from an arm where every other part is free (the report's medians). Against the shipped wall of 194.6 ms:

| real part (all else free) | wall at 8 workers | at 1 worker |
|---|---|---|
| the scan (FCMRD) | **179.1 ms** | 169.7 ms |
| emission, walk and spelling (SCMRD) | 98.1 ms | 694.2 ms |
| construction (SFMRD; its store read cold, see §5) | 83.6 ms | 609.5 ms |
| delivery (SFCMR) | 52.3 ms | 54.7 ms |
| MTF/RLE2 (SFCRD) | 20.1 ms | 141.1 ms |
| RLE1/CRC/output (SECMD against SECMRD) | about 14 ms | about 66 ms |
| nothing (SFCMRD) | 5.3 ms | 5.8 ms |

**The scan alone takes 179 ms at 8 workers,** 92% of the shipped wall, with every worker-side part free. So at 8 workers, no improvement to the workers' parts, even all of them together, can bring the wall below about 179 ms while the scan stays a serial station. That is at most 8%. The scan must leave its serial station before worker-side gains can show at 8. This sharpens the matrix's S × C result and confirms the pooled scan (`../../../pooled-scan/registration.md`) as the first build.

## 3. Two costs the matrix couldn't see

- **Emission's spelling of words into bytes** (`E_consume`, Residual A − B) is **298 ms at 1 worker** (22% of the 1,343 ms wall), and 37 ms at 8.
  - The matrix's emission oracle left it in place, so the matrix never measured it.
  - With every part free except the spelling, it is 87–93% of what remains: SECMRD's 304 ms at 1 worker and 41 ms at 8.
  - It is a worker-side cost that scales with workers. It matters below the knee, and above it once the scan is pooled.
- **Delivery's copy-out is about 48 ms at every worker count** (47.3–49.3 ms, SFCMR − SFCMRD). The in-order reader copies 839 MB to the caller on one thread, about 17 GB/s. It doesn't scale with workers. Once the scan is pooled and the workers are fast, it becomes the next serial floor.

## 4. The third station (memory): not supported

The all-ideal wall with the spelling kept (Residual A, SECMRD) doesn't rise from 8 to 10 workers: 42.4 [40.3, 44.4] → 42.9 → 40.5 [37.6, 43.5] ms. With the oracles' copies gone, nothing here shows memory binding above 8. The matrix's rise from 8 to 10 (112 → 142 ms) was the old oracles' copying. Memory traffic stays far below the machine's bandwidth: 13–17 GB/s in the all-ideal arms, 15 GB/s as shipped at 10. The design analysis's three-station model isn't needed for this, and 9 workers is judged alongside 8 in the pooled-scan kill test (its §5).

## 5. Caveats

- **Construction's room is measured with a cold store.** The zero-copy borrow leaves the walk (A7) reading construction's store cold: 98.9 ms per worker at 8 in SCMRD, against 72.4 ms as shipped. Construction's row in §2 overstates its cost by roughly that much.
- **One workload, one machine:** wikidata ×4 on the i9, the scanner pinned, pre-spun.
- **Diagnostic (C1a):** it decides no production default.

## 6. What it changes

- **Characterisation is closed** (`../../../plan.md` thread 1).
- **The pooled scan's §5 readings:**
  - Residual A doesn't rise from 8 to 10, so 8 and 9 are both judged;
  - Residual B is 2.7% of the wall (under 10%), so the capture is not uncapped;
  - the oracles are valid, so the prototype can be built.
- **Two new targets join the plan:**
  - the spelling of words into bytes: 298 ms at 1 worker, a representation question for emission;
  - delivery's 48 ms serial copy-out: an API and ownership question.
