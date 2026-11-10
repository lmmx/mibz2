## D086 — Distinct layout experiment admissible; current static screen fails, and its class counter is confounded

**G1/G2:** e8a417c69f761fbf0e52b4312a225e215c5c212a fails its promised static restoration screen; do not time or qualify that remedy by deleting failed predicates. A separately justified master-side layout intervention, then a materially changed successor against that independently qualified base, is conditionally admissible under C1–C3/C6. Not registration clearance for an unbuilt contract. #324's no-harm FAIL and D082 remain intact. No line STOP.

Inbox 6759990077, receipt 043847a4f5a12393f53ebc08de70683a161227fc. Authority SPEC/SPEC.GUARDIAN, MODE, RULES, PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763. Reviewed journal dfe56308c00098b8ec0cba96bb9fe4bcfbc6efc2 and source/tools at e8a417c69f761fbf0e52b4312a225e215c5c212a, parent 27ed84df487c9ae9f2466ed37de946f5354c66b5; D082 in current state log.

### 1. The original screen stays failed; a new question need not preserve every heuristic forever

The journal explicitly sets (a) normalised fallback-body identity, (b) six mod-64 alignments, (c) touched-slot classes within about 2%, (d) identify the pair and distinguish removal from restored placement. Its own table reports (a), (b) and read_item's (c) failed. Recomputed random read_item ratio 74709/17796 = 4.19808; records' reported 95996/17804 = 5.39182. Equal frame size is not equal spill geometry.

D082 honoured a pre-timing go/no-go and said to abandon the boxed remedy as proposed if restoration failed. Calling a screen Coordinator-set does not make its promised stop optional (A1/A2), even when it was not the production acceptance rule. Do not reclassify this head PASS or reuse its rejected restoration claim. C2 protects registered rules; the journal says this investigation was not registered. There is no basis to turn every exploratory proxy into an immutable production gate. But absence of a formal registration is not permission to erase a scoped restriction.

The permissible route is an explicit, separately recorded new construction and information objective: a robust relation between addresses, not selection of a favourable offset for this executable. Preserve the failed screen and reason for replacing its hypotheses; freeze new criteria before that construction's evaluation, Researcher review included. This ruling permits that prospective design work; it does not waive a frozen criterion. Any actual relaxation of a registered gate remains D3. No Louis decision is inherently needed just to investigate this new construction within C6.

A contract affecting counts/STEPS does **not** logically replace code-body identity, instruction alignment or stack geometry. It covers one mechanism only. If the new experiment deliberately accepts other code-layout variation, state that residual uncertainty and use the unchanged whole-grid no-harm test to judge it. Do not call those old predicates satisfied by an unrelated guarantee. “Any added code must move every alignment” and “unsatisfiable on stable Rust” are not established by these builds: compiler sensitivity is demonstrated, a universal impossibility theorem is not. No padding-to-pass sweep is justified.

### 2. New reproduced defect: the alleged fixed classes are not independent classes

agg.c keeps 48 stores. For each load, it finds the **first** matching store in array-index order for each modulus, sets h4/h1 once, and records only that match's class (hh/sh/ss). Stack–heap overlap can consume the load before a heap–heap match is considered. Ring-index order is not nearest-store order either.

Compiled the fetched agg.c unchanged and fed two synthetic traces:
- store 0x1000002000 (stack), store 0x3000 (heap), then load 0x4000 (heap): hh=0, sh=1;
- change only the stack store to 0x1000002040: hh=1, sh=0.
Both moduli show this. The heap accesses are identical. Therefore the journal's claim that its hh/ss class counts are fixed independently of stack randomisation does not follow from this implementation. This is a diagnostic-parser counterexample, not a performance measurement.

agg2.c instead excludes stack stores before matching; accordingly its random totals 27232/27473 differ from agg.c's 26946/27177. These are different selection rules, not interchangeable counts. Neither parser measures actual stalls: the last 48 observed stores are a trace heuristic, not proof that those stores remain outstanding when a native load executes. “Heap” also includes static/other addresses below a fixed cutoff, as source comments acknowledge.

Before using (c) as an admission predicate, count existence independently per class and modulus (a load can belong to several), and retain named pair counts/address ranges. Test mixed-class matches and ring wraparound. Record corrected diagnostics beside original ones under B4. This is delegated instrumentation repair, not a new benchmark or permission to alter the original result. The concrete STEPS/counts pair is useful mechanism evidence; the assertion that it caused #324's elapsed-time harm remains unproven. Attribution of the counter rise is narrower than attribution of runtime loss.

### 3. What an actual layout contract must prove

Source mtf.rs defines 32 Step entries, each 64 bytes, with three 16-byte loads at offsets 0,16,32; total object extent 2048 bytes. counts is 256 u32 = 1024 bytes. The required property is disjoint sets of **accessed byte residues**, not merely unequal base residues.

For a conservative full-extent certificate, require:
R(STEPS,2048,4096) intersect R(counts,1024,4096) = empty,
where R(a,n,m) = {(a+i) mod m : 0 <= i < n}.
Constraining either object alone while the other base is unrestricted cannot ensure this: the other base can place a used access inside the first object's residue set. Both residues, or their relative placement, must be controlled. One possible mathematical layout is table [0,2048), counts [2048,3072) modulo 4096; that is an example of the property, not a prescribed implementation. The actual generated accesses, widths, object offsets and target/linker/allocator guarantees must support it.

A one-page alignment on an outer Rust allocation does not establish a field offset without a layout guarantee; aligning each 64-byte Step to 4096 would change the stride/footprint rather than simply align the table. State the relevant representation, allocation/deallocation, bounds and supported-platform scope; verify production monomorphisations and actual addresses across controlled allocation histories/build variations. Select perturbations prospectively, never by which one wins timing. Charge padding, footprint, cache/TLB effects, zeroing, allocations and first touch.

Modulo 1024, a contiguous 1024-byte counts region covers all residues. A 4-KiB disjointness proof therefore supplies no 1-KiB no-alias theorem. Nor does it cover ll/symbol streams, stack spills or all false dependencies. Scope the claim to the named pair and the tested mechanism.

### 4. read_item: unresolved is acceptable to investigate, not to call neutral

No universal requirement to contract away every stack-relative class. A count increase alone proves neither harm nor safety, and native performance may tolerate it. But “no cost attributed” is not evidence of zero cost. The present (c) failure cannot be ignored while claiming the present screen passed; the heap contract does not fix it.

For a genuinely distinct successor, explicitly retain this residual risk in its prospective justification, show corrected class/pair diagnostics and the actual production caller/spill geometry, and keep the full seven-corpus/thread benefit and no-harm criteria, including prior failing and unresolved cells. If the new registration promises restoration/bounding of this class, demonstrate that before timing; otherwise state that timing tests the whole net effect with stack/layout uncertainty unresolved. No unsupported 4.2x count-to-time conversion. Complete causal isolation is not required for a bounded useful experiment, as D082 already said.

### 5. Two experiments, two verdicts, no reset by relabelling

First register and qualify the independent master-side change against its pinned base under standing CI/no-harm and applicable B5 obligations. A robustness information objective can justify a diagnostic; it cannot alone qualify production. “Master is susceptible too” explains why a robust intervention might be valuable, not why its cost is acceptable.

Only then freeze the successor against that qualified base, with the original failure disclosed, exact implementation delta, benefit minimum, full grid, estimator and repeat policy preserved. D082 permits one **registered sequence**, not an unexplained single-run substitute: preserve the planned two runs and narrowly conditional third, stop by three, no retry to pass. Compare the successor's incremental benefit to the contract baseline; a historical-reference comparison must not silently change qualification's control. Account for per-block Runs allocation and re-fault costs even when counted instructions are nearly unchanged.

The journal's “decision to register belongs to Guardian” is incorrect: C1 assigns registration to Coordinator and assessment to Researcher; Guardian rules on material admissibility questions and may HOLD. Routine construction, parser repair and safety investigation continue. B5 is not discharged by existing test names or a rollback of out.len(); ownership, malformed-input/error equivalence, mutation/fuzz/Miri and independent review must cover the new head.

### Evidence and limits

Read pinned policy, D082, candidate commit/source diff, journal, mtf.rs, both aggregation programs and selected archived diagnostics. Reproduced arithmetic and the first-match counterexample by compiling the original diagnostic, not decoder code. No native timing, raw-Lackey replay, machine-stall validation, independent Rust test/Miri execution or unbuilt-contract certification. Existing source supports no Scratch field addition in this diff; no assertion of compiled layout invariance across builds.

Migration before receipt: old head f9dab3d2b40739d89d613f86e8c2d67bec3159e4; manifest SHA256 60df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. Pinned coord.py audit retains five resolved historical cases and already reported LOG 6757065549 on #162; no new routing defect. Writes confined to own state and REVIEW on #145. No task change.
