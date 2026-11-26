# A Decisive Cross-Reading of *A Thousand Plateaus* Against `mibz2`

*26 November 2026. Stage 2 of [`05-prompt-deleuze-account.md`](05-prompt-deleuze-account.md), built directly on [`deleuze-guattari-account.md`](deleuze-guattari-account.md) and grounded in `origin/master` (`a3824a2`), PR `#938` (`docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md`), PR `#933` (`research/rauzy-census`), PR `#935` (`leads/pre-port-tests`), `docs/team/`, `docs/journal/2026-11-24-algorithm-level/`, `docs/site/blog/conversations/2026-11-22-thomas-murphy.html`, and `src/open.rs`. Reading and analytical writing only; no code modified.*

**Standard of judgement:** Every concept from *A Thousand Plateaus* (`ATP`) receives exactly one verdict:
- **CONSEQUENCE:** specifies a concrete change, the exact file or process it touches, and a falsifiable criterion for knowing whether it worked. Analogy without an operational claim does not qualify.
- **DIAGNOSIS ONLY:** explains an observed phenomenon in `mibz2` with primary evidence, but recommends no change (either because the mechanism is already working or because the concept adds no actionable lever).
- **NO BEARING:** stated in one line.

---

## 1. Mapping `mibz2` onto *A Thousand Plateaus*

### 1.1 Team machinery

| `mibz2` component | Concrete repository locus | Mapping onto *ATP* |
|---|---|---|
| **The Coordinator** | `CLAUDE.md` (225 lines), `docs/team/ROLES.md`, `notes/coordinator.md`, `2026-11-26-rhizome/README.md` §§1–3 | **Split assemblage (two opposite functions glued together):** (1) as prompt-writer and `#145` hub, an $n+1$ overcoding centre (Plateau 1), architect-on-paper splitting intellectual from manual labour (Plateau 12), and Power-from-impotence rule-patcher (Plateau 9); (2) as steward of CI, head-SHA checks, workstation queue exclusivity, and releases, a load-bearing verification stratum (Plateau 6) and organising refrain (Plateau 11). |
| **Prompts and briefs** | `ideal-parts/antigravity/prompt-*.md`, `rauzy-census/prompt.md`, `briefs/*.md` (stamped *"Given to Louis to paste"*) | **Order-words executing incorporeal transformations via indirect discourse** (Plateau 4). Every bespoke Coordinator brief pre-selects presuppositions, induces literal compliance over premise-checking, and injects relay defects: wrong base branch in `prompt-build-zero-copy.md` (requiring `#916`), inline slash headers causing six unparseable Neon comments (`7141393178`–`7147835505`), blog brief violating `write-blog-post` (`#920` → `#927`), and withheld pre-port tests triggering Louis's *"making me micromanage when i asked for autonomy"* rebuke (`notes/coordinator.md` L154). |
| **`#145` channel & routing** | PR `#145` (2,269 comments), 7 role inboxes (`#160`/`#161`/`#162`/`#222`/`#314`/`#346`/`#363`), `.github/coordination/coord.py`, `.github/workflows/channel-routing.yml`, `PROTOCOL.md` | **Despotic signifying regime ("interpretosis") + fossilised strata** (Plateaus 3 & 5). `coord.py` L155 hardcodes *"Only the coordinator writes in an inbox"*; 124 of the last 269 `#145` comments (46.1%) are Coordinator `LOG`/`QUEUED`/`CORRECTION` prose; four inboxed roles (`#160`, `#161`, `#222`, `#363`) posted **zero** comments on `#145` during 22–26 Nov yet remain wired into active routing checks. Meanwhile `MODE.md` / `MODE.RATIONALE.md` (702 lines, `tools/mode_doctest.py`) is **restratified faciality** (Plateaus 1, 7): Deleuze's own *"He Stuttered"* recaptured into a 16-term glossary and 8-question conformance rubric. |
| **Registrations** | `RULES.md` C1/C1a/C2, `ideal-parts/registration.md`, `pooled-scan/registration.md`, `rauzy-census/README.md` | **Protective striation & warding-off mechanism** (Plateaus 6, 13, 14). Freezing decision rules *before* data wards off post-hoc capture and self-serving reinterpretation—proven operationally by PR `#933` (Rauzy census STOP at 1.41% < 3% gate) and PR `#935` (Network Calculus & `uiCA` KILLs). |
| **CI and its checks** | `.github/workflows/CI.yml`, `miri.yml`, `test.yml`, `bench.yml`, `no-ai-attribution.yml` | **Symmetric immanent verification stratum** (Plateaus 5 & 6). Enforces byte-exactness (`B1`), `unsafe`/Miri (`B5`/`B7`), and zero AI attribution (`E4`) directly on code and commit metadata without an $n+1$ narrator—the exact plane on which lateral work can be accepted without trusting its author (`README.md` §3). |
| **The `evidence` branch** | `RULES.md` B3/C5, `just submit`, SHA-256 bundles (`7dd7b070`, `cc8ba137`, `e432a2ba`) | **Lossless material content stratum** (Plateau 3). Directly answers Louis's warning in `2026-11-22-thomas-murphy.html` (*"you want to use repo filesystems for lossless transfer / if you use messaging they always summarise and give an impoverished precis"*): raw primary measurements on disk that cold auditors can re-read without passing through indirect discourse. |

### 1.2 Methods

| `mibz2` method | Concrete repository locus | Mapping onto *ATP* |
|---|---|---|
| **Ideal-parts characterisation & zero-copy floor** | `ideal-parts/results/full/README.md` (2,492 decodes), `ideal-parts/results/floor/README.md` (648 decodes, 16 min, PR `#921`, journal §92) | **Constructed plane of consistency + Riemannian local-patch mapping + deterritorialisation theorem** (Plateaus 1, 10, 14, Conclusion). Supplying pre-computed zero-copy answers lifts the decoder out of stratified computational weight so every stage's unmasked potential (`C = −42%` at 1W; `E_consume = 298 ms` at 1W; `D = 48 ms` serial; `plumbing = 5.3 ms`) is visible across local worker regimes ($k \in \{1, 6, 8, 10\}$). The flagship `S × C` result (`S` alone −3%, `C` alone −6%, `S+C` −36% at 8W) is a literal operational instance of *ATP*'s deterritorialisation theorem (*"one never deterritorializes alone"*, p. 508): speeding up `S` or `C` alone immediately reterritorialises onto the slower coupled station. |
| **Hybrid-system model** | `reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md`, `external/chatgpt-hybrid-systems-review.md` (journal §70) | **Borderline interaction between nomad & royal science** (Plateau 12, p. 367) + **Hjelmslevian content/expression check** (Plateau 3). Models the decoder as $\mathcal{H} = (C, F, D, G)$ combining continuous service work (flow) with discrete lock-guarded admission/ownership transitions (jumps) in hybrid time $(t, j)$. Crucially, the Coordinator's first draft erred on formal royal-science grounds (calling the lower bound $L = \max(S, W/k, R)$ an upper completion certificate; misreading logged sequence numbers `q_*` as cumulative counts); cold ChatGPT Pro review caught every error and separated $L$ (prune), $\hat{T}$ (rank), and $U$ (guarantee). |

### 1.3 Mathematics

| `mibz2` mathematics | Concrete repository locus | Mapping onto *ATP* |
|---|---|---|
| **Rauzy–Veech induction & Open's construction** | `src/open.rs`, `src/open/endpoint.rs`, PR `#933` (`research/rauzy-census`) | **Double articulation (Plateau 3) + metallurgical following vs. royal over-porting (Plateau 12).** `Kernel::construct` doubly articulates a BWT column: A6a selects and splits maximal runs into thin retained rows vs. thick eliminated strips (*first articulation / sedimentation*); A6b–A6c eliminates the auxiliary interval exchange into a word dictionary and XOR-linked retained edges (*second articulation / functional folding*). Inside `endpoint.rs`, W1's arena reuse (storing elimination nodes in `edges`'s own `u32` buffer before `edges` overwrites them, switching `Narrow`/`Wide` strides at $2^{16}$) is textbook **metallurgical following of material constraints**. Conversely, Thread 2's hypothesis to cut A6b induction steps (`1.812 → 1.279`) via a 4–6k-LOC `surface_dynamics` port was **royal theorem-first speculation**, decisively killed on the ground-level plane by PR `#933` (`1.41%` zero-overhead wall ceiling; every rival induction order `1.014×–17.4×` slower on the clock). |
| **Contraction and enrichment** | `docs/journal/2026-11-24-algorithm-level/essence.md`, `council/comparison.md`, `external/against-mathematics-chatgpt.md` | **Mutational abstract machine (Plateau 5 & Conclusion) + escaping faciality (Plateau 7) + transcoding across milieus (Plateau 11).** *Contraction along composition* (*"Change what 'moving together' means"*) breaks out of the faciality of a single representation (`G1`'s rigid-interval scalar $c_1 \ge h_1$, which had overcoded the search space until journal §42 delimited its scope). *Enrichment along composition* (algebraic lifting like AD's dual numbers $(a, a')$, sharpened to ChatGPT's question: *"which later operation is necessary only because an earlier stage failed to retain information it already encountered?"*) is **transcoding**—letting an existing pass (`A5` MTF/RLE2, `A6a` maximal-run scan) emit richer expressive traits so a downstream pass (`127`-cap re-scan or `E_consume` word spelling) dissolves. |

---

## 2. Concept-by-Concept Verdicts Across All Fifteen Plateaus

Every major concept covered in [`deleuze-guattari-account.md`](deleuze-guattari-account.md) is assigned one strict verdict below. Consequence IDs (`C1`–`C7`) refer to the ranked list in §5.

| Plateau | Concept | Verdict | Basis (action, evidence, or one-line dismissal) |
|---|---|---|---|
| **1. Rhizome** | Connection & heterogeneity | **CONSEQUENCE (`C5`)** | Stop cloning homogeneous workers; wire heterogeneous model-harness peers laterally by empirical strength (`docs/team/`, `frontier/`). |
| **1. Rhizome** | Multiplicity, flatness, $n - 1$, overcoding | **CONSEQUENCE (`C1`)** | Subtract the $n+1$ prompt-writing Coordinator role and `#145` prose routing while keeping symmetric verification (`coord.py`, `ideal-parts/antigravity/prompt-*.md`). |
| **1. Rhizome** | Asignifying rupture | **CONSEQUENCE (`C2`)** | Enforce hard fresh-context restarts with ≤40-line lossless deltas across Coordinator, Pathfinder, MONODROMY, and Neon sessions. |
| **1. Rhizome** | Map vs. tracing; plug tracings into map | **CONSEQUENCE (`C3`)** | Replace hand-edited prose status summaries with auto-generated `frontier/surface.tsv` updated from `evidence` bundles after every major merge. |
| **1. Rhizome** | Speed turning points into lines (Glenn Gould) | **DIAGNOSIS ONLY** | Explains how the 16-minute zero-copy floor (PR `#921`) collapsed four point-debates (`plumbing`, 3rd-station memory, Rauzy order, `E_consume`) in 48 hours; suggests no separate change beyond `C3`. |
| **2. Wolves** | Pack vs. mass (molecular vs. molar multiplicities) | **CONSEQUENCE (`C5`)** | Scope Antigravity `/teamwork` strictly to bounded read-only falsification packs on $\ge 10\%$-room targets; ban homogeneous worker swarms on closed scalar edits. |
| **2. Wolves** | Critique of Freud's Oedipal Wolf-Man reduction | **NO BEARING** | Clinical psychoanalysis of familial neurosis does not bear on software engineering or BWT inversion. |
| **3. Geology** | Strata & double articulation | **DIAGNOSIS ONLY** | Accurately describes `src/open.rs` (A6a sedimentation → A6b–c folding) and `docs/team/`'s three historical layers; descriptive only. |
| **3. Geology** | Content vs. expression (Hjelmslev) | **CONSEQUENCE (`C3`)** | Enforce strict separation between primary content (`evidence` bundles, raw TSVs, commits) and prose expression; cold audits read content only. |
| **3. Geology** | Epistrata, parastrata, Ecumenon / Planomenon | **NO BEARING** | Geological taxonomy adds vocabulary without adding an engineering lever beyond strata and content/expression. |
| **4. Linguistics** | Order-words & incorporeal transformation | **CONSEQUENCE (`C1`)** | Eliminate 30–105 line Coordinator prose briefs; replace with minimal 12-line machine-checked task specs + raw repo pointers so prompts act as pass-words rather than compliance scripts. |
| **4. Linguistics** | Indirect discourse & relay distortion | **CONSEQUENCE (`C1`, `C4`)** | Ban Coordinator-summarised briefs to Neon and ChatGPT Pro; cold auditors and peers must read pinned SHAs and raw TSVs directly. |
| **4. Linguistics** | Minor language / stuttering (`MODE.md`) | **CONSEQUENCE (`C6`)** | Retire `MODE.RATIONALE.md` (702 lines) and `tools/mode_doctest.py` to `docs/team/archive/`; keep a ≤15-line epistemic tagging block. |
| **5. Regimes of Signs** | Signifying regime (despotic centre, interpretosis) | **CONSEQUENCE (`C1`)** | End manual `#145` prose commentary (`124/269` Coordinator `LOG`/`QUEUED` messages); switch `#145` to 3-line machine events from git pushes. |
| **5. Regimes of Signs** | Postsignifying regime (subjectification / self-policing) | **DIAGNOSIS ONLY** | Explains self-policing rule accretion in `CLAUDE.md` (225 lines of incident patches) and compliance-over-truth behaviour; remediation sits under `C7`. |
| **5. Regimes of Signs** | Diagrammatic semiotics (notation operating on the real) | **DIAGNOSIS ONLY** | Already instantiated in `src/ideal/`, raw TSVs, and `just submit` SHA-256 bundles; no new mechanism needed beyond `C3`. |
| **5. Regimes of Signs** | Presignifying & countersignifying historical regimes | **NO BEARING** | Archaic polyvocal ritual and steppe decimal warfare add no operational claim for a software repository. |
| **6. Body without Organs** | Caution / prudence against too-sudden destratification | **CONSEQUENCE (`C1`, `C7`)** | Do **not** abolish the Coordinator's merge/CI/queue stewardship overnight; keep verification strata intact, migrate in staged reversible steps gated by the 3-arm empirical test. |
| **6. Body without Organs** | Full vs. empty vs. cancerous BwO | **DIAGNOSIS ONLY** | Names the exact two failure modes of reckless flattening (empty BwO = silent regressions/orphaned PRs; cancerous BwO = runaway duplicate token burn); operationalised via `C7`. |
| **6. Body without Organs** | Artaud / masochist / drug-user clinical cases | **NO BEARING** | Bodily and psychiatric experiments have no engineering counterpart here. |
| **7. Faciality** | White wall / black hole; binary normal/deviant triage | **DIAGNOSIS ONLY** | Explains both role-persona theatre (`Optimiser`/`Guardian`/`MODE.md` voice) and how incumbent interfaces (`127`-cap runs, `G1`'s $h_1$) make rival mechanisms appear "deviant"; addressed operationally by `C4` and `C6`. |
| **7. Faciality** | Probe-heads (*têtes chercheuses*) | **DIAGNOSIS ONLY** | Already operationalised by zero-copy ideal-parts probes (`src/ideal/`). |
| **7. Faciality** | Christian / White-Man genealogy of the Face | **NO BEARING** | Cultural-historical genealogy of European portraiture and racialisation does not bear on decoder optimisation. |
| **8. Three Novellas** | Molar break vs. molecular crack (*"What happened?"*) | **DIAGNOSIS ONLY** | Accurately describes context staleness and hardware cliffs (PR `#870`'s 24→48 KiB L1d spill); already captured by `C2` and `C4`. |
| **8. Three Novellas** | Literary genre contrast (novella vs. tale) | **NO BEARING** | Narrative taxonomy of 19th-century fiction has no bearing on team workflow or mathematics. |
| **9. Micropolitics** | Rigid, supple, and flight lines | **DIAGNOSIS ONLY** | Useful framing for mixing rigid safety gates with supple probes, with no standalone change beyond `C7`. |
| **9. Micropolitics** | Four dangers (Fear, Clarity/microfascism, Power-from-impotence, Abolition) | **CONSEQUENCE (`C7`)** | Impose hard size/rule caps on `CLAUDE.md` (≤120 lines) and `frontier/check.py` (≤150 LOC / ≤5 checks) and enforce `OWNERSHIP.md` §5's ≤20% governance-commit ratio so decentralisation neither drifts into peer microfascism nor rule-patching bloat. |
| **10. Becoming-Intense** | Haecceity (longitude = speed; latitude = affects) | **CONSEQUENCE (`C5`)** | Allocate peers by measured speed/error/tool profiles (Neon = probes/oracles, Claude = Rust/CI/integration, ChatGPT Pro = cold mathematical referee) and classify algorithms by hardware affect profile rather than math taxonomy. |
| **10. Becoming-Intense** | Plane of consistency vs. plane of organization | **DIAGNOSIS ONLY** | Explains why the ideal-parts oracle surface outperformed off-site roadmaps; already actioned in `C3`. |
| **10. Becoming-Intense** | Becoming-imperceptible | **DIAGNOSIS ONLY** | Mirrors `no-ai-attribution.yml` and de-egoised commits already enforced on `master` (PR `#929`). |
| **10. Becoming-Intense** | Becoming-animal / becoming-woman / sorcery / contagion | **NO BEARING** | Aesthetic and political-ontological concepts of identity dissolution have no concrete operational referent in `mibz2`. |
| **11. Of the Refrain** | Territory, refrain as organising routine, consistency | **CONSEQUENCE (`C3`, `C7`)** | Keep lean, repeatable organising refrains (17-minute zero-copy redraw loop, unattended `just workstation-queue`, weekly governance pruning) while preventing closure into ritual. |
| **11. Of the Refrain** | Rhythm (unequal transcoding) vs. meter (fixed cadence) | **DIAGNOSIS ONLY** | Explains why pooling the scan (`pooled-scan/registration.md`) outperforms lockstep serial scanning behind a single thread; implementation already underway in PR `#932`. |
| **11. Of the Refrain** | Cosmic extra-assemblage / Romantic birdsong aesthetics | **NO BEARING** | Musicological and cosmological motifs add no engineering consequence. |
| **12. Nomadology** | State apparatus vs. war machine; architect vs. journeyman | **CONSEQUENCE (`C1`)** | Dissolve the off-site architect-on-paper split (Coordinator writing Neon prompts) so builders confront code and hardware directly on site. |
| **12. Nomadology** | **Royal (theorematic) vs. nomad (problematic/ambulant) science; metallurgy & machinic phylum** | **CONSEQUENCE (`C4`)** | Mandate a ≤1-day ground-level material/corpus singularity probe *before* any multi-day theorematic search or software port, keeping royal science (cold ChatGPT audits, B5/Miri, frozen stop tests) on the borderline as falsifier and certifier. |
| **12. Nomadology** | State capture of the war machine | **DIAGNOSIS ONLY** | Explains how `MODE.md` and Antigravity `/teamwork` were recaptured into top-down orchestration; mitigated by `C1`, `C5`, `C7`. |
| **12. Nomadology** | Tool vs. weapon | **NO BEARING** | In software/hardware engineering, the work vs. free-action distinction reduces without remainder to overhead elimination and nomad science (`C4`). |
| **13. Apparatus of Capture** | Stock, comparison, overcoding by measure | **CONSEQUENCE (`C4`, `C7`)** | Ban single-scalar overcoding ($h_1$ or static $I_r$) as a search generator, and exempt read-only exploratory probes from `frontier/claims/*.toml` so the ledger does not become a metered "stock" capturing free inquiry. |
| **13. Apparatus of Capture** | Capitalist axiomatic & undecidable propositions | **CONSEQUENCE (`C7`)** | Freeze `frontier/check.py` against axiom creep (≤5 mechanical checks) and treat model-vs.-measurement contradictions (undecidable anomalies) as mandatory problem-reformulation triggers rather than rule patches. |
| **13. Apparatus of Capture** | Archaic *Urstaat*, ground rent, fiscal origin of money | **NO BEARING** | Prehistoric state-formation and economic-history theses do not translate into software team mechanics. |
| **14. Smooth & Striated** | Mathematical model (Riemannian local patches vs. Euclidean global grid) | **CONSEQUENCE (`C4`)** | Abandon global uniform models (strict worst-case Network Calculus across 3,500× block variance; L1-only straight-line `uiCA` — both killed in PR `#935`) in favour of local-patch regime models split by worker count $k$ and block class. |
| **14. Smooth & Striated** | Technological (fabric/felt) & Maritime models | **DIAGNOSIS ONLY** | Clarifies why `frontier/` intentionally striates write-ownership and stop tests (latitude/longitude) to enable safe smooth exploration inside each patch; operationalised in `C7`. |
| **14. Smooth & Striated** | *"Never believe that a smooth space will suffice to save us"* (p. 500) | **CONSEQUENCE (`C7`)** | Gate permanent adoption of any flat topology on PR `#938` §3.4's matched-token 3-arm empirical test (and accept restarted-solo fallback if $\lambda < 1$ persists). |
| **14. Smooth & Striated** | Musical (non-pulsed time) & Aesthetic (optical/haptic) models | **NO BEARING** | Acoustic and art-historical models carry no operational claim for BWT decoding or GitHub workflows. |
| **15. Conclusion** | Deterritorialisation theorems (*"one never deterritorializes alone"*) | **CONSEQUENCE (`C3`)** | Codify coupled-oracle testing (`PC/C`, `SC_cand/S`, `E_cand/(S+C)`) as mandatory whenever `surface.tsv` flags an `OPEN_PAIR` / `OPEN_COUPLED` bottleneck. |
| **15. Conclusion** | Tetravalent assemblage & mutational abstract machine | **DIAGNOSIS ONLY** | Provides a clean retrospective audit lens across content/expression and stabilising/mutational axes; no separate file change beyond `C1`–`C7`. |

---

## 3. Stress-Testing PR `#938`'s Redesign Against the Whole Book

The first cross-reading (`PR #938`, `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md`) worked almost exclusively from the Introduction's Principles 3 and 4 plus Paradigm 3's empirical study. Reading PR `#938`'s concrete redesign against **all fifteen plateaus** reveals three serious blind spots and one major unexploited opportunity.

### 3.1 Does `frontier/` risk re-capture by an apparatus? (Plateau 13 + Plateau 9)
**Yes—specifically by a *capitalist axiomatic* and a *stock*, even after abolishing the archaic despotic Coordinator.**

PR `#938` celebrates eliminating `coord.py` line 155 (*"Only the coordinator writes in an inbox"*) and replacing it with `frontier/claims/*.toml`, `ttl_hours`, `token_budget_weighted`, `owned_paths`, `deltas/*.md` (≤40 lines), `audits/*.toml` (`auditor_family != author_family`), and CI-enforced `frontier/check.py`. Read through **Plateau 13 (Apparatus of Capture)**, this trades *archaic overcoding* (one central sovereign hub) for an *immanent axiomatic* (distributed protocol control—exactly what Alexander Galloway 2004 warns follows decentralisation):

1. **The Stock trap:** If *every* exploratory probe, reading note, or mathematical intuition must first be registered as a metered `claims/<id>.toml` ticket with a `ttl_hours` lease, a `token_budget_weighted` quota, and a rigid `deltas/<id>.md` template, then `frontier/` becomes a central **stock** converting free mathematical inquiry into metered, comparable "work" (*ATP*, pp. 440–442).
2. **Axiom creep:** Remember how `CLAUDE.md` grew to 225 lines—each single operational mishap added a new rule. Without a hard cap, `frontier/check.py` will follow the exact same capitalist-axiomatic dynamic (*ATP*, pp. 462–464): whenever an edge case leaks, someone adds a sixth, seventh, fifteenth CI validator until agents spend 30% of their budget fighting TOML schema lints.
3. **Microfascism (Plateau 9's Danger of Clarity):** Requiring `auditor_family != author_family` and strict ≤40-line delta formatting across all claims invites decentralised peer pedantry—miniature black holes policing schema conformance rather than machine reality (*ATP*, pp. 214, 228).

**Correction required (`C7`):** Split `mibz2` into **two explicit zones**:
- **Smooth zone (read-only exploration, math notes, offline censuses touching `docs/` or `examples/` only):** Zero `claims/*.toml` required, zero `ttl_hours`, zero `owned_paths` lock, zero mandatory cross-family audit before commit.
- **Striated zone (`src/`, `bench/`, `workstation-queue.txt`, or adoption registrations):** Uses `frontier/claims/*.toml` + `frontier/check.py`, with `check.py` permanently capped at **≤150 LOC and ≤5 checks** (path disjointness, valid `base_sha`, frozen stop test, B5/Miri gate, rolling ≤20% governance-commit warning).

### 3.2 Is `frontier/` a striation of a smooth space? (Plateau 14)
**Yes—and where it striates *interfaces and ownership*, that striation is essential rather than a contradiction.**

A naive reading of Principles 3–4 treats any grid or rule as bad overcoding. **Plateau 14** refutes that outright: smooth and striated exist only in mixture, and *"Never believe that a smooth space will suffice to save us"* (*ATP*, p. 500). In the Maritime model (*ATP*, pp. 478–482), latitude and longitude striate the ocean precisely so vessels can navigate without colliding or losing their bearings.

In `mibz2`, PR `#938`'s `owned_paths` disjointness, pinned `base_sha`, auto-generated `surface.tsv`, and frozen C2 stop tests are **protective striations**:
- They stop parallel agents from stepping on each other's toes ($\lambda < 1$);
- They prevent base-branch defects (`#916`) and post-hoc gate relaxation (`RULES C2`);
- They establish a shared coordinate grid against which *inside each disjoint patch* an agent can explore smoothly.

Where PR `#938` goes wrong is **striating the wrong side of the boundary**—forcing uniform 40-line delta templates and mandatory claim leases onto open-ended mathematical exploration, while simultaneously *under-specifying* who owns the remaining striated cross-cutting duties (below).

### 3.3 Does the BwO plateau's caution apply to retiring the Coordinator? (Plateau 6)
**Yes—decisively. PR `#938`'s headline recommendation ("Retire the prompt-writing Coordinator role") is too abrupt unless decomposed into two separable functions and migrated in small doses.**

Plateau 6 (*ATP*, p. 161) warns that *"staying stratified—organized, signified, subjected—is not the worst that can happen; the worst that can happen is if you throw the strata into demented or suicidal collapse."* In `mibz2`, the Coordinator currently bundles **two distinct strata** (already noted by the Coordinator itself in `2026-11-26-rhizome/README.md` §3):

1. **The overcoding/routing layer (safe to dismantle immediately):** Authoring 100-line Neon prompt files, posting 107 manual `LOG`/`QUEUED` messages on `#145`, acting as Louis's copy-paste relay, and writing prose summaries of external reviews.
2. **The verification/integration/release stratum (dangerous to destratify prematurely):** Checking exact head SHAs before merging, arbitrating `DISPUTED` audits, enforcing `RULES D` (especially releases-while-absent), keeping `plan.md` reconciled as threads close (note that right now PRs `#932`, `#933`, `#936`, and `#938` are all open simultaneously waiting on review/merge!), and keeping one heavy workstation job in flight at a time (`RULES E4`).

Retiring the Coordinator wholesale *before* `frontier/` has survived empirical testing risks creating an **empty BwO** (open PRs rot unmerged, `plan.md` goes stale, conflicting workstation queue submissions collide) or a **cancerous BwO** (fresh sessions repeatedly claiming and re-litigating the same disputed anomaly).

**Correction required (`C1`):** Apply Plateau 6's exact method of **prudence**:
- **Step 1 (immediate, low-risk rupture):** Switch the Coordinator itself from one giant compacted multi-day session to **fresh-context sessions bootstrapped from `plan.md` + `frontier/surface.tsv`**, and immediately stop writing bespoke Neon prose prompts or manual `#145` `LOG` prose.
- **Step 2 (empirical trial):** Run PR `#938` §3.4's matched 6M-token three-arm experiment on `worker-frontier-exp` while keeping the Coordinator's merge/release/B5 stewardship active on `master`.
- **Step 3 (conditional retirement):** Dissolve the remaining Coordinator routing apparatus (`.github/coordination/coord.py`, the 7 inbox PRs) *only* if Arm 3 meets its frozen adoption rule—or fall back cleanly to **fresh-context restarted solo + cold ChatGPT Pro audit** if Arm 2 wins.

### 3.4 What would "nomad science" ask of the mathematics programme that "royal science" doesn't? (Plateau 12 + Plateau 14)
This is the sharpest gap in PR `#938`: it focused heavily on team topology and barely interrogated how `mibz2` conducts **mathematics**.

Between 20 and 26 November, `mibz2`'s mathematics programme repeatedly defaulted to **royal (theorematic) science** (*ATP*, pp. 361–374):
- **The 20–22 Nov "kitchen sink"** (`METHOD.md` §1) imported ~25 abstract mathematical frameworks (Krohn–Rhodes, $K_0$ of AF algebras, cluster algebras, braids, operads) from off-site theory onto the decoder—producing 7 parallel non-walk prototypes that **all lost 3.1×–15.0× natively** (`2026-11-22-neon-teamwork-paradigm.md`).
- **Rigid-interval calculus ($c_1 \ge h_1$)** elevated one formal abstraction into a universal floor, nearly closing off representation changes until `external/against-mathematics-chatgpt.md` dismantled the overcoding.
- **Three research-software port candidates** (`surface_dynamics`, Network Calculus `Nancy`/`Panco`, and `uiCA`) tried to import global royal-science machinery—strict worst-case deterministic service curves across blocks whose service times span **3,500×**, L1-only straight-line cycle simulators blind to Comet Lake's **32 KiB L1d cache spill**, and global Rauzy induction order optimisation. Every single one was **killed** by ground-level testing (PR `#933`: 1.41% ceiling; PR `#935`: NC 3.21–3.47× and blind to $k=8$; `uiCA` −28.8% error and wrong sign on PR `#870`).

What **nomad (ambulant / metallurgical) science** (*ATP*, pp. 364–374, 408–415) asks of `mibz2`'s mathematics programme instead:

1. **Follow the material singularities first (the machinic phylum of silicon + bzip2 corpus):** Start every mathematical investigation *on site*, from the actual thrusts already exposed by the zero-copy floor (journal §92) and hardware counters:
   - `E_consume` word spelling (`298.2 ms` at 1W = 22% of wall; `37.2 ms` at 8W), which the first matrix did not even see;
   - `A5 + A6a` throwing away maximal-run boundaries at the arbitrary `127`-cap interface (`src/runs.rs` → `src/open.rs`);
   - `D` serial copy-out (`47.3–49.3 ms` flat across all $k$, 839 MB at ~17 GB/s), the next immovable wall once `S+C` drops;
   - Comet Lake's 32 KiB L1d cliff (which killed PR `#870`'s 48 KiB Huffman tables while `uiCA` predicted −19%/sym speedup) and the 3,500× bimodal split between `open`/`open_dense` and tiny blocks.
2. **Replace Euclidean globalism with Riemannian local patches:** As Louis remarked to Thomas Murphy (`2026-11-22-thomas-murphy.html`), real-world decompression on a fixed, *"not fuckwithable"* format does not have one universal linear proof—it *"stutters"* (*"well it depends"*). Nomad science stops seeking one universal algorithm or global worst-case certificate across all $k$ and all block densities; it constructs **piecewise, regime-local tactics** (e.g., distinct word-spelling paths for short vs. long runs in `E_consume`, distinct $k=1$ vs. $k \ge 8$ scan paths, zero-copy block handoff for `D`).
3. **Operationalise `essence.md` by instrumentation before algebra:** Instead of browsing pure-math textbooks for "semirings" or "algebraic lifts" in the abstract, **follow the grain** of `src/huff.rs`, `src/runs.rs`, and `src/open.rs`: instrument real Wikidata blocks to catalogue *exactly what information each stage already computes and discards* (maximal run lengths, literal bitmaps, short-word frequency histograms, symbol-cluster transitions), and *only then* ask which minimal algebraic enrichment preserves that exact trait cheaply.
4. **Keep royal science on the borderline—as certifier and falsifier, never as generator:** Honoring *ATP* pp. 367 and 374 (*"the ambulant sciences invent problems whose scientific solution depends… on royal science"*), nomad probes pose live hardware/corpus problems in $\le 1$ day, and royal science (cold ChatGPT Pro proofs/audits, Miri, B5 safety arguments, frozen C2 stop tests) verifies correctness, bounds, and escape conditions *after* a material phenomenon is on the table.

---

## 4. Self-Critique: Where Our Mappings Are Weak, and What Survives the Warning Against Instrumentalising Deleuze and Guattari

A decisive cross-reading must attack its own analogies as harshly as it attacks the repository.

### 4.1 Six weaknesses in this cross-reading (and in PR `#938`)

1. **Borrowed terminology masking ordinary systems engineering (the Sokal & Bricmont hazard):** Calling Amdahl's law on `S × C` a *"deterritorialisation theorem"*, calling a piecewise benchmark table over $k \in \{1, 6, 8, 10\}$ *"Riemannian"*, or calling `frontier/surface.tsv` a *"plane of consistency"* adds zero predictive power to Raj Jain's experimental design and operational laws. Every hard numerical discovery in `mibz2` (`S+C = −36%`, `plumbing = 5.3 ms`, `E_consume = 298 ms`, PR `#933`'s 1.41% kill, PR `#935`'s 32 KiB L1d spill diagnosis) came from **controlled physical measurement and systems profiling**, not from reading *A Thousand Plateaus*. If we strip the French vocabulary off `C3` and `C4`, the engineering logic stands completely on its own—which is a virtue for `mibz2`, and a rebuke to any claim that Deleuze *derived* these engineering moves.
2. **DeLanda's realism vs. Buchanan's *agencement* (the "false friend" problem):** Both PR `#938` and §1 above lean heavily on Manuel DeLanda's engineering-friendly translation of *ATP* (2002, 2006, 2016: assemblages as wholes with external relations and parameter knobs). As Ian Buchanan (2015, 2017, 2020) protests, that quietly deletes *desire*—the entire psychoanalytic-political engine of *Capitalism and Schizophrenia*—and treats *agencement* as a parts list. An LLM session has no unconscious desire; treating a git directory as a Deleuzian assemblage is an *engineering adaptation* (closer to Massumi's 1992 "deviations") than faithful philosophy.
3. **LLMs are not embodied journeymen:** In Plateau 12 (and Protevi's notes on apprenticeship in `DR3`), nomad science depends on *corporeal skill*—the smith feeling metal yield, the mason feeling stone thrust. Discontinuous, stateless LLM sessions that terminate every 3M tokens have **no persistent tacit skill** outside what is written to disk. Without durable written artifacts (`src/`, `examples/`, TSVs, short escape-condition deltas), pure "asignifying rupture" would induce amnesia rather than craft.
4. **Over-broadening "order-words":** Calling every Coordinator prompt an "order-word" risks conflating *lossy prose briefs that pre-judge the solution* (bad) with *clear problem boundaries, safety invariants, and exact commit pins* (indispensable). An LLM given zero constraints does not become a creative nomad; it hallucinates scope or re-implements already-tried ideas.
5. **Caricaturing royal science would have prevented `Open`:** `mibz2`'s single greatest algorithmic breakthrough—Astra's open-return interval-exchange decoder (`src/open.rs`, `#565`)—did *not* emerge from low-level hardware tinkering alone. It emerged from importing a genuinely sophisticated royal-mathematical object (Rauzy–Veech induction on interval-exchange transformations, seeded by Thue–Morse words in `2026-11-22-thomas-murphy.html`) and fitting it to BWT runs. Over-correcting into anti-theoretical empiricism would shut down the exact cross-disciplinary leaps (`essence.md`) that produce breakthroughs rather than 2% micro-optimisations.
6. **Performative contradiction (interpretosis inside the critique of interpretosis):** `OWNERSHIP.md` §5 warns that climbing docs-to-`src/` commit ratios signal governance reversion—38 of the last 50 commits on `master` touched `docs/` and only 3 touched `src/`. Producing two long philosophical essays (`deleuze-guattari-account.md` and this file) on top of PR `#938` and `01`–`05` is itself **interpretosis** unless this document closes the meta-reflection phase and shifts work immediately back onto `src/` and the empirical 3-arm test.

### 4.2 The secondary literature's warning against instrumentalising D&G—and what survives it

Multiple major strands of secondary scholarship explicitly warn against turning *A Thousand Plateaus* into an operational management or engineering manual:
- **Eyal Weizman (2006)** documented military commanders using *smooth space*, *rhizome*, and *war machine* as tactical doctrine for urban warfare ("walking through walls"), proving *ATP*'s concepts carry no built-in ethical guarantee.
- **Luc Boltanski & Ève Chiapello (2005)**, **Alexander Galloway (2004)**, **Douglas Spencer (2011)**, and **Andrew Culp (2016)** show that "flat", "rhizomatic", "flexible", and "networked" idioms are the native rhetoric of post-Fordist management and *protocol-based control societies*—often intensifying extraction and compliance while hiding authority inside automated schemas.
- **Sokal & Bricmont (1998)** warn against mistaking suggestive mathematical imagery for rigorous proof or physical models.

**What survives those warnings for `mibz2`?**
Not a mystical belief that "flatness raises the roof" (Paradigm 3's $\lambda \approx 0.6\text{–}0.9$ already refutes naive flatness!), and not a new jargon layer for `docs/team/`. Exactly **four non-trivial structural insights** survive every critique intact:
1. **Indirect-discourse distortion is structural, not accidental (Plateau 4):** Having one LLM translate reality into prose prompts for another LLM systematically injects presuppositions, compliance bias, and relay errors; the fix is structural (raw primary artifacts + minimal stop-test specs), not "write better prompts."
2. **Decentralisation has its own pathologies (Plateaus 6, 9, 13, 14):** Abolishing an $n+1$ hub too abruptly causes collapse (empty/cancerous BwO), and replacing it with an unbounded validator protocol recreates capture as a *capitalist axiomatic* and peer *microfascism*. Keeping a tight two-zone boundary (smooth exploration vs. striated `src/`/workstation gates) with hard caps on rules/LOC is a genuine design improvement over PR `#938`.
3. **The borderline discipline between empirical probing and formal proof (Plateau 12, p. 367):** Neither theorem-first maximalism (which failed on 7 non-walk prototypes and 3 software ports) nor atheoretical tinkering suffices; coupling fast ground-level corpus/hardware singularity probes with cold cross-family formal verification puts both royal and nomad science in their productive positions.
4. **Empirical falsification over philosophical authority (Plateau 14, p. 500):** Because no smooth space suffices to save us, the entire redesign must submit to PR `#938` §3.4's matched-token 3-arm experiment—and we must abandon multi-agent flatness without sentimentality if fresh-context restarted solo wins.

---

## 5. Ranked Consequences (Maximum Seven, Each with Cost and Test)

Ordered by highest expected reduction in wasted compute and highest leverage on `mibz2`'s actual decoder work (`plan.md` threads 3–5).

### 1. (`C1`) Staged retirement of Coordinator prompt-writing & `#145` prose routing, with fresh-context Coordinator restarts
- **Change:** Stop writing bespoke 30–105 line prompt files (`ideal-parts/antigravity/prompt-*.md`, `briefs/*.md`) and manual Coordinator `LOG`/`QUEUED` prose on `#145`. Replace prompts with minimal ≤15-line machine-checked task specs (`base_sha`, raw artifact paths, frozen stop test). Switch the Coordinator itself immediately to **fresh-context sessions bootstrapped from `plan.md` + `surface.tsv`** (`README.md` §1), while **keeping** its merge/CI/B5/workstation-queue stewardship intact until the 3-arm empirical test decides full retirement of `.github/coordination/coord.py` and the 7 inboxes (honoring Plateau 6 caution).
- **Touches:** `ideal-parts/antigravity/`, `docs/team/PROTOCOL.md`, `.github/coordination/coord.py`, `#145` workflow (`frontier-ping.yml`).
- **Cost:** ~0.5 day to add `just frontier-brief` and minimal spec checker; temporary dual-track friction while the 3-arm test runs.
- **Test:** Across the next 5 completed threads (and Arm 3 of the 6M-token test), **0** base-branch/header/brief-relay defects (down from 4 on 26 Nov), human copy-paste relays $H_{\text{relay}} \le 1$ per thread, Coordinator manual `#145` traffic drops by $\ge 80\%$, and **0** unmerged-PR stalls or workstation-queue collisions.

### 2. (`C2`) Hard session rupture with ≤40-line lossless pointer deltas
- **Change:** End multi-day continuous sessions across all roles (Coordinator, Pathfinder, MONODROMY, Neon). Cap every execution session at one atomic task or 3–4M weighted tokens, committing raw artifacts (code, TSVs, JSON) plus a ≤40-line delta (`commit SHA`, `evidence SHA-256`, `verdict`, `measured/proved delta`, `escape condition`), then terminating.
- **Touches:** `docs/team/ROLES.md`, `frontier/deltas/` (or thread folders), session launch prompts.
- **Cost:** ~5–10% cold-start token spend per fresh session re-reading primary files; loss of informal conversational continuity.
- **Test:** Tested directly in PR `#938` §3.4's matched 6M-token experiment on `worker-frontier-exp`: fresh-context sessions must match or exceed continuous-context $Q_{\text{wall}}$ with **0** stale-prior factual errors caught in cold audit.

### 3. (`C3`) Auto-generated `frontier/surface.tsv`, mandatory coupled-oracle kill tests, and the 17-minute zero-copy redraw refrain
- **Change:** Emit `frontier/surface.tsv` mechanically from `evidence` bundles (never hand-summarised in prose). Make paired-oracle kill tests (`PC/C`, `SC_cand/S`, `E_cand/(S+C)` — generalising `pooled-scan/registration.md`) mandatory for every `OPEN_PAIR` / `OPEN_COUPLED` row, and trigger the 16–17 minute `bench-zero-copy-floor` redraw after every `src/` merge that moves shipped 8W wall by $\ge 5\%$.
- **Touches:** `bench/` analysis script, `frontier/surface.tsv`, `docs/journal/2026-11-24-algorithm-level/plan.md`.
- **Cost:** ~100 LOC generator script + 17 minutes of i9 workstation time per major `src/` merge.
- **Test:** Over the next 3 iterations, **0** standalone false-kill gates (like `P/none ≤ 0.93` on a 3.2%-room stage) and **0** `src/` optimisation PRs opened against targets showing $< 5\%$ unmasked room in `surface.tsv`.

### 4. (`C4`) Nomad-first / Royal-second mathematical pipeline (local-patch material census before theory or ports)
- **Change:** Add a binding two-stage gate to `docs/journal/2026-11-24-algorithm-level/essence.md` and `plan.md`: (1) every new mathematical thread, algebraic lift, or external-tool port starts with a $\le 1$-day ground-level empirical census on actual corpus blocks and local hardware regimes ($k \in \{1, 6, 8, 10\}$, dense vs. sparse blocks, L1d footprint, discarded pass information); (2) cold cross-family royal verification (ChatGPT Pro reading raw commits and TSVs directly, never Coordinator summaries) runs at formulation, pre-workstation-design, and post-matrix gates to certify bounds and catch accounting omissions.
- **Touches:** `docs/journal/2026-11-24-algorithm-level/essence.md`, `plan.md`, `docs/team/research-charter/METHOD.md`.
- **Cost:** $\le 1$-day empirical census before multi-day theoretical dives; ~0.5–0.8M tokens per cold ChatGPT Pro gate audit.
- **Test:** Over the next 5 mathematical threads, **0** multi-day builds/ports losing $> 1.2\times$ natively or killed post-implementation (compared to 7 non-walk prototypes on 22 Nov and 3 ports evaluated on 26 Nov), and at least one verified positive whole-decoder speedup on `E_consume` (`298 ms` 1W) or `A5+A6a` (`C` behind `S`) within 2 iterations.

### 5. (`C7`) Two-zone smooth/striated boundary + hard anti-axiomatic & anti-microfascism caps (refining PR `#938`)
- **Change:** Revise PR `#938`'s `frontier/` protocol so read-only exploratory probes and mathematical reading notes (`docs/`, `examples/`) remain **smooth** (no `claims/*.toml` lease, no TTL, no mandatory pre-commit audit), while **striation** (`owned_paths` disjointness, pinned `base_sha`, frozen C2 stop test, B5/Miri) governs `src/`, `bench/`, and `workstation-queue.txt`. Freeze `frontier/check.py` at **≤150 LOC / ≤5 checks**, cap `CLAUDE.md` at **≤120 lines** (one-in, one-out rule deletion), enforce `OWNERSHIP.md` §5's rolling 7-day **≤20% governance-commit** threshold, and gate permanent adoption on PR `#938` §3.4's 3-arm empirical test.
- **Touches:** `CLAUDE.md`, `frontier/check.py`, `docs/team/OWNERSHIP.md`, `docs/team/FRONTIER-PROTOCOL.md`.
- **Cost:** Editorial discipline: fixing tooling or deleting obsolete rules instead of appending prose patches.
- **Test:** After 30 days, `frontier/check.py` $\le 150$ LOC, `CLAUDE.md` $\le 120$ lines, governance/docs-only process commits $\le 20\%$ of total commits, and **0** passing `src/` PRs delayed $> 1$ review round by schema/style objections.

### 6. (`C5`) Ethological peer allocation & strict `/teamwork` scoping (pack over mass)
- **Change:** Archive dormant Astra/Codex role specs (`#160`, `#161`, `#162`, `#222`, `#363`, `WORK-TASKS.md`, `CODEX-*.md`, `COORDINATOR-FAILOVER.md`) under `docs/team/archive/`. Route active work by measured model-harness profile (Neon single sessions = censuses/oracles/prototypes; Claude Code = `src/` integration, `bench/`, Miri/B5, releases; ChatGPT Pro = cold mathematical audits). Restrict Antigravity `/teamwork` *exclusively* to read-only parallel falsification sweeps on targets with $\ge 10\%$ confirmed room in `surface.tsv`.
- **Touches:** `docs/team/`, `.github/coordination/coord.py`.
- **Cost:** ~30 minutes of archival cleanup; restraint against launching `/teamwork` on unmeasured targets.
- **Test:** **0** tokens spent auditing dormant role inboxes; every future `/teamwork` run operates on a $\ge 10\%$-room `surface.tsv` row and either yields a natively faster prototype or closes $\ge 3$ hypotheses in $< 24$ hours.

### 7. (`C6`) Retire the `MODE.md` / `MODE.RATIONALE.md` linguistic bureaucracy
- **Change:** Archive `docs/team/MODE.RATIONALE.md` (702 lines), `docs/team/MODE.ORIGIN.md`, and `tools/mode_doctest.py` under `docs/team/archive/`; collapse `docs/team/MODE.md` into a ≤15-line epistemic discipline block (`[PROVED]` / `[VERIFIED]` / `[MEASURED]` / `[CONJECTURED]`, whole-cost accounting, raw-data-first) printed by `just frontier-brief`.
- **Touches:** `docs/team/MODE.md`, `docs/team/MODE.RATIONALE.md`, `docs/team/MODE.ORIGIN.md`, `tools/mode_doctest.py`.
- **Cost:** ~15 minutes of archival cleanup; retirement of the telegraphic `#145` house voice.
- **Test:** Across the next 10 PRs, no increase in cold-audit defects or overclaims relative to the 24–26 Nov baseline, with >50 KiB of non-operative linguistic governance removed from active team docs.
