---
name: register
description: Use on every blog post before it ships, and on any published post that reads as blunt, staccato or list-driven. It sets the blog's register - an educated, connected narrative with exact vocabulary - and gives the passes that move a draft there without tipping into mannered prose.
---

# Register: tell the reader a coherent story

Our drafts keep landing in the wrong register. Each sentence is true and short, but the prose
reads like a checklist being read aloud: a bare finding, then a bulleted glossary, then a
bulleted list of deliverables, then a roadmap. Words are chosen for being plain rather than
precise. Louis, on the "Halving the single-row steps" opening (24 November 2026): "It says things
like a yes or no rather than the word binary … it's quite embarrassing to read the opening, which
is all just quite blunt, and you feel like you're being shoved around rather than … told a nice,
coherent story."

The target is the register of a good essay in a scientific magazine or a careful lecture: an
educated reader is being led through an argument by someone who knows where it goes. It sits
between two failures:
- **Too blunt** (this skill's subject): staccato sentences, lists where an argument should be,
  homely circumlocutions for terms that exist, commands to the reader.
- **Too mannered** (write-blog-post's "Plain statement, not mannered prose"): metaphor and
  flourish standing in for the fact. That rule still holds. Elevating the register means better
  words and better joins, never decoration.

## 1. Use the exact word

Where a field has a term for the thing, use it, and gloss it on first use if the reader may not
know it. A circumlocution is not plainer; it is vaguer, and it sounds naive.

| Blunt | Exact |
|---|---|
| a yes-or-no target; a target that is met or not | a binary target; a decision problem |
| a yes (meaning a decoder that meets the target) | a decoder that meets the target |
| turned into a yes-or-no question | recast as a decision problem |
| the thing it can never remove | the invariant it cannot remove |
| goes faster on one thread, barely on ten | a 16% gain on one thread that all but vanishes at ten |
| how much it costs grows with | its cost scales with |

Check: grep the draft for `yes-or-no`, `yes or no`, `met or not`, `a yes`, `a no`, `thing`,
`stuff`, `a lot`, `big`, `basically`. Each hit either becomes the exact term or earns its place.

## 2. Join the sentences

Blunt prose states facts side by side and leaves the reader to infer how they relate. Say how
they relate: *because*, *so*, *yet*, *which is why*, *that is*, *in turn*, *by contrast*. Vary the
length: a long sentence that carries a chain of reasoning, then a short one that lands it. A run
of three sentences under twelve words is a signal to join two of them.

> Blunt: "There is no new decoder yet. What this post reports is a change in the size of step we
> attempt, and what it has already established."
>
> Joined: "This post has no new decoder to report. What it reports is a change in the size of
> step we attempt, and a boundary that change has already established."

## 3. Narrate the opening; don't list it

The opening is a story with a turn: where we were, what changed, what that made possible, what
this post shows. Definitions enter the story at the moment they are needed, in apposition or a
subordinate clause, not as a glossary the reader must hold before anything has happened. A
deliverables list becomes a paragraph that says why each piece was needed and how they fit.

- **No glossary block** at the top of a post. Weave each term into the sentence that first needs
  it ("call an *event* a step that settles one row on its own"), and link it there
  (link-and-gloss). A term that needs more than a clause gets a gloss.
- **No bulleted deliverables** in an opening. Lists are for things that really are parallel and
  will be scanned later (a checklist, a table of results), not for an argument.
- **Lead with the situation, not the negative.** "There is no new decoder yet" as a first
  sentence shoves; the same fact lands better after the reader knows what was sought.
- **Lead the reader; don't instruct them.** Prefer "we can now ask" to "ask"; prefer "it follows
  that" to a bare imperative. Imperatives are for procedures, not essays.

## 4. Keep it exact

An elevated register is no licence to blur. Every claim keeps its number, scope and source; the
bound stays "every decoder in the current family", not "any decoder". If a fancier word would be
less precise, keep the plain one. Words like *bouleversement* or *anthyphairesis* belong where they
name the thing exactly, with a gloss or a link.

## 5. Shift register in the apparatus, not the argument

The blog's most enjoyable page, the conversation with Thomas Murphy
(`docs/site/blog/conversations/2026-11-22-thomas-murphy.html`, embedded in "Decompression is the
harder problem"), gets its liveliness from register shifts that are
**licensed by the ornament around them**. The main text stays modern and exact. The old voice
lives only where the typography has already changed costume (Louis, 24 November 2026, asking that
we mine it):

- **The fleuron gloss speaks as an editor of an old book.** "Herein the post sets forth the device
  this chat keeps circling: the comment thread of a GitHub pull request, made a message board for
  the agents, wherein every missive is headed as a letter is … So the company goes about its
  labours by summons and not by vigil: no agent polls, and naught passes through Louis's hands."
  Note the pattern: an antique frame ("Herein … sets forth", "wherein", "keeps watch upon", "is
  roused only when"), a closing antithesis ("by summons and not by vigil"), then a plain
  restatement of the fact after the colon. The conceit never replaces the fact; it is paid off by
  it.
- **Blackletter falls on the one word a sentence turns on.** *naught*, *impoverished*, *theorems*:
  a single word, the hinge of its sentence, set in `il-black`. Never a phrase-long run, and never
  more than one in a paragraph.
- **A mock-heroic rubric for a skirmish.** "Against the Beclauded Subagent", in blackletter,
  heads the passage about a sub-agent that deleted work: a title in the manner of a polemical
  tract, set over an episode that is small and comic. Use it where the content is a conflict or a
  turn, never over a result.
- **An epigrammatic gloss.** "A matter of taste: a liking, a greed even, for proofs about programs
  that run, not a verdict on number theory." One sentence, with a self-correction in the middle ("a
  liking, a greed even") and a disclaimer at the end that keeps it honest.
- **The Compendium of Links** with illuminated capitals closes the page as a book's apparatus
  would.

Words that suit the gloss voice: *herein, wherein, whence, thereby, naught, as may be, sets forth,
keeps watch upon, is roused, the reader may*. Each gloss should use at most one or two of them, and
a post should give the old voice to a few glosses, not all of them. In the body, these words are
mannered prose and are cut.

## 6. Check

- [ ] The opening reads as connected paragraphs, with no glossary block and no deliverables list.
- [ ] The section 1 grep is clean, or each remaining hit is deliberate.
- [ ] No run of three short, unjoined sentences; relations between facts are stated.
- [ ] No imperative addressed to the reader outside a procedure.
- [ ] Read the opening aloud: it should sound like a lecturer leading the room, not a list being
      read out. Then read it against "Plain statement, not mannered prose": no metaphor has
      crept in where a fact should be.
- [ ] Any old-voice turns sit in glosses, rubrics or blackletter words, and each is paid off by a
      plain fact; none are in the body.
- [ ] check_prose.py, link_audit.py and publish-page's checks still pass.

## Where it sits

After write-blog-post's drafting and before link-and-gloss and publish-page. A published post
that reads bluntly is fixed as a docs commit to master (as a blog card's revise stage, `frontier/README.md`).
