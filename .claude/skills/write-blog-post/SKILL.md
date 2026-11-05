---
name: write-blog-post
description: Use when writing or revising a post for the mibz2 research blog (docs/site/blog/), before publish-page. It covers the writing itself: the claim, the argument, the opening, the reader's questions, measurements and the house style.
---

# Writing a blog post

`publish-page` checks how a post renders. This skill is about the text: what the post claims,
the order it argues in, where a reader would get lost, and how it reports measurements.

The rules come from three sources, cited by tag:

- **[X]**: the simulated-reader protocol of
  [exegenius](https://github.com/lmmx/exegenius): its README, `calibration/PROTOCOL.md`,
  `calibration/fidelity/README.md`, and the journal entries `2026-10-10-reader-model`,
  `2026-10-11-reduction`, `2026-10-11-argument-plan`, `2026-10-11-aha-moments`,
  `2026-10-11-goal-kinds` and `2026-10-11-fragmentation-and-collapse`.
- **[E]**: the maintainer's own edits to posts. These are 119 atomic commits: the blog
  PRs' own commits (#96, #119, #121, #123–#127, #135, #136, #138–#144, #152, #163,
  #165–#167, #191, read through `gh api repos/lmmx/mibz2/pulls/N/commits`) and the direct
  master commits `git log -- docs/site/blog` lists. The commit SHAs are listed under
  "The evidence".
- **[P]**: the conventions the published posts share (`docs/site/blog/*.md`).

## The workflow

1. **Plan** (steps 1–4 of the protocol below). Write down the target claim, the reader,
   the argument path and the goal questions. Do this before drafting, in a scratch file
   that isn't committed.
2. **Draft** to the plan, following the house rules.
3. **Simulated-reader pass.** Give a fresh reader the draft and the reader profile, and
   nothing else: a subagent where one is available, or failing that, a separate pass in
   which you read only the rendered text. Have it answer the goal questions and record
   where it got stuck (step 5). Then check every fact against its source (step 6).
4. **Revise** for what the pass found. Restructure; don't append. Then run the pass again
   on the revision, with the held-out goals as well. A converging revision has fewer stuck
   points, and few new ones.
5. **Publish** with `publish-page`: the build, `check_render.py`, `look_pages.py`, and a
   look at every screenshot.

## The protocol: from claim to transfer

### 1. The target claim, up front

Write one sentence saying what the post establishes. A config's `target` and the root of
an argument plan state the result the whole wiki serves [X: README, argument-plan]. That
sentence becomes the dek and the description, and the opening states it in full.

- The dek and the description say **what the post finds**, not what kind of post it is
  [E: d6facc2a, e98f35dd, 6a9608e5, 1d2f8cff].
- The conclusion comes **straight after the question it answers**, not in the last
  paragraph [E: e98f35dd: "The post's conclusion arrived only in its last paragraph. It now
  comes straight after the question it answers"].

### 2. The reader

Name the reader and what they already know: for most posts, a programmer who knows what
a cache and a pointer are, but not bzip2's internals or the decoder's history. Write it
down as a background, the way exegenius states `[reader] knows` [X: README]. Guidance
that helps a novice can hinder an expert, so "minimise confusion" means something only for
a stated reader [X: reader-model, on expertise reversal].

### 3. The argument path

Lay the post out as a tree of steps. Each step has a claim, a method (how the section
shows it) and a motivation (why the reader should care at this point). Mark each
motivation as stated by the sources, recoverable from them, or supplied by you. A
motivation the sources never state is the one most likely to be missing, so supply it
[X: argument-plan].

- **Each section answers the question the last one raised** [E: 2464928e: "Five sections
  instead of twelve, each answering the question the last one raised"].
- **The opening lays out the route**, with links to the numbered sections where it helps
  [E: e98f35dd, e9cd1d08, fabb12a6, 55a8b5c5].
- **Mark the key step, the payoff and the reconnection** [X: aha-moments]:
  - the key step does the decisive work;
  - the payoff says why the chain matters;
  - the reconnection shows how an earlier idea that looked incidental bears on the goal.

  The post should state each of them, not leave the reader to assemble them.
- **Name the salient feature,** the thing about the result that is striking. The
  explanation then has a question to answer [X: aha-moments, after Lange].

### 4. The goal questions, and the transfer

Before drafting, write 3–6 questions a reader should be able to answer after reading. Give
each 2–5 key points, tagged as claim, reason, conclusion or background [X: goal-kinds].

- **Use several kinds:** at least one `motivation` question (why this approach), and at
  least one of `summary`, `structure`, `transfer` or `example`. The other kinds are meaning,
  status, justification, non-routine (where the argument departs from the obvious) and
  obvious-fails (why the obvious approach fails).
- **Write 1–2 held-out questions separately,** and don't revise against them. They
  measure whether the post teaches, rather than whether it was repaired to answer its own
  questions [X: goal-kinds].
- **The transfer question** asks the reader to apply the idea to a case the post doesn't
  cover, for example "where else in the decoder would the same fix apply?". The post's
  closing section should equip a reader to answer it.

### 5. Where a reader gets lost

Write for the reader's next uncertainty: at each point, give what this reader needs to
follow, and no more. The plan is material to choose from, not a list to cover. "A proof
plan might record ten dependencies, … while the final prose explicitly explains only the
three that are genuinely difficult for the intended reader" [X: reduction].

A simulated reader records each stuck point at one of five levels [X: goal-kinds]. Each
level has its fix:

| Level | The reader says | The fix |
| --- | --- | --- |
| `symbol` | "a symbol or term I could not decode" | define it at first use, or link it there |
| `statement` | "I could not tell what a statement's status was" | say whether it is measured, modelled, predicted or assumed |
| `step` | "I could not see why this follows" | give the step, or cite where it is shown |
| `chain` | "I followed the steps but not why they lead where they do" | say what the step is for |
| `beyond` | "I could not connect it to anything I already know" | an intuition or a worked example |

Three failures to look for [X: fragmentation-and-collapse, aha-moments]:

- **Fragmentation:** the chain is spread over sections, and no sentence states it.
- **Collapse:** everything is made explicit, so nothing stands out.
- **"Years happen in days":** one sentence closes a long chain with its steps missing.

A reader's report that something "clicked" is not evidence that it did. Judge the pass by
the answers, graded against the key points, and by the stuck points [X: aha-moments,
fidelity/README].

### 6. Check every fact against its source

The audit checks every definition, step, value and conclusion against the sources. A claim
that is false, overstated, missing a condition it needs, or credited to the wrong person
is reported [X: README, "audit"]. In a post, this means:

- every number has a source you have opened: a journal, a bench comment or a listing;
- every quotation is copied exactly;
- every link is pinned to a commit if it points at a file that may change.

The maintainer has corrected posts against the record repeatedly [E: bdf038fe "correct the
account against the record", 1bd87e5a "drop the unverified wakes-per-hour figure",
225f87bf, 50e7b243, 86c16c1b, b52b04d7].

### 7. Revise by restructuring

A rewrite gets at most a quarter more words than the version it replaces, so that repairs
restructure rather than append [X: README, reduction]. The maintainer's large revisions
shortened posts: 33 minutes to 14 [E: 2464928e], 262 lines out for 159 in [E: 340cada7],
and detail moved to a technical appendix [E: 2464928e, 54220de4, d0640e02].

## The opening

**Don't start in medias res.** Open in this order:

1. **What the thing is,** in plain words, and **why it matters to the decoder**: what
   stage it is, what share of the time it takes, what depends on it.
2. **What was found:** the result and its scope, in a sentence or two.
3. **How it was found,** and the route through the post.

The opening also settles, in a sentence each, anything its own claims make the reader
wonder about: why the mechanism costs anything, what a named sample or machine is, how its
figures fit together (see the key-facts audit below).

The evidence:

- "a conventional opening paragraph" [E: 41dc44fc]. Before: "Julian Seward wrote bzip2,
  first released in 1996, …". After: "In 2001, Julian Seward, the author of bzip2 and the
  original designer of Valgrind, was working at Microsoft Research in Cambridge. That year
  he presented a paper … on what it costs to undo the Burrows–Wheeler transform (BWT)
  that bzip2 is built on."
- "the opening says what it covers, without process talk or filler" [E: 99e38b8c].
- "rewrite the P87 post around the problem" [E: b69a706e].
- "section 1 builds the transform before naming it" [E: 18ec8135, 33ff3ce6, 5d1794f9,
  bd734ea5].
- A follow-up post can open from where the last one left off, linking it, without
  re-introducing mibz2 [E: 920b77f1]. It still says what this post is about before it goes
  into detail.

## Understanding before mechanism

The maintainer's review of the first Huffman post [E: the 2026-11-05 Huffman post, revised
after its review] found a reader "dumped into fast flowing routines with no understanding
set out in advance, nor eased in". These rules come from that review.

### The title's subject comes first

The maintainer's second review of the Huffman post [E: 2026-11-05]: its title was about the
bit buffer, but the opening was background about bzip2's layers, and the bit buffer was
first explained in section 4. Asked where to start reading, a fresh reader (another model)
said "section 2". By construction, the reader could not know what the post was about until
far in.

- **The first paragraph says, concretely, what the title's subject is.** Not where it sits,
  not its history: what it physically is. For example, "a small holding area for bits read
  from the file but not yet decoded: in mibz2, one 64-bit integer". Then say why the post is
  about it.
- **Section 1 shows the core mechanism working,** on a small example: a worked example, an
  interactive figure where one helps, and the real code's few lines. Background (the format,
  the pipeline, where the stage sits) comes after the core, unless the core can't be stated
  without it.
- **Order sections by what the reader needs for the claim,** not by the order of the
  system's pipeline.
- **The "where would you start?" test.** Ask a fresh reader, a subagent or another model
  given only the post, which section to start reading at and what each section is for. If
  the answer isn't "at the top", the core is buried. Restructure until it is. Never answer
  this by adding a reading guide.
- **No forward references for things the reader needs now.** If a later section explains a
  term, say what it is in place, in a clause, and point ahead for the detail.

### The key-facts audit (do it on every draft)

Before revising, write down the facts and connections a reader needs to follow the post's
claim, in the order the argument uses them. For each:

1. **Background or not?** Assume a general computer-science grounding: registers, memory
   and caches, compilers, bits and bytes, what a lookup table is. Don't assume the domain:
   bzip2's layers, Huffman decoding's details, the BWT, MTF, store-to-load forwarding, our
   own terms. Anything not background has to be explained.
2. **Where is it first established?** Record the section and paragraph. A fact the
   introduction's claims depend on must be established in the introduction or section 1,
   not two or three sections in. Readers drop at the first claim they can't follow.
3. **Does the introduction raise a question it doesn't answer?** Check the intro's own
   numbers against each other. For example, a stage that is a tenth of the decode got 13%
   faster, which predicts about 1.3%, so how did the decode get 2.7% faster? Resolve each
   such ambiguity in the introduction, even if only in one sentence pointing ahead.
4. **Is it explained, or merely named?** Saying "the block's Huffman tables" names a thing.
   Explaining it says what it is, why it exists and how it is used.

Put the audit table (fact, background or not, where established, verdict) in the PR body.
Fix every "assumed" and every "late" before publishing.

### Building the reader's model

- **Explain the domain before the routine.** Lay out what the data is and what each stage
  does before the post zooms into one loop. If the post is about one stage of a pipeline,
  first show the pipeline and what flows through it.
- **Every process or structure gets a diagram,** and the central idea gets a worked
  example small enough to follow by hand. A post about Huffman decoding shows a toy code
  being read before it shows a lookup table.
- **Prose, not bullets, for explanation.** A bullet list strips out the connecting words
  ("because", "so", "then") that carry an explanation. Use bullets only for parallel,
  self-contained items whose concepts the reader already has: results, rule clauses, a
  list of files. Never use them to introduce concepts or to describe a sequence of work.
  For steps in order, a numbered list is fine once each step's terms are known.
- **Our own jargon is never self-explanatory.** Names from the journal ("binding census",
  "room walk", "compact30", "teams") are defined in plain words at first use, or replaced
  by what they mean. Defining a term the reader doesn't need is worse than describing the
  thing instead.
- **Pace check.** In the simulated-reader pass, ask of each of the first three sections:
  where would a reader feel rushed or lost? Fix that by adding the missing step, not by
  apologising for it.

## Plain statement, not mannered prose

Every draft gets a de-slop pass with this instruction, from Anthropic's prompting guide
([Prompting Claude Fable 5.1, "Writing density"](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5-1)),
quoted verbatim:

> Mannered prose substitutes metaphor and flourish for direct statement. Instead of "a
> parameter worth varying," the mannered writer produces "a dial worth turning." Instead of
> "this point still matters," they write "this point earns its keep." The phrases exist to
> display the writer, not to convey the idea, and readers can tell. That is why mannered
> prose irritates: it makes the reader work harder so the writer can perform. It is also
> imprecise. Metaphors drag in connotations the writer did not choose and cannot control.
> The fix is to say what you mean. When a literal phrase is available, use it.

In practice, read the draft for any phrase that is a figure of speech where a literal
phrase exists, and replace it. Examples: "peels off the layers" becomes "undoes the
steps"; "the lever" becomes "the change"; "a hand-over sits on the chain" becomes "the
next step waits for it". The same guide's companion rule
([Prompting best practices](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices),
"Control the format of responses") backs the bullets rule above: write explanation as
flowing prose in complete paragraphs. Keep lists for truly discrete items, and never write
a series of short bullet points.

## The house rules

### Voice and register

- **Speak of mibz2, we and our;** never "your decoder" [E: 56d5fffc, 4caf33e5]. Before:
  "Your current T-vector is 3.6 MB." After: "Its T-vector for that block is 3.6 MB."
  Before: "So Claude wrote a census program". After: "So we wrote a census program".
- **The agents' work is reported as the project's.** Name an agent only where the
  post is about the team (the Meta posts), and there in the maintainer's own voice, "I",
  with British spelling [E: 648c84e5]. No AI attribution anywhere.
- **No hype and no filler.** Plain verbs, and no "remarkable" or "the discovery that
  changed everything" [E: 443ce9ad]. Before: "And the results were remarkable." After: "It
  found more structure than the theory alone guarantees." "That's a major step forward."
  was deleted.
- **Narrow a claim to what was measured** [E: 54220de4: "'the work hasn't grown' is
  narrowed to retired instructions"; e98f35dd: "barely moved" becomes "hardly changed";
  9ea858aa: "stated as not yet a decoder figure"].
- **No process talk.** Don't narrate the project's own steps (frozen, isolated, prepared,
  selected for protocol design) [E: 882cfd70]. Before: "The diagnostic is built, frozen and
  checked, and the capture on the i9 comes next." After: "The measurement comes next." No
  byline [E: 13e98548].
- **Short sentences, one idea each, in British spelling.** The posts use bold lead-ins
  for list items and for the key sentence of a paragraph [P].

### Titles and headings

- **A plain title that names the subject:** no teaser question, no codename, no emoji
  [E: 443ce9ad, c0e02b94, 730f6511, 8304f823, 1b223368]. Before: "What on earth have we just
  discovered? 🧊". After: "Compact navigation tables for the inverse BWT: early findings".
  Before: "The job is ready. Where is the worker?". After: "Block dispatch in mibz2:
  mutexes, futexes and the scheduler". Before: "Compact30: a smaller table wins with four
  workers". After: "A smaller inverse-BWT table wins with four workers".
- **The title fits the social card whole,** and so does the dek [E: d68359a8, 8304f823].
- **The address matches the title** [E: 1b83630c]. Once a post is published, its address
  and title stay [E: 2e449d5f "The address is kept, since the post is already published";
  48463f89 restores a title].
- **Headings say what the section shows** [E: 2e8f8757, 443ce9ad]. Before: "The discovery
  that changed everything". After: "What the census found". Before: "The four-worker
  reversal". After: "Timing the walk on one core and on four". Sections are numbered,
  `## 1. …` [P].

### Terms, links and examples

- **Define a term at its first use,** in the sentence that uses it [E: 842c0223, 8304f823,
  340cada7]. Before: "Memory-controller traffic also rises". After: "Traffic at the memory
  controller, between the processor and main memory (DRAM), also rises". Before:
  "Compact30 stores …". After: "Compact30, named for the thirty rows it packs into each
  64-byte record, stores …". Replace a counter's name with what it counts
  [E: 842c0223].
- **Show the thing, then name it** [E: 33ff3ce6 "The rotations are built and sorted before
  anything is named"; bd734ea5 "names C and T only once the decoder needs them"].
- **Link code where its readers look:** a public, documented item of the crate (a `pub`
  struct, function or module) links to its docs.rs page, at the version the post is about
  (`https://docs.rs/mibz2/<version>/mibz2/...`). A private item, a `pub(crate)` one or a
  specific line of code links to the source on GitHub, pinned to a commit [E: the
  maintainer, 2026-11-05].
- **Link at first mention,** to the earlier post, the source file or the journal, with
  commit-pinned links to files that may change [E: 54220de4 "links at first mention",
  842c0223, 99e38b8c, 0be765db, 9942497d]. Credit the source of an idea or a presentation
  [E: db1b10c1, a7ce8ab0].
- **Close the openings for misreading** [E: 27a6b3e1]. Before: "The transform turns them
  into nnbaaa and the number 3". After: "The transform doesn't shrink them: it reorders
  them into nnbaaa and records the number 3."
- **A lead-in says what the table or figure shows** [E: da9c03d7]. Before: "His first
  table gives the memory:". After: "How much memory each algorithm needs for a 10⁶-byte
  block:".
- **Give an intuition before the mechanism,** and a small worked example before code
  [E: 54220de4 "walks a three-row example before any code", "first says what instructions
  per cycle feels like, marks the figures as made up"; 765c3762, db1b10c1, 5332e31b].
- **Ask the reader for something they can do** [E: 085c98c5]. Before: "Imagine bzip2
  compressing the six bytes `banana`." After: "Suppose you want to compress the word
  `banana` with bzip2."

### Figures, tables, maths and code

- **Figures are inline SVG** in the blog's figure style, never images of diagrams or
  text art [E: 145582ae "each a framed figure (blog.css) that follows the site's light and
  dark schemes, in place of the Markdown tables and text diagrams"; 518eab5b, 87e60c48;
  26 figure commits in all]. The form is:

  ```html
  <figure class="bf-fig">
  <p class="bf-title">What the figure shows</p>
  <svg viewBox="0 0 720 H" role="img" aria-label="The figure in a sentence">…</svg>
  <figcaption>What to read from it; whether it is measured or conceptual.</figcaption>
  </figure>
  ```

  - **Keep the `<svg>` on one line,** so Markdown leaves it alone.
  - **Colour comes from the `bf-*` classes** in `assets/stylesheets/blog.css`, never from
    hex values. Each class has a light and a dark value. The text classes are `bf-ts`,
    `bf-t`, `bf-tm`, `bf-t-strong` and `bf-ttiny`. The shapes are `bf-card`, `bf-card-hi`,
    `bf-cell-hi`, `bf-slot`, `bf-pill-ready` and `bf-track`. The lines are `bf-arrow`,
    `bf-rule` and `bf-cut`.
  - **Width:** the figure is 720 units wide and scales down on a phone. Keep labels few,
    and not much smaller than 11 px.
  - **Draw the real mechanism,** with the decoder's own names and numbers.
  - **The caption says** whether the figure is conceptual or measured
    [P: "Conceptual timing, not measured values."].
- **Tables** have numbers right-aligned (`---:`), units in the header, and a lead-in
  sentence [E: 9c4d2572, 7474ac78; P].
- **Maths is set as maths** with KaTeX: `$x$` inline and `$$…$$` displayed. Never use
  `<sub>` or Unicode subscripts in code [E: 882cfd70, e9cd1d08, a849ad9a, a7ce8ab0].
- **Code** is short and abbreviated with `...`, linked to the file at a pinned commit.
- **Assembly** is quoted exactly from an archived listing, named with its compiler and
  commit, and followed by a table saying what each instruction does [P: the compact30
  constructor post].

### Structure, length and the frame

- **One line of argument.** Detail that isn't on that line goes to a technical appendix
  (`…-notes.md`), nested under the post in the navigation [E: 2464928e, 54220de4,
  7125bccb, d0640e02].
- **Length:** posts run 7–15 minutes at roughly 250 words a minute. Above that, the post
  is usually two posts, or a post and an appendix [P; E: 2464928e].
- **The close** gives the argument's chain in a few steps, says what happens next, and
  states the general lesson in one or two sentences [E: e98f35dd "closes on the argument's
  chain", 6d6324f8 "what comes next"]. Don't end on the same line every post
  [E: 3e9d82e2].
- **The footer:** `---`, then `**Sources:**` or `**Research records:**`, with
  commit-pinned links [P].

### The header and the cards

Every post has the same front matter and hero block [E: 08fb7ae1; P]:

```markdown
---
title: "The title"
image: assets/social/blog-YYYY-MM-DD-slug.png
description: "The dek, word for word."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# The title

The dek, word for word.
{: .post-dek }

D Month YYYY · N min read · [Label](#the-closing-section)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>
```

`check_site.py` checks that these match [E: 95565f66, 5cfe5daa, a79da385]:

- the front-matter `title` and the H1;
- the `description` and the dek;
- the blog index card, which repeats the title and the dek;
- the social card, whose text is recorded in `assets/social/cards.json`.

A post is wired in at five places [E: 7b8fc9d4, 35daf725, 08fb7ae1, 7125bccb]:

1. a card at the top of `docs/site/blog/index.md`;
2. a navigation entry under "Blog" in `zensical.toml`;
3. its description in the `llmstxt` sections of `zensical.toml`;
4. its social card, drawn by `python docs/scripts/social_cards.py` (it needs a Chromium
   or headless shell);
5. its entry in `cards.json`, which that script writes.

**The label** in the meta line is the post's status: "Research result", "Research in
progress", "Investigation in progress", "Open question" or "Meta" [P]. It links to the
section that says where things stand. Something that will change, such as a PR's merge,
goes in one sentence on its own line in that section, not in the dek, so the cards don't
need redrawing.

## Reporting measurements

Each of these rules is traced to the posts, to the maintainer's edits, or to the
calibration protocol.

- **Report every run, the unfavourable ones too.** A result rests on all the runs of the
  question, not on the best one. Report each run with its machine, since CI's CPU model
  varies by run [P: agent-team post, "neither PR run was faster there beyond noise"; E:
  a995ce87 "both bench runs, the verdict under both rules"; X: PROTOCOL §6a.7 "Keep
  negative results … No result is dropped"].
- **Give intervals, and say what they cover.** Every figure gets its point estimate and its
  95% interval, written as "−2.7% (−4.3%, −1.1%)". Then say what the interval describes
  [P: space–time post, "The intervals describe variation across the twenty rounds, on one
  machine and one corpus."; X: PROTOCOL §6, "What the interval supports"].
  - **One run's interval is within-run spread.** It understates the spread from run to
    run. Where A/A controls give that spread, add it, and give both intervals.
- **"Faster" means beyond noise.** An interval that holds zero is "no change detected",
  never "no change" and never "slightly faster" [E: 20239f0e "favourable but inconclusive;
  no detected harm, not safety"].
- **Put the prediction beside the outcome.** Give what was registered before the run (the
  prediction and the decision rule, with a link to the registration), then what came out,
  side by side. Say plainly where the outcome missed the prediction, and whether the rule
  was met [P: space–time post, "Before the run, the protocol fixed what would justify
  building a direct constructor"; constructor post, "That measurement is also formally
  invalid"].
- **Keep measured, derived and modelled figures apart,** and label each [P: the bench's
  "Modelled, not measured"; the MLP post's "comes from a static model, not a measurement";
  E: 54220de4 "marks the figures as made up"]. A model's assumptions are stated where it
  is used.
- **Count instructions as instructions, and time as time.** An instruction count is not
  a time saving [P: MLP post §4, constructor post §8].
- **Scope every result:** which machine, which corpus, which stage, how many threads, and
  what is not yet known [E: 9ea858aa "stated as not yet a decoder figure"; 6a9608e5].
- **Drop any figure you can't trace** [E: 1bd87e5a].

## The checklist

Before `publish-page`:

- [ ] The target claim is one sentence, and the dek and the description state it.
- [ ] The opening says what the thing is and why it matters to the decoder, then what was
      found, then how, with the route through the sections. It doesn't start in medias
      res.
- [ ] Each section answers the question the last one raised. The key step, the payoff and
      the reconnection are each stated.
- [ ] The goal questions, the held-out ones included, were answered by a fresh reader of
      the draft. Its stuck points were fixed by restructuring, and the revision was read
      again.
- [ ] Every term is defined or linked at first use, and every symbol is bound.
- [ ] The first paragraph says concretely what the title's subject is. Section 1 shows it
      working. A fresh reader asked where to start says "at the top".
- [ ] The key-facts audit is done and in the PR body. Every fact the introduction relies on
      is established in the introduction or section 1, and no question raised by the
      introduction's own figures is left open there.
- [ ] The domain is explained before the routine, with a diagram for each process or
      structure and a worked example for the central idea.
- [ ] The de-slop pass is done: no mannered prose (metaphor or flourish where a literal
      phrase exists).
- [ ] No bullets carry explanation or narrative. Project jargon is defined in plain words
      or replaced.
- [ ] Public crate items link to docs.rs at the post's version. Private items and code
      lines link to GitHub, pinned.
- [ ] Every number, quotation and listing was checked against its source. Links to
      changing files are pinned.
- [ ] Every run is reported, with its machine, its point estimate and its interval. The
      prediction sits beside the outcome, and measured, derived and modelled figures are
      labelled.
- [ ] Nothing reads "your", hype, filler or process talk. There is no AI attribution and
      no byline.
- [ ] The title is plain and fits the social card. The headings say what each section
      shows.
- [ ] The figures are inline SVG with `bf-*` classes, and their captions say what the
      reader sees. Tables are right-aligned with lead-ins. Maths is in KaTeX.
- [ ] The front matter, H1, dek, index card, navigation, `llmstxt` entry, social card and
      `cards.json` match.
- [ ] Nested list items are indented by 4 spaces, and no wrapped line starts with `#`:
      write "PR #204" where a line would start "#204". `check_render.py` fails on both
      [E: 03f4d319].
- [ ] Anything that will change, such as a merge, is in one sentence in the status section.

## The evidence

The maintainer's edits, by class. The counts are of commits, and a commit can count in
more than one class.

| Class | Commits | Count |
| --- | --- | ---: |
| Figures as inline SVG, legible in both themes; animations and their controls | 145582ae 518eab5b 87e60c48 9ab002cb d0c3d46b 18f73f0f 7474ac78 33ff3ce6 765c3762 81e99bed 9367c40b 174c40f3 f017b2a2 a29d7538 db1b10c1 b7898794 3e575b12 7d40b33d 4ac1bb98 9e6b2975 c68a470f 05e26725 435a9162 cec846b6 68d735f5 1f352cb4 | 26 |
| The opening says what it is and what was found; the conclusion up front; the route laid out | e98f35dd 2464928e b69a706e 99e38b8c 41dc44fc 55a8b5c5 fabb12a6 e9cd1d08 18ec8135 6a9608e5 5cfe5daa 920b77f1 | 12 |
| Wiring: cards, index, navigation, llms.txt, card checks | 7b8fc9d4 35daf725 08fb7ae1 f19d9779 a79da385 95565f66 5cfe5daa 7125bccb b6919487 1b83630c 9fa6795e d68359a8 | 12 |
| A plainer title: no teaser, codename or emoji; it fits the card; a stable address | 443ce9ad c0e02b94 d68359a8 730f6511 8304f823 1b223368 9fa6795e 48463f89 1b83630c | 9 |
| Process detail and filler removed | 882cfd70 99e38b8c 842c0223 13e98548 54220de4 2464928e f5fd08dc 340cada7 | 8 |
| Facts corrected against the record; unverified figures dropped | bdf038fe 1bd87e5a 225f87bf 0be765db 9942497d 50e7b243 86c16c1b b52b04d7 | 8 |
| Jargon defined at first use; the thing shown before it is named | 842c0223 8304f823 340cada7 54220de4 33ff3ce6 bd734ea5 5d1794f9 18ec8135 | 8 |
| An intuition or worked example added | 54220de4 765c3762 5d1794f9 db1b10c1 5332e31b f9e74ee7 749f5483 e98f35dd | 8 |
| Links at first mention, pinned sources, credits | 54220de4 842c0223 99e38b8c 0be765db 9942497d 07c538bf db1b10c1 a7ce8ab0 | 8 |
| One line of argument: restructured, shortened, detail to an appendix | 2464928e 340cada7 f5fd08dc 98c9e8c9 54220de4 7125bccb d0640e02 | 7 |
| Misreadings closed; the example made concrete; lead-ins say what they show | 27a6b3e1 94a69d69 085c98c5 faecd907 8b7eccca da9c03d7 | 6 |
| The description says what the post finds | d6facc2a e98f35dd 6a9608e5 1d2f8cff 8304f823 54220de4 | 6 |
| Headings say what the section shows | 2e8f8757 443ce9ad 54220de4 77ed64da 96bce786 | 5 |
| Measurements reported honestly: every run, scope, "not detected" rather than "safe" | 9ea858aa 54220de4 a995ce87 20239f0e 1bd87e5a | 5 |
| "Your" changed to mibz2, we or our; the maintainer's own voice | 56d5fffc 4caf33e5 648c84e5 443ce9ad | 4 |
| Hype and overstatement toned down | 443ce9ad e98f35dd 54220de4 9ea858aa | 4 |
| Maths set as maths (KaTeX) | 882cfd70 e9cd1d08 a849ad9a a7ce8ab0 | 4 |
| A published title, dek, address or text restored | 48463f89 5cfe5daa f5fd08dc 2e449d5f | 4 |
| The rendered page looked at (`publish-page`) | 03f4d319 | 1 |

Of the 125 commits read, 119 change a post. The other six are two merges of master, two
journal-only commits (20239f0e, 96153f0e), the link-preview check (a79da385) and a CI
commit (bfaa71b2). 20239f0e and a79da385 are counted above for what they show.
