# Human-readable comment references
Assignment: [6693597404](https://github.com/lmmx/mibz2/pull/162#issuecomment-6693597404). Proposal only.

Recommend **Oracle-authored descriptive Markdown links, with a short, consistent citation style in supporting material**. No runtime reference table initially. The actor and action belong in the visible text; the identifier belongs in the destination. The existing renderer already supports this. A table would centralise repetition, but introduces a second representation and a cross-version join to solve a small authoring problem.

## Evidence and scope

Inspected:
- Governing [SPEC](https://github.com/lmmx/mibz2/blob/a6a703cefcc40ac4115803f44a8a29ad76a3a7ef/docs/team/SPEC.md), SPEC.RESEARCHER, MODE, RULES, PROTOCOL and [SPEC.ORACLE](https://github.com/lmmx/mibz2/blob/a6a703cefcc40ac4115803f44a8a29ad76a3a7ef/docs/team/SPEC.ORACLE.md) at that pin.
- Site tree at 818d521c6d0d5da09b4b0f34322baaf54cbf39af; [observatory.js](https://github.com/lmmx/mibz2/blob/818d521c6d0d5da09b4b0f34322baaf54cbf39af/docs/site/assets/javascripts/observatory.js), especially render, linkComments, moves, build, start and watch; the page Markdown.
- Oracle record [456fcb96](https://github.com/lmmx/mibz2/tree/456fcb964af5a3f28b33175f134a739fe10b9840): account, weak, changes, manifest and commit message.
- Direct comment reads for [6683965024](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024) and [6693507432](https://github.com/lmmx/mibz2/pull/160#issuecomment-6693507432). Both are posted by GitHub account lmmx with FROM: coordinator. The first is TYPE: ACK, created 6 November 2026 11:39:53 UTC; the second TYPE: CHECKPOINT, created 19:27:09 UTC and addressed TO: optimizer.

This is source inspection and a design assessment, not a browser/accessibility test or implementation. The record version is deliberately fixed; later events do not silently rewrite the examples.

## Counterexamples before choosing

1. **Recipient is not speaker.** The policy brief on #160 is the Coordinator's assignment to the Optimiser, not the Optimiser's report. A label taken from the destination inbox would be wrong. The account's current “assigned the Optimiser” correctly describes the action but leaves the sender implicit.
2. **GitHub author is not the role.** All team roles can post as lmmx (RULES A1). “Louis approved” cannot be inferred from that login, nor authenticated from a FROM header. Use the role declared by the message; make that convention explicit once in the supporting record. Missing or conflicting headers mean “role not established”, not a guessed role.
3. **Message type is not disposition.** The retraction is carried by an ACK, not a DECISION. “Coordinator · ACK” alone hides the consequential action. The gist must say “retracts the 25% prerequisite”; it must not say “reopens the experiment”, because the dispatch pause stayed.
4. **A current lookup cannot label old history.** If a comment is edited from hypothesis to finding, loading today's metadata while displaying yesterday's account rewrites the earlier evidence state. This applies to a table just as much as an API lookup.

## Three directions

| Direction | What the reader sees | Spec/page change | Failure modes and cost | Re-pin |
|---|---|---|---|---|
| A. Oracle-authored inline Markdown | “Coordinator retracts the 25% prerequisite”, as a normal link; short date/type when useful | No renderer change for authored links. Add a concise Oracle authoring convention; retire the automatic numeric link guess separately | Repeated labels may disagree; a vague possessive hides what happened; copied permalink may be wrong. O(R) label bytes for R occurrences. Review the rendered wording and destination at authoring time | A one-off descriptive link already fits the pinned spec. Making the convention a persistent instruction requires an Oracle spec amendment and Louis's re-pin |
| B. Per-version reference table, rendered into chips | “Coordinator · acknowledgement · 6 Nov — retracts the 25% prerequisite”; optional expanded provenance | New record schema, explicit reference tokens, raw-file loader and schema-aware renderer. Oracle supplies the complete display text, not merely fields the browser turns into a new interpretation | Missing keys, stale rows, conflicting prose, accidental use of latest table for old commits, inaccessible hover-only details. O(U) metadata for U unique references plus O(R) tokens and optional local text. One additional raw HTTP fetch, zero additional GitHub REST calls if fetched by already resolved SHA | Required for persistent Oracle schema obligations. Page change needs an ordinary site release once, not a build per record update |
| C. Consistent citation syntax using ordinary Markdown | In levels: “Coordinator — acknowledgement, 6 Nov 2026: retracts the 25% prerequisite”. In prose: actor + verb in the sentence | Authoring rule only; use inline links or file-local Markdown reference definitions. This is a writing style, not a third runtime mechanism | Overlong repeated labels crowd the levels; role/type alone does not distinguish two messages. Shared Markdown definitions do not cross files automatically. Keep the gist; use date/time only as needed | Same as A: no re-pin to write clearer links now; re-pin to make a new rule binding |

**Choose A with C's short convention.** The existing account is already prior art inside this repository: descriptive evidence links work without a table. Reuse that mechanism for the supporting text. A future need for filtering, a shared bibliography or many repeated citations could justify B; none is needed to remove anonymous numbers now. Tooltips may supplement visible attribution, never carry it alone. No code face for human labels.

## Concrete reference style

Supporting detail, first citation or where chronology matters:

[Coordinator — acknowledgement, 6 Nov 2026: retracts the 25% prerequisite](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024).

A second source, with recipient explicit:

[Coordinator — assignment to the Optimiser, 6 Nov 2026: design the three-team policy](https://github.com/lmmx/mibz2/pull/160#issuecomment-6693507432).

“Assignment” here summarises this CHECKPOINT's content. It is Oracle-authored wording, not a universal browser mapping of CHECKPOINT to assignment. Preserve the exact TYPE in provenance when useful. For multiple same-day messages, add UTC time or a more specific gist. Prefer short local labels after the actor is unambiguous.

In the account, keep its ordinary prose rather than inserting the support citation grammar:

“The coordinating agent has [asked the Optimiser to design a production policy](https://github.com/lmmx/mibz2/pull/160#issuecomment-6693507432).”

No forced date, uppercase type, ID or role catalogue in the opening. Name the actor where authorship affects meaning, especially a pause, disputed premise or decision. A link that establishes a result need not turn every sentence into a roll call. A reported instruction from Louis remains “the Coordinator reports Louis's instruction” unless the evidence supports the stronger attribution.

In changes.md, distinguish **version date** from **source-comment date**. Write the transition in words, then label the channel-watermark link by actor/action. In manifest.md retain the exact machine watermark alongside its descriptive citation; it is provenance, not the account.

For new commit messages, put the transition first:
“oracle: confirmation complete; policy design assigned”.
Put the exact watermark in the message body, e.g. “Channel-through: #145 6693504170”. SPEC.ORACLE requires the *message* to repeat it, not the first line. moves() displays only the first line. Thus no renderer change or history rewrite is needed for future subjects. Do not put Markdown link syntax in subjects: moves() treats them as plain text. Old commit subjects stay old; do not relabel them from the newest record.

## Production procedure and fidelity

At rebuild time the Oracle already reads comments. From that observation:
1. Capture the exact source permalink (including repository, PR and anchor), the declared FROM/TYPE, GitHub login, created/updated time and observation time. These are separate facts. Do not infer a source from a ten-digit pattern or an inbox number.
2. Write the label from the observed content. Preserve uncertainty, disagreement, negation and the difference between a statement and an endorsement. A gist is a summary, not a quotation.
3. Put the links and any necessary provenance in the same record commit as the account and levels. For reused citations, maintain consistent attribution; do not force identical contextual wording.
4. Before committing, check actor versus recipient, label versus source, exact destination, date meaning, and agreement across files. Unknown attribution remains visibly unknown. When an edited comment materially changes a claim, correct the next version and say so in changes.md; never change older commits.

No full archival transcript or new evidence gate is proposed. RULES E2 already distinguishes mutable routine comments from committed policy decisions. Exact headers and a short relevant quote may be kept where the attribution is contested. A digest detects a change but cannot reconstruct deleted text. A permalink is an address, not an immutable content snapshot.

If B is adopted later, key records by repository + comment kind + ID, not by ten digits alone. Store declared role separately from posting account; explicit label, canonical URL, exact type, created/updated/observed times and summary provenance. Require explicit reference tokens; unknown token stays unresolved, never gets a fabricated #145 URL. Fetch the table from the exact selected record SHA, also for historical views. If a table is missing at an old commit, show that commit's authored text, not today's labels. Validate consistency at authoring time; the page may check schema and escape text, but cannot invent missing descriptions. Safe links and existing sanitisation remain necessary.

## API budget and the one-version qualification

A and C add **zero** network requests. Ordinary explicit links bypass linkComments(), whose tree walker excludes links, and bypass its click resolver, which only intercepts .obs-comment links. They also work for new tabs and copied links; the current guessed #145 link cannot reliably do that for an inbox comment because modified clicks skip resolution.

B adds at most one raw-file fetch per selected version, not a REST request. It must never resolve each ID in the browser or fetch a separate table per visible chip. It requires a one-time page release; future Oracle commits still require no CI/build/deploy.

The supplied “2 per view, 1 per five minutes” is the normal-path description, not a global upper bound: watch() detecting a change invokes start(), which performs the two API reads again; existing bare-reference clicks make their own call. This proposal adds none.

**A separate existing defect:** start() catches API failure and reads multiple files by the mutable branch name. A branch move between those reads can mix versions; the comment in the source expressly describes that fallback. The page's prose promises everything comes from one version, which this path does not establish. A citation table would inherit and amplify the mismatch.

Minimum correction when implementing the proposal: use a previously resolved immutable SHA for an explicit last-known-version fallback; if none exists, show that a consistent version could not be loaded and link to the repository. Do not present separately fetched branch-name files as settled. No extra API call is necessary. This is a renderer correction for the Coordinator, not a new Oracle authority or a request to redeploy on every record update.

## Adoption and limits

The persistent authoring convention belongs in SPEC.ORACLE and is activated only by Louis's re-pin; the present review changes no pin. MODE and RULES need no change. Researcher's task and state branch remain as pinned. A one-off improvement within the current spec is already permitted; the distinction is between writing a clearer reference and treating an unpinned new schema as compulsory.

Expected benefit: readers identify the speaker and consequential action without a click. Established here: the present renderer accepts the recommended form and needs no enrichment request; two real comments expose sender/recipient and type/action failure modes. No user test or measured usability gain is claimed. The next useful step is a small representative authoring trial, including weak.md, a level, changes, a new subject and an old-version view; table machinery has lower marginal value until that fails.

Migration check passed: old head 1283b969e9e295e268257cd1b6be3ad7da29e6dd equals migrated_from; manifest SHA256 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0 matches. Pinned routing logic run offline on fresh connector snapshots: REVIEW routes to #145; no unresolved routing violations. verify() hard-codes FROM coordinator, so Researcher publication needs the separate exact-body/header/destination check rather than misrepresenting that helper as a pass.
