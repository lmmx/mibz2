# Outline: The decoder's true floor and the limits of a pooled scan

*Blog card: `frontier/blog/floor-and-pooled-scan.toml` · Stage: `outline` · Prepared 26 November 2026.*
*Primary sources: `docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/` (`README.md`, `report.txt`, `zero-copy-floor.tsv`; `origin/evidence` commit `cc8ba137`, SHA-256 `10df0d89…5e34`), `docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md`, `docs/journal/2026-11-24-algorithm-level/pooled-scan/results/` (`README.md`, `report.txt`, `pooled-scan.tsv`, `phases.tsv`; `origin/evidence` commit `c71c4a56`, SHA-256 `97e397ad…8867`), `docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/README.md` (`origin/evidence` commit `7dd7b070`), `frontier/surface.tsv`, `frontier/dead-ends.tsv`, and the two preceding posts `docs/site/blog/2026-11-26-complementary-bottlenecks.md` and `docs/site/blog/2026-11-26-from-bottlenecks-to-better-algorithms.md`.*

---

## 1. Target claim, title, dek and takeaways first

### Target claim (one sentence)
Replacing copying oracles with zero-copy borrows proves that `mibz2`'s pipeline plumbing costs only 5.2–5.7 ms (2.7% of the 8-worker wall) while exposing a 179.1 ms serial scan ceiling and two hidden costs (298 ms of word spelling at 1 worker and 48 ms of flat serial delivery), and testing a pooled Huffman scan behind an idealised constructor shows why moving scanning work onto busy workers misses its pre-registered gate narrowly at 8 and 9 workers and wins only where workers have spare capacity.

### Title and dek candidates
The card's working title is *"What every part free leaves behind, and why pooling the scan wasn't enough"*. Before drafting, settle the final headline and matching URL slug (published URLs never redirect):

1. **Recommended:** *The decoder's true floor and the limits of a pooled scan* (11 words, 56 chars — fits the social card whole; proposed slug: `docs/site/blog/2026-11-26-true-floor-and-pooled-scan.md` or publish date)
2. **Alternative (closer to card):** *What every part free leaves behind, and why pooling the scan fell short* (13 words, 71 chars)
3. **Alternative (mechanism first):** *Zero-copy oracles and a pooled Huffman scan: measuring the floor and the serial wall* (14 words, 84 chars)

**Dek candidate** (checked with `docs/scripts/check_prose.py`: **PASS** — 1 sentence, 25 words, 1 break, 1 figure, Flesch reading ease 46, inside the 16–28 word / ≤ 1 break / ≤ 2 figure / 35–70 reading-ease band):
> Zero-copy stand-ins drop mibz2's plumbing floor to 5 milliseconds and uncover two hidden costs, while pooling the scan pays off only when workers have slack.

### Takeaways first (placed straight after the opening, before Section 1)
- **Pipeline plumbing is negligible.** With every stage replaced by zero-copy stand-ins and emission's word spelling removed too, decoding takes **5.2–5.7 ms** across 1–10 workers (**2.7%** of the shipped 194.6 ms wall at 8 workers), a third of the 14–19 ms forecast; queues, hand-offs and locks need no work.
- **Memory did not bind from 8 to 10 workers.** Removing the first matrix's 4.9 GB of oracle copying flattened the all-ideal wall (with spelling kept) to **42.4 → 42.9 → 40.5 ms** at 8, 9 and 10 workers (13–17 GB/s memory traffic). The earlier 112 → 142 ms rise was an experimental copying artifact, refuting the three-station memory hypothesis at these counts.
- **The serial scan caps worker-only gains at 8% at 8 workers, and the floor revealed two large hidden costs.** Keeping only the scan real takes **179.1 ms** at 8 workers (**92%** of the shipped wall). Beneath it sit emission's word spelling (**298 ms at 1 worker**, 22% of the 1,343 ms wall; **37 ms at 8**) and delivery's single-threaded copy-out (**48 ms flat** across 1–10 workers for 839 MB at ~17 GB/s), which becomes the next serial floor once scanning leaves its single thread.
- **The pooled scan missed its frozen gate narrowly, even though its plumbing worked cleanly.** All 10 exactness tests passed with 0 dropped blocks, 0 fallbacks and 0 degraded decodes; peak memory stayed within 1.4% (K3), the AVX2 marker finder took **4.82 ms** (9.84 GB/s, half of K4's 10 ms limit), and standalone cost passed K2. Yet behind ideal construction it captured **52%** of the scan's room at 8 workers and **48%** at 9, leaving its 95% upper bounds **0.009** and **0.042** above K1's half-capture bar.
- **Pooling redistributes scanning CPU instead of removing it, so it pays only from worker slack.** Workers spent the same **173–193 ms** of combined scan CPU as the serial scanner (**171–190 ms**). Loading that work onto saturated workers slowed standalone decoding below the knee (+10% at 6–7, +5% at 8), broke even at 9 (`1.000`), and won at 10 (**−6.4%**, `187.2` vs `197.6 ms`, **63%** capture behind ideal construction). The prototype stops as built; the next steps are shrinking worker-side costs first and testing a slack-only hybrid scan-ahead.

---

## 2. Intended reader, voice and hard constraints

- **Intended reader:** A systems programmer or performance researcher who understands threads, caches, memory bandwidth, queues and basic experimental confidence intervals, and may arrive either directly from [Complementary bottlenecks](2026-11-26-complementary-bottlenecks.md) or cold from a search link.
- **Cold-entry grounding:** Because this is a follow-up post, the opening must name and link both [Complementary bottlenecks](2026-11-26-complementary-bottlenecks.md) and [From bottlenecks to better algorithms](2026-11-26-from-bottlenecks-to-better-algorithms.md), and recap in plain words what the six pipeline parts (`S`, `M`, `C`, `E`, `R`, `D`), the worker-count knee (~7.7 workers; 8 shipped), and an *oracle* (a pre-recorded stand-in for a stage) are before using any of them.
- **Hard site & editorial constraints (checked at `review` and `ready`):**
  1. **Zero agent or model names anywhere in the post:** refer strictly to *"we"*, *"the design analysis"*, *"an independent review"*, and *"the prototype"*.
  2. **Mechanism names, not model or persona names:** name every component by what it does (zero-copy borrow oracle, AVX2 nibble filter for 48-bit block markers, paired-oracle kill test, slack-only hybrid scan-ahead).
  3. **Project voice, British spelling, plain statement:** serious researcher register; no slogans, no internal process theatre or chat quotes, no mannered prose; narrow every claim to what was measured (one workload, wikidata ×4, on one machine, the Intel i9-10900K).
  4. **KaTeX for multi-term equations:** two-station vs. pooled wall formulas $T_{\text{serial}}(k) \approx \max(s, w/k)$ and $T_{\text{pooled}}(k) \approx (s + w)/k + \epsilon$, break-even threshold $k^* = 1 + w/s$, and log-scale capture $\ln(C/PC) / \ln(C/SC)$ against $\sqrt{SC/C}$.
  5. **No redirects, ever:** finalise title and slug before first publication.
  6. **Independent review & `publish-page`:** `review` stage is done by a different reviewer checking every number against `results/floor/` and `pooled-scan/results/`; `ready` requires `publish-page` green on the final head.

---

## 3. Argument path (six sections, each answering the last)

### Opening (before Section 1)
- **Claim:** State the two unanswered questions left at the close of [Complementary bottlenecks](2026-11-26-complementary-bottlenecks.md) and [From bottlenecks to better algorithms](2026-11-26-from-bottlenecks-to-better-algorithms.md)—what the decoder's true floor is once experimental copying and leftover word spelling are removed, and whether pooling the Huffman scan into the workers unlocks the 36% scanner–construction pair—and deliver the answers and five takeaways immediately.
- **Method:** Two concise grounding paragraphs (recapping the six-part pipeline `S, M, C, E, R, D`, the ~7.7-worker knee, and why the first matrix's 112–142 ms floor was contaminated), followed by the takeaways list and a one-sentence route through Sections 1–6.
- **Motivation:** *Supplied* — ensures a cold reader understands the stakes and the system before seeing any arm name or timing table.

### Section 1. How much of the first matrix's floor was the decoder's?
- **Salient feature:** Once the oracles stopped copying 4.9 GB of recorded answers and emission's word spelling was removed too, the all-ideal decode collapsed from 112–142 ms to a flat **5.2–5.7 ms** at every worker count—and the supposed memory-bandwidth rise from 8 to 10 workers vanished completely.
- **Claim:** The decoder's inter-stage plumbing (queues, locks and hand-offs) is negligible (**2.7%** of the shipped 8-worker wall), and memory bandwidth does not form a third binding station between 8 and 10 workers.
- **Method:**
  - Explain what changed in the instrumentation (`bench-zero-copy-floor`, PR `#917`, zero-copy borrow oracles in PR `#921`, evidence `cc8ba137`): each oracle hands over a borrowed slice instead of copying its buffer, and a new setting `F` (`E_full`) skips emission's word-to-byte spelling (`E_consume`) alongside the graph walk.
  - Report the run's controls briefly and cleanly: 648 decodes in 16 minutes on the i9-10900K (wikidata ×4, 8 randomised blocks, 4 traced, scanner pinned to CPU 6); 216/216 pins held, removed parts' busy time ≤ 0.15% (against a 5% VOID line), zero spelling calls in any `F` arm, every decode byte-identical, live memory-controller counters.
  - Present the two floors across 1, 6, 7, 8, 9 and 10 workers:
    - **Residual B (`SFCMRD`, plumbing alone):** `5.7 [5.5, 5.8] ms` at 1 worker, `5.2 [5.0, 5.5] ms` at 8 workers, `5.4 [5.3, 5.5] ms` at 10 workers (`5.2–5.7 ms` everywhere; median `5.3 ms` at 8, **2.7%** of shipped `194.6 ms`). Per-worker handover overhead in traces drops to `0.96 ms/thread` at 1 worker and `0.15 ms/thread` at 8 (0.3–0.4% of Residual A). Score the design analysis's 14–19 ms 1-worker forecast cleanly as a **MISS**—plumbing is a third of what was feared, and closed as an optimisation target.
    - **Residual A (`SECMRD`, spelling kept, zero-copy):** `303.9 [302.9, 305.0] ms` at 1 worker, `54.0` at 6, `46.7` at 7, **`42.4 [40.3, 44.4]` at 8 → `42.9 [40.4, 45.3]` at 9 → `40.5 [37.6, 43.5] ms` at 10**. Memory bandwidth stays at **12.8–16.6 GB/s** in the all-ideal arms and **15.1 GB/s** shipped at 10 workers—nowhere near saturation. The first matrix's 112 → 142 ms rise was the copying oracles' traffic; the three-station memory hypothesis is not supported here.
- **Motivation:** *Stated by sources* (`results/floor/README.md` §§1, 4; `complementary-bottlenecks.md` §6).

### Section 2. What remains when one stage at a time is real?
- **Key step:** Removing the copying noise does not just shrink the floor; it exposes the exact isolated cost of every stage and uncovers two major costs the first matrix could not see.
- **Claim:** At 8 workers the serial scan alone takes **179.1 ms** (92% of the shipped wall, capping worker-only gains at ≤ 8%), while beneath it sit two newly quantified targets: **298 ms** of scalable word spelling at 1 worker (`E_consume`) and **48 ms** of flat serial delivery copy-out (`D`).
- **Method:**
  - Present the clean "one part real, all else free" table of medians at 8 workers and 1 worker against shipped `194.6 ms` / `1,342.7 ms`:
    - Scan (`FCMRD`): **179.1 ms** (8W) / **169.7 ms** (1W) — 92% of the shipped 8-worker wall.
    - Emission, walk + spelling (`SCMRD`): **98.1 ms** (8W) / **694.2 ms** (1W).
    - Construction (`SFMRD`): **83.6 ms** (8W) / **609.5 ms** (1W) — explicitly state the **cold-store caveat**: zero-copy borrowing leaves the walk (`A7`) reading construction's output cold (`98.9 ms/worker` in `SCMRD` vs `72.4 ms` shipped at 8W; `697.9` vs `545.2 ms` at 1W), so `SFMRD` slightly overstates construction's isolated cost.
    - Delivery (`SFCMR`): **52.3 ms** (8W) / **54.7 ms** (1W).
    - MTF/RLE2 (`SFCRD`): **20.1 ms** (8W) / **141.1 ms** (1W).
    - RLE1/CRC/output (`SECMD` vs `SECMRD`): **~14 ms** (8W) / **~66 ms** (1W).
    - Nothing (`SFCMRD`): **5.3 ms** (8W) / **5.8 ms** (1W).
  - Draw out the three consequences that reshape the engineering plan:
    1. **Why the scan must move first at 8 workers:** `FCMRD = 179.1 ms` sharpens the matrix's `S × C` finding into a hard bound: even making every worker stage cost zero saves at most 8% at 8 workers while scanning stays serial.
    2. **Hidden cost 1 — emission's word spelling (`E_consume = A − B`):** **298.2 [297.0, 299.4] ms** at 1 worker (**22%** of the 1,342.7 ms wall) and **37.2 [35.2, 39.1] ms** at 8 workers—making up **87–93%** of `SECMRD` across counts (confirmed by trace ledger: `299.7 ms` at 1W, `39.8 ms/thread` at 8W). The first matrix missed it because its emission oracle left spelling in place. It scales cleanly with workers (`298 → 49 → 41 → 37 → 38 → 35 ms`), so it matters below the knee and above it once scanning is relieved.
    3. **Hidden cost 2 — delivery's serial copy-out (`SFCMR − SFCMRD`):** **47.3–49.3 ms** at every worker count (`48.5 ms` at 1W, `47.3 ms` at 8W, `49.3 ms` at 10W). The in-order reader copies **839 MB** to the caller on one thread at **~17 GB/s** (`16.9–17.5 GB/s`, `5.0–5.1 M` LLC load misses vs `0.3 M` in `SFCMRD`). It does not scale with workers: once scanning leaves its single thread and workers get faster, this 48 ms serial copy-out becomes the next floor.
- **Motivation:** *Stated by sources* (`results/floor/README.md` §§2, 3, 5, 6).

### Section 3. Designing a pooled scan — and how to test a masked stage fairly
- **Reconnection:** Directly picks up [From bottlenecks to better algorithms, §9](2026-11-26-from-bottlenecks-to-better-algorithms.md#9-dissolving-the-serial-scanner) and connects the floor run's findings to the pooled scan's pre-registered test.
- **Claim:** Taking scanning off its serial thread failed three times earlier for specific structural reasons (non-binding workload, unbounded look-ahead memory, and serialised/duplicated helper scanners); the new prototype fixes all three, and tests its value fairly by pairing it with construction's zero-copy oracle rather than judging it against a 3.2% standalone ceiling.
- **Method:**
  - **Lessons from three earlier parallel scans (`pooled-scan/registration.md` §2):**
    1. *10 November (`bench-par-scan`):* proved the marker finder is nearly free (**9.3 GB/s**, 0.19% of decode), and won on level-1 (1.615×) and records (1.780×), but level-9 Wikidata was still worker-bound then (0.981 at 10W).
    2. *22 November (`bench-parscan-c37`, `#741`):* won at 8 and 10 workers (`0.935`, `0.883`), but slowed 2–6 (`1.11–1.13`) and blew up peak RSS to **1.28× at 10** and **1.99× at 20** by buffering the whole input and holding $4 \times \text{threads}$ blocks in flight.
    3. *23 November (`bench-parscan-stream`, `#770`):* capped memory, yet slowed 8–20 workers by **54–61%** (`1.605×` at 8, `1.537×` at 10) because a lead scanner plus two helpers serialised behind one chain head and **nearly doubled** scanning CPU (`0.18 + 0.07 + 0.08 = 0.33–0.36 s` vs `0.17 s` base), leaving workers ~40% idle.
  - **The prototype's mechanism (`registration.md` §3):** the feeder thread runs an AVX2 nibble filter for bzip2's 48-bit block marker on chunks as they arrive (bounded memory); there is no scanner thread and no joiner thread—each worker claims a candidate inside the shipped admission window and runs `scan · M · C · E · R` end-to-end, eliminating the **190.6 MiB** symbol queue; the in-order reader accepts blocks matching the exact bit-offset chain and falls back serially on any false or missed marker (exact bytes and exact errors preserved unconditionally; 1 worker stays on the shipped path).
  - **The paired-oracle kill test (`registration.md` §§4, 5, 7):** explain why the design analysis's original standalone −7% gate at 8–9 workers was impossible by construction (S's whole standalone room at 8 is 3.2%). Instead, judge `P` behind construction's zero-copy oracle (`PC/C`) against the ideal scan behind `C` (`SC/C`):
    $$\text{capture} = \frac{\ln(C / PC)}{\ln(C / SC)}, \qquad \text{K1 kills if } \text{UB}_{95\%}\!\left(\frac{PC}{C}\right) > \sqrt{\frac{SC}{C}}.$$
    State the remaining frozen gates—**K2** (standalone `P/none` 95% lower bound > 1.03), **K3** (median peak RSS `P/none` > 1.10 at 6–10), **K4** (single-threaded finder > 10 ms)—and show how Section 1's floor run automatically set §5's switches *before any code ran*: Residual A stayed flat from 8 to 10, so **both 8 and 9 workers were judged**, and Residual B was 2.7% (< 10%), so capture stayed capped.
- **Motivation:** *Stated by sources* (`pooled-scan/registration.md` §§1–5, 7).

### Section 4. The kill test's verdict: KILL, narrowly, on K1
- **Salient feature:** Every engineering failure mode from the past was solved—exactness, memory, finder speed, zero wasted decodes, and worker idleness—yet K1 fired at both judged counts by margins of **0.009** and **0.042**.
- **Claim:** The pooled scan prototype ran cleanly with zero fallbacks and captured roughly half of the scan's ideal room behind construction (**0.52** at 8 workers, **0.48** at 9), missing the pre-registered 95%-upper-bound proof of ≥ 50% capture at both judged counts while passing K2, K3 and K4.
- **Method:**
  - State the run's validity (`pooled-scan/run-20261126T133034Z`, recipe `bench-pooled-scan`, `#932`, evidence `c71c4a56`, 480 decodes + 11-pass finder in 15 minutes, 12 randomised blocks, unpinned): all 10 exactness tests passed, and every pooled decode was wholly pooled (`0 dropped, 0 fallbacks, 0 degraded`).
  - Present the full K1/K2/capture table across 6, 7, **8 (judged)**, **9 (judged)** and 10 (reported):
    - **8 workers (judged — K1):** `PC/C = 0.8618 [0.8481, 0.8758]` vs bar `0.8670` (capture **0.521**; upper bound misses by **0.009**; medians `C = 183.2`, `PC = 158.1`, `SC = 135.3 ms`); `P/none = 1.0519 [1.0233, 1.0812]` (K2 passes: lower bound `1.023 ≤ 1.03`).
    - **9 workers (judged — K1):** `PC/C = 0.8685 [0.8328, 0.9058]` vs bar `0.8636` (capture **0.481**; upper bound misses by **0.042**; medians `C = 184.2`, `PC = 159.9`, `SC = 140.2 ms`); `P/none = 0.9999 [0.9832, 1.0170]` (break-even).
    - **10 workers (reported):** `PC/C = 0.8484 [0.8249, 0.8725]` vs `0.8772` (capture **0.627**); `P/none = 0.9357 [0.9086, 0.9637]` (**−6.4%**, `187.2` vs `197.6 ms`).
    - **6–7 workers (reported):** `P/none = 1.1045` (+10.4%) and `1.1004` (+10.0%); `PC/C = 1.0646` (capture **−0.506**) at 6 and `0.9351` (capture **0.291**) at 7.
  - Report the passing gates and plumbing checks:
    - **K3 (memory):** peak RSS `P/none` between `0.988` and `1.014` across 6–10 workers (within 1.4%, eliminating c37's 1.28–1.99× bloat).
    - **K4 (finder):** median **4.82 ms [4.81, 5.00]** (**9.842 GB/s**, AVX2 on CPU 4, 1,724 block magics and 48 end magics over 47.4 MB)—less than half the 10 ms limit.
    - **Plumbing in traces:** chain copied **45.2 MiB** in **4.32–7.24 ms** (estimated ~5 ms), finder ran in **4.97–7.35 ms** in-decode, the **190.6 MiB** symbol queue disappeared, and worker idle share (`job_wait / life`) dropped from **6.5–20.2%** in `none` to **1.2–2.5%** in `P` at 8–10 workers.
  - Contrast standalone wall against the companion post's unscored forecast (`172–175 ms` at 8–9): measured **207.5 [206.5, 208.6] ms** at 8 and **195.2 [192.3, 198.1] ms** at 9.
- **Motivation:** *Stated by sources* (`pooled-scan/results/README.md`, `report.txt`, `pooled-scan.tsv`).

### Section 5. Why pooling wasn't enough: redistributing work onto busy workers
- **Key step:** Explains the exact causal reason for *every* number in Section 4—why standalone pooling lost by 5–10% at 6–8 workers, broke even at 9, won by 6.4% at 10, and left a 23 ms gap between `PC` (158.1 ms) and `SC` (135.3 ms) at 8 workers.
- **Claim:** Pooling does not remove the scan's CPU—workers spent the same **173–193 ms** scanning (`A2 + A3`) as the serial scanner (**171–190 ms**)—so replacing a dedicated scanner with worker-pool scanning adds $s/k$ to every worker's load and can beat the serial scanner only when workers have enough slack ($k > 1 + w/s$).
- **Method:**
  - Show the primary trace evidence first: workers' summed scanning CPU in `P` (`172.9, 174.1, 177.7, 182.8, 192.7 ms` at 6–10 workers) equals the serial scanner's in `none` (`171.3, 170.6, 177.5, 173.4, 189.6 ms`). Unlike `#770`, zero work was duplicated—the work simply moved from one core onto the $k$ worker cores.
  - Give the two-line queueing comparison (reconnecting to [From bottlenecks to better algorithms, §9](2026-11-26-from-bottlenecks-to-better-algorithms.md#9-dissolving-the-serial-scanner)):
    $$T_{\text{serial}}(k) \approx \max\!\left(s,\ \frac{w}{k}\right), \qquad T_{\text{pooled}}(k) \approx \frac{s + w}{k} + \epsilon.$$
    Work through the exact arithmetic on the measured numbers:
    - Below the knee ($w/k \ge s$), $(s + w)/k$ is strictly worse than $w/k$ by at least $s/k$ (~22–30 ms), which cannot be offset by removing the symbol queue because Section 1 proved the pipeline plumbing is only ~5 ms.
    - Above the knee ($w/k < s$), pooling beats the dedicated scanner only once $(s + w)/k < s$, i.e.
      $$k > 1 + \frac{w}{s}.$$
      With the measured knee $w/s \approx 7.7$, even zero-overhead pooling cannot break even until $k \approx 8.7$; with ~10 ms of finder and copy overhead, break-even lands right at **$k = 9$** (measured `1.000`), losing at $k = 8$ (`207.5 ms` vs `192.9 ms` median; `185 + 178/8 ≈ 207 ms`) and winning at $k = 10$ where shipped workers sat **20.2%** idle (`187.2` vs `197.6 ms`, −6.4%).
    - Behind ideal construction (`SC = 135.3 ms` at 8 workers, `140.2 ms` at 9), adding $s/k \approx 178/8 \approx 22\text{ ms}$ (and $178/9 \approx 20\text{ ms}$) puts `PC` at ~158 and ~160 ms—matching the measured **158.1 ms** and **159.9 ms** almost to the millisecond, and explaining why pooling can only capture roughly half of an *ideal* (zero-CPU) scan at 8–9 workers!
- **Motivation:** *Recoverable from sources* (`pooled-scan/results/README.md` §"What the run shows about why"; `from-bottlenecks-to-better-algorithms.md` §9) — supplies the exact quantitative bridge that makes the narrow miss feel inevitable rather than unlucky.

### Section 6. Keeping the verdict — and what the two runs tell us to build next
- **Payoff & Transfer:** Closes the experimental arc cleanly and equips the reader with the general principle for evaluating work-redistribution changes in coupled pipelines.
- **Claim:** The frozen K1 verdict stands—the prototype stops as built rather than relaxing a gate after seeing data—and together the two runs reorder the next work unambiguously: cut worker-side demand first, keep the serial scanner alive with slack-only worker scan-ahead, and prepare for delivery's 48 ms serial floor.
- **Method:**
  - **Why the verdict is not relaxed:** Missing K1 by 0.009 at 8 workers looks close, yet Section 5 shows the miss is structural, not noise: unconditional pooling still taxes the shipped 8-worker configuration whenever workers lack slack. Honouring the pre-registered decision rule prevents shipping complexity that regresses 6–8 workers.
  - **The redrawn map (what is closed vs. what opens next):**
    1. *Closed:* pipeline plumbing (~5 ms, 2.7%); the third-station memory hypothesis at 8–10 workers; unconditional full pooling at 8 workers as built.
    2. *Worker-side reductions come first:* cutting $w$ both speeds up 1–7 workers immediately and lowers $1 + w/s$ so scanning relief can pay at 8 workers. Three concrete targets sit open: emission's newly exposed **298 ms** word spelling at 1 worker (`spelling-census`), the MTF-to-construction direct fold (`mtf-fold-census`), and forward wiring with multi-lane emission (`forward-emission`).
    3. *A slack-only hybrid scan (`pooled-scan-hybrid`):* keep the dedicated serial scanner thread so busy workers never pay a scan tax below the knee, and let idle workers scan ahead *only* when they would otherwise wait for work (`job_wait`). Alternatively, re-test full pooling under a fresh registration once worker-side reductions lower $w$.
    4. *The next serial floor — delivery's 48 ms copy-out (`delivery-copyout`):* once scanning is relieved and workers drop toward 100 ms, copying 839 MB on one thread at 17 GB/s becomes nearly half of the remaining decode—an API and ownership question (caller-provided buffers, direct `read()` into the user's slice, parallel copy, or a zero-copy block-lending API).
- **Motivation:** *Stated by sources* (`results/floor/README.md` §6; `pooled-scan/results/README.md` §"What it means"; `frontier/surface.tsv`; `frontier/claims/`).

---

## 4. Planned figures and visual analogies

Follow `write-blog-post/SKILL.md`'s visual causal reasoning rule and `dataviz` conventions: each figure pairs plain technical labels with a clear causal comparison or visual analogy, marks judged vs. reported points where relevant, and states in its caption where the analogy stops holding.

1. **Figure 1 (Sections 1 & 2): Taring the scale — from the first matrix's false floor to each stage alone (8 workers)**
   - *Type:* Two-panel empirical chart.
   - *Left panel:* Decomposition of the 8-worker all-ideal wall (first matrix's 112 ms → zero-copy `SECMRD` 42.4 ms → `E_consume` 37.2 ms + `SFCMRD` 5.2 ms), with an inset comparing the first matrix's rising 8 → 10 curve (`112 → 142 ms`) against zero-copy `SECMRD`'s flat curve (`42.4 → 42.9 → 40.5 ms` with 95% CIs).
   - *Right panel:* Horizontal bar chart of each stage alone real at 8 workers against the shipped 194.6 ms wall (`FCMRD` 179.1, `SCMRD` 98.1, `SFMRD` 83.6 [marked with cold-store caveat], `SFCMR` 52.3, `SFCRD` 20.1, `R` ~14, `SFCMRD` 5.3 ms).
   - *Visual analogy:* Taring a scale so the shipping crates (4.9 GB of oracle copies and unremoved word spelling) come off the reading, exposing how light the tray itself is (5 ms) and how heavy the remaining single-stage items are.
   - *Where the analogy stops:* Pipeline stages overlap in time rather than summing like static weights (`179 + 98 + 84` > `195 ms`), and zero-copy borrowing leaves construction's output cache-cold for the walk.
2. **Figure 2 (Section 2): Draining the reservoir — scalable spelling vs. flat serial floors (1–10 workers)**
   - *Type:* Multi-series line chart with 95% CI bands over $k \in \{1, 6, 7, 8, 9, 10\}$ showing `E_consume` (`298.2 → 35.2 ms`, scaling down with workers) against three flat horizontal ledges: Scan alone `FCMRD` (`169.7–179.1 ms`), Delivery copy-out `D` (`47.3–49.3 ms`), and Plumbing `SFCMRD` (`5.2–5.7 ms`).
   - *Visual analogy:* Adding workers drains the water level of parallel work (`E_consume`), exposing fixed rock ledges one after another: the 179 ms scan reef, the 48 ms delivery ledge, and the 5 ms riverbed.
   - *Where the analogy stops:* Adding workers slightly raises single-thread scan time via shared-cache contention (`169.7 → 179.1 ms`), and delivery's 48 ms overlaps with decoding until workers outpace the reader.
3. **Figure 3 (Section 4): The pooled-scan kill test — paired-oracle capture (K1) and standalone crossover (K2)**
   - *Type:* Two-panel interval-strip chart over $k \in \{6, 7, 8, 9, 10\}$ explicitly distinguishing **judged** ($k = 8, 9$) from **reported** ($k = 6, 7, 10$).
   - *Left panel:* `PC/C` point estimates and 95% CIs against the $\sqrt{SC/C}$ K1 threshold and `SC/C` ideal-scan floor, annotating capture (`−0.51, 0.29, 0.52, 0.48, 0.63`) and the exact upper-bound misses (`+0.009` at 8, `+0.042` at 9).
   - *Right panel:* `P/none` point estimates and 95% CIs against the `1.00` break-even line and K2's `1.03` kill threshold (`1.105, 1.100, 1.052, 1.000, 0.936`).
4. **Figure 4 (Sections 5 & 6): Moving work vs. stealing slack — dedicated scanner, full pool, and hybrid scan-ahead**
   - *Type:* Causal capacity/swim-lane diagram at $k = 8$ (balanced knee) and $k = 10$ (worker slack) comparing `none` (1 scanner + $k$ workers), `P` ($k$ pooled workers absorbing $s/k$), `PC` vs `SC` (showing the exact $s/8 \approx 22\text{ ms}$ gap), and the proposed slack-only hybrid.
   - *Visual analogy:* A kitchen with one prep cook (`S`) and $k$ line cooks (`W`). Dismissing the prep cook and splitting prep across the line cooks eliminates waiting between courses (idle time falls from 6.5–20% to 1–2.5%), but when the line cooks are already flat out ($k \le 8$), each cook takes on $s/k$ more work and service slows down; only when line cooks are waiting for prep ($k = 10$), or when only idle cooks step in to help (the hybrid), does service speed up.
   - *Where the analogy stops:* Software cores share an L3 cache, pay ~5 ms each for boundary finding and candidate copying, and still commit finished blocks strictly in order at the reader.

---

## 5. Goal questions, transfer question and held-out questions

### Goal questions (for the simulated-reader pass during `review`)
1. **(`motivation`) Why did the team run a second, zero-copy floor experiment after the 4-hour ideal-parts matrix was already published?**
   - *Key points:* [background] The first matrix's all-ideal walls (391 ms at 1W, 112 → 142 ms at 8–10W) mixed pipeline plumbing, ~4.9 GB of oracle buffer copying, and emission's unremoved word-to-byte spelling; [reason] without separating them, the team could neither tell whether inter-stage plumbing needed work nor whether memory bandwidth formed a third bottleneck above 8 workers; [conclusion] zero-copy borrows plus the `F` (no-spelling) arm settled both in 16 minutes: plumbing is 5.2–5.7 ms (closed) and Residual A is flat at 40–43 ms from 8 to 10 workers (no third memory station).
2. **(`summary`) What two large costs did the zero-copy floor uncover that the first matrix had hidden, and how differently do they behave as workers increase?**
   - *Key points:* [claim] Emission's word spelling (`E_consume`) and delivery's serial copy-out (`D`); [reason] `E_consume` takes 298 ms at 1 worker (22% of wall) and scales down to 37 ms at 8 workers, because workers run it in parallel; [reason] delivery's copy-out takes 48 ms flat (47.3–49.3 ms) at every worker count because one reader thread copies 839 MB at ~17 GB/s; [conclusion] spelling is a priority worker-side target now, while delivery becomes the next hard serial floor once scanning is relieved.
3. **(`obvious-fails`) Why didn't the team judge the pooled scan with a simple standalone speedup test at 8 workers, and why did the prototype still fail its paired-oracle test despite passing every plumbing check?**
   - *Key points:* [reason] At 8 workers the serial scan's entire standalone room is only 3.2%, so any standalone 7% speedup bar kills every pooled scan by construction; [claim] pairing `P` with construction's zero-copy oracle (`PC/C` against $\sqrt{SC/C}$) isolates the scan's 32% room behind construction; [reason] pooling passed exactness, memory (K3 ≤ 1.4%), finder speed (K4 = 4.82 ms) and standalone K2, yet workers still executed the same ~178 ms of scan CPU; [conclusion] spreading $s$ across $k$ workers adds $s/k \approx 22\text{ ms}$ on top of `SC`'s 135 ms, landing at 158 ms (52% capture at 8, 48% at 9) and missing K1's 95% upper-bound bar by 0.009 and 0.042.
4. **(`structure`) Why was the pooled scan slower at 6–8 workers, break-even at 9, and 6.4% faster at 10?**
   - *Key points:* [claim] Replacing a dedicated scanner with worker-pool scanning changes $\max(s, w/k)$ into $(s + w)/k + \epsilon$; [reason] below the knee ($w/k \ge s$) workers are already saturated, so adding $s/k$ slows them down (+10% at 6–7, +5% at 8); [reason] pooling beats a dedicated scanner only once $k > 1 + w/s$ (about 8.7 before overhead, landing at 9 with ~10 ms of finder/copy cost); [conclusion] at 10 workers, where shipped workers sat 20.2% idle waiting on the serial scanner, absorbing the scan consumes slack and cuts wall time by 6.4%.

### Transfer question (the closing section must equip the reader to answer this)
5. **(`transfer`) Suppose a streaming pipeline has a dedicated serial parsing stage taking $s = 100\text{ ms}$ and a worker pool whose total parallel work is $w = 400\text{ ms}$, with negligible plumbing overhead. Would removing the dedicated parser and making each of $k = 5$ workers parse its own blocks speed the pipeline up, and what architectural change would win at both $k = 4$ and $k = 6$?**
   - *Expected answer:* At $k = 5$, the dedicated parser takes $\max(100, 400/5) = 100\text{ ms}$, while 5 pooled workers take $(100 + 400)/5 = 100\text{ ms}$ (break-even at $k^* = 1 + w/s = 5$ even with zero overhead; slightly slower once finder/copy overhead is included). At $k = 4$ pooling loses ($125$ vs $100\text{ ms}$); at $k = 6$ it wins ($83.3$ vs $100\text{ ms}$). Keeping the dedicated parser and letting workers parse ahead *only when idle* (a slack-only hybrid) avoids slowing $k = 4$ while capturing slack at $k = 6$; alternatively, cutting worker work $w$ first lowers the break-even count $1 + w/s$ below 5.

### Held-out questions (do not draft specifically to answer these; test during `review`)
6. **(`non-routine`, held out)** Why should a reader treat construction's 83.6 ms isolated row in the zero-copy floor table differently from the scan's 179.1 ms row?
7. **(`justification`, held out)** If the pooled scan missed K1 at 8 workers by only 0.009 while capturing 52% of the room behind ideal construction, why did the team stop the prototype instead of shipping it behind an 8-worker flag?

---

## 6. Link-and-gloss plan (for `draft` and `revise`)

- **Internal blog posts to link at first mention:**
  - [Complementary bottlenecks: reading the ideal-parts matrix](2026-11-26-complementary-bottlenecks.md) (the 44-arm matrix, the six parts `S, M, C, E, R, D`, `S × C` = −36% at 8 workers, and the contaminated 112–142 ms floor).
  - [From bottlenecks to better algorithms](2026-11-26-from-bottlenecks-to-better-algorithms.md) (the residual accounting of Residuals A and B, the three-station memory hypothesis, and the pooled-scan design and 172–175 ms forecast in §9).
  - [The decoder as a queueing network](2026-11-25-the-decoder-as-a-queueing-network.md) (the bottleneck law and the measured 7.67-worker knee).
  - [Open Returns: decoding a block from a tenth of its rows](2026-11-19-open-returns.md) (construction `C` and emission `E` in the interval-exchange kernel, and what "spelling words into bytes" means).
  - [Huffman decoding in mibz2: keeping the bit buffer in a register](2026-11-05-huffman-bit-buffer.md) (what the Huffman scan `S` does).
  - [Masked bottlenecks in a two-stage decoder](2026-11-24-the-scanner-hides-the-emitter.md) (how one bottleneck hides another in a coupled pipeline).
- **Primary repository sources to link (commit-pinned at draft time):**
  - `docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md` and `report.txt`
  - `docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md`
  - `docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md` and `report.txt`
- **Candidate fleuron glosses (3–4 max, skippable asides):**
  1. **Zero-copy borrow oracle:** how a stand-in hands a stage a read-only reference to pre-recorded output rather than copying buffers under a lock, why that removes replay traffic, and why it leaves downstream caches cold.
  2. **48-bit block marker (magic):** how bzip2 marks each compressed block with the first twelve digits of $\pi$ in binary-coded decimal (`0x314159265359`), why markers are not byte-aligned, and how an AVX2 nibble filter scans all eight bit alignments at nearly 10 GB/s without ever trusting a false match.
  3. **Log-scale capture ($\ln(C/PC) / \ln(C/SC)$):** why a multiplicative threshold sets "half the room" at the geometric mean $\sqrt{SC/C}$ rather than the arithmetic midpoint.
  4. **Pre-registered kill test (C2):** why freezing the decision threshold before running the benchmark—and keeping a narrow 0.009 miss as KILL—prevents rationalising a structural flaw as noise.

---

## 7. Verified fact and number ledger

Every figure below was verified directly against `docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/`, `docs/journal/2026-11-24-algorithm-level/pooled-scan/results/`, `pooled-scan/registration.md`, `ideal-parts/results/full/README.md`, and `origin/evidence` on 26 November 2026.

| Claim / figure | Tag | Primary source |
|---|---|---|
| Zero-copy floor run: `zero-copy-floor/run-20261126T112751Z`, recipe `bench-zero-copy-floor` (PR `#917` at `1e27e46f`, oracles `#921`), `origin/evidence` commit `cc8ba137`, SHA-256 `10df0d89…5e34`; 648 decodes in 16 min on i9-10900K (wikidata ×4, 8 blocks, 4 traced, scanner pinned CPU 6) | `[VERIFIED]` | `results/floor/README.md`; `results/floor/report.txt`; `git log origin/evidence` |
| Floor validity: 100% byte-identical; scanner pin held in 216/216 pinned traces; removed parts' busy time ≤ 0.15% (vs 5% VOID); 0 spelling calls in any `F` arm; live memory-controller counters | `[MEASURED]` | `results/floor/README.md`; `results/floor/report.txt` lines 4–12 |
| Residual B (`SFCMRD`, plumbing alone): 1W `5.7 [5.5, 5.8]`, 6W `5.3 [5.3, 5.4]`, 7W `5.2 [4.9, 5.5]`, 8W `5.2 [5.0, 5.5]` (**2.7%** of shipped 194.6 ms), 9W `5.3 [5.1, 5.5]`, 10W `5.4 [5.3, 5.5]` ms (medians 5.8 ms at 1W, 5.3 ms at 8W); forecast 14–19 ms at 1W scored **MISS** | `[MEASURED]` | `results/floor/README.md` §1; `results/floor/report.txt` lines 17–22, 71, 79, 106; `zero-copy-floor.tsv` |
| Residual A (`SECMRD`, spelling kept, zero-copy): 1W `303.9 [302.9, 305.0]` (ratio `0.2277`), 6W `54.0 [51.2, 56.7]`, 7W `46.7 [45.1, 48.2]`, **8W `42.4 [40.3, 44.4]` → 9W `42.9 [40.4, 45.3]` → 10W `40.5 [37.6, 43.5]` ms** (flat; third-station hypothesis not supported) | `[MEASURED]` | `results/floor/README.md` §4; `results/floor/report.txt` lines 17–22; `zero-copy-floor.tsv` |
| Live memory bandwidth (floor run): `SFCMRD` 11.9–13.1 GB/s; `SECMRD` 15.2–16.6 GB/s at 8–10W; `SFCMR` 16.9–17.5 GB/s; shipped `none` 8.6 GB/s (8W), 11.6 GB/s (9W), 15.1 GB/s (10W) | `[MEASURED]` | `results/floor/README.md` §4; `results/floor/report.txt` lines 77–130 |
| Worker handover overhead in `SECMRD` traces: `0.96 [0.89, 1.02]` ms/thread at 1W (0.3% of Residual A); `0.15 [0.15, 0.16]` ms/thread at 8W (0.4% of Residual A) | `[MEASURED]` | `results/floor/report.txt` lines 56–67; `zero-copy-floor.tsv` lines 134–169 |
| Each stage alone real (medians, 8W / 1W, vs shipped `194.6` / `1,342.7 ms`): Scan `FCMRD` **179.1 / 169.7 ms** (92% of 8W wall, ≤ 8% worker-only room); Emission `SCMRD` **98.1 / 694.2 ms**; Construction `SFMRD` **83.6 / 609.5 ms**; Delivery `SFCMR` **52.3 / 54.7 ms**; MTF/RLE2 `SFCRD` **20.1 / 141.1 ms**; RLE1/CRC (`SECMD − SECMRD`) **~14 / ~66 ms**; Nothing `SFCMRD` **5.3 / 5.8 ms** | `[MEASURED]` | `results/floor/README.md` §2; `results/floor/report.txt` lines 77–112 |
| Cold-store caveat on `C`: walk (`A7`) per worker in `SCMRD` vs `none` is `697.9` vs `545.2 ms` at 1W and `98.9` vs `72.4 ms` at 8W (`SFMRD` overstates `C`'s isolated cost by roughly that much) | `[MEASURED]` | `results/floor/README.md` §5; `results/floor/report.txt` lines 38–44 |
| `E_consume` (`A − B`, emission word spelling): 1W **298.2 [297.0, 299.4] ms** (**22%** of 1,342.7 ms wall), 6W `48.7 [45.9, 51.4]`, 7W `41.5 [40.0, 43.0]`, 8W **37.2 [35.2, 39.1]**, 9W `37.6 [35.1, 40.1]`, 10W `35.2 [32.3, 38.0]` ms; 87–93% of `SECMRD`; ledger `299.7 ms` (1W), `39.81 ms/thread` (8W) | `[MEASURED]` | `results/floor/README.md` §3; `results/floor/report.txt` lines 17–22, 56–67; `zero-copy-floor.tsv` |
| Delivery copy-out (`SFCMR − SFCMRD`): 1W `48.5 [46.8, 50.3]`, 6W `47.9 [47.1, 48.8]`, 7W `48.5 [47.7, 49.3]`, 8W `47.3 [46.4, 48.2]`, 9W `49.1 [48.0, 50.2]`, 10W `49.3 [46.8, 51.8]` ms (**47.3–49.3 ms**, ~48 ms flat); 839 MB copied on 1 thread at ~17 GB/s (`5.0–5.1 M` LLC load misses vs `0.3 M` in `SFCMRD`) | `[MEASURED]` | `results/floor/README.md` §3; `results/floor/report.txt` lines 17–22, 79–125; `zero-copy-floor.tsv` |
| First matrix continuity (`origin/evidence` `7dd7b070`, 2,492 decodes): 1W `C` −42.1% (`1,306→756 ms`); 8W shipped 195 ms, `S` −3.2%, `C` −5.6%, `S+C` −36% (`195→125 ms`); all-ideal walls 391 / 94 / 112 / 142 ms (~4.9 GB oracle copying) | `[MEASURED]` | `results/full/README.md` §§1–4; `2026-11-26-complementary-bottlenecks.md`; `2026-11-26-from-bottlenecks-to-better-algorithms.md` §3 |
| Three earlier parallel-scan attempts: `bench-par-scan` (10 Nov: finder 9.3 GB/s, Wikidata 0.981 at 10W, L1 1.615×, records 1.780×); `bench-parscan-c37` (`#741`: 0.935 at 8W, 0.883 at 10W, 1.11–1.13 at 2–6W, peak RSS 1.28× at 10W / 1.99× at 20W); `bench-parscan-stream` (`#770`, evidence `e432a2ba…44340bb7`: 1.605× at 8W, 1.537× at 10W, 1.540× at 20W; lead 0.18–0.20 s + two helpers 0.07–0.08 s each ≈ 0.33–0.36 s vs 0.17 s base, workers ~40% idle) | `[MEASURED]` | `pooled-scan/registration.md` §2 |
| Pooled-scan kill test run: `pooled-scan/run-20261126T133034Z`, recipe `bench-pooled-scan` (PR `#932` at `1f6a578c`), `origin/evidence` commit `c71c4a56`, SHA-256 `97e397ad…8867`; 480 decodes + 11-pass finder in 15 min on i9-10900K (wikidata ×4, `47,419,732` B, 12 blocks, 3 traced, unpinned, finder pinned CPU 4) | `[VERIFIED]` | `pooled-scan/results/README.md`; `pooled-scan/results/report.txt`; `phases.tsv`; `git log origin/evidence` |
| Pooled-scan validity: all 10 exactness tests passed; every pooled decode wholly pooled (`0 dropped, 0 fallbacks, 0 degraded`) | `[MEASURED]` | `pooled-scan/results/README.md`; `phases.tsv` line 4; `report.txt` lines 37–65 |
| K1 verdict (KILL at both judged counts): **8W** `PC/C = 0.8618 [0.8481, 0.8758]` vs $\sqrt{SC/C} = 0.8670$ (`SC/C = 0.7516`), capture **0.521**, upper bound misses by **0.0088 ≈ 0.009**; **9W** `PC/C = 0.8685 [0.8328, 0.9058]` vs $\sqrt{SC/C} = 0.8636$ (`SC/C = 0.7459`), capture **0.481**, upper bound misses by **0.0422 ≈ 0.042**; reported **10W** `0.8484 [0.8249, 0.8725]` vs `0.8772`, capture **0.627**; **7W** `0.9351`, capture `0.291`; **6W** `1.0646`, capture `−0.506` | `[MEASURED]` | `pooled-scan/results/README.md`; `pooled-scan/results/report.txt` lines 7–13, 91–94; `pooled-scan.tsv` |
| K2 standalone `P/none` (PASS; kill if lower bound > 1.03 at 8 or 9): 1W `1.0026 [0.9927, 1.0126]`, 6W `1.1045 [1.0942, 1.1148]`, 7W `1.1004 [1.0871, 1.1140]`, 8W `1.0519 [1.0233, 1.0812]`, 9W `0.9999 [0.9832, 1.0170]`, 10W **`0.9357 [0.9086, 0.9637]`** (−6.4%) | `[MEASURED]` | `pooled-scan/results/README.md`; `pooled-scan/results/report.txt` lines 7–13; `pooled-scan.tsv` |
| K3 peak RSS `P/none` (PASS, ≤ 1.10 at 6–10): `1.010` (1W), `1.003` (6W), `0.988` (7W), `1.007` (8W), `0.990` (9W), `1.014` (10W) — within 1.4% | `[MEASURED]` | `pooled-scan/results/README.md`; `pooled-scan/results/report.txt` lines 15–22; `pooled-scan.tsv` |
| K4 finder (PASS, ≤ 10 ms): median **4.82 ms [4.81, 5.00]** (order statistics `[2, 10]`, 98.83% coverage), **9.842 GB/s**, AVX2 true, CPU 4, `1,724` block magics and `48` end magics over `47,419,732` B | `[MEASURED]` | `pooled-scan/results/README.md`; `pooled-scan/results/report.txt` lines 24–27; `phases.tsv` line 9 |
| Pooled-scan median walls (ms): 8W `none 192.9`, `P 207.7` (block mean `207.5 [206.5, 208.6]` vs forecast `172–175`), `C 183.2`, `PC 158.1`, `SC 135.3`, `S 184.9`; 9W `none 195.3`, `P 196.7` (block mean `195.2 [192.3, 198.1]`), `C 184.2`, `PC 159.9`, `SC 140.2`, `S 174.5`; 10W `none 197.6`, `P 187.2`, `C 187.2`, `PC 161.0`, `SC 147.0`, `S 169.6` | `[MEASURED]` | `pooled-scan/results/report.txt` lines 31–67; `pooled-scan.tsv` lines 55–56 |
| Pooled-scan traces (`A2+A3`, idle share, chain plumbing): workers' summed scan CPU in `P` = `172.9–192.7 ms` (`177.7` at 8W, `182.8` at 9W) vs serial scanner in `none` = `170.6–189.6 ms` (`177.5` at 8W, `173.4` at 9W); worker idle share (`job_wait/life`) falls from `6.5–20.2%` (`none`, 8–10W) to `1.2–2.5%` (`P`); `pool_copy` copies `45.2 MiB` in `4.32–7.24 ms` (estimated ~5 ms), removing `190.6 MiB` queued symbols; `pool_find` `4.97–7.35 ms` | `[MEASURED]` | `pooled-scan/results/README.md`; `pooled-scan/results/report.txt` lines 69–89; `pooled-scan.tsv` lines 57–91 |
| Queueing arithmetic check: break-even $k^* = 1 + w/s \approx 8.7$ (using measured knee $w/s \approx 7.67$ from `2026-11-25-the-decoder-as-a-queueing-network.md`); behind ideal `C`, `SC + s/k` gives `135.3 + 178/8 = 157.6 ms` (measured `PC = 158.1 ms`) at 8W and `140.2 + 178/9 = 160.0 ms` (measured `PC = 159.9 ms`) at 9W | `[CALCULATED]` | Derived directly from measured `SC`, `A2+A3` medians in `pooled-scan/results/report.txt` and knee in `2026-11-25-the-decoder-as-a-queueing-network.md` |
