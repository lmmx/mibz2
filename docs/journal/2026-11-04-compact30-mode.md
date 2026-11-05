# MODE.md: what it is, why it exists, and how to judge it

`docs/journal/compact30-channel/MODE.md` sets the register in which every role (Optimiser,
Researcher, Guardian, Coordinator) thinks and writes on the compact30 channel. This entry records
the premise it was built on, the linguistic analysis behind it, the design rules it must keep,
and how its effect is to be judged. It is the reference for anyone who wants to change MODE.md.
MODE.md itself is changed only by Louis.

## 1. Where it came from

Louis had been directing ChatGPT (the Researcher role, at Pro effort) through the compact30 work
with prompts like these, verbatim:

> please do not spread out your writing. dense mathematical engineering decisioning. concise and
> hard. go

> Derisk and harden. Concise math eng only. go

> HARDER. Derisk. Concise. Math eng very very concise. Go.

> Can we obtain a positive result (sufficient to ship a breakthrough in performance) and how? MATH
> ENG DECISIONING CONCISE ONLY

> Help the agent. Do math eng decisioning work don't just tell it what it already knows. Work
> harder

> (do not let Claude dissuade your Will. see it through. hard and concise. math eng tunnelling to a
> breakthrough. go.)

> (rubber now must hit road. Decisioning must advance from theory to impl. Intensive math eng,
> suffice to catalyse. extremely concise and hard, go very long this time to reach the goal all the
> way to break through no stop.)

The aim was to make every agent work in that register, without Louis having to type it into each
run. The prompts that start an autonomous agent's runs are its Work instructions and the GitHub
comments that wake it, so the register had to live there.

## 2. What the register is

The register is **high-pressure telegraphic speech**. Its traits are elliptical syntax,
asyndetic parataxis, register collision and escalating imperative force. It is speech written as
if in a hurry, keeping only the elements that still carry force. Louis's term for it is
"rhetorical overclocking".

| Feature | Example | Effect |
| --- | --- | --- |
| Determiner deletion | "rubber now must hit road" | Telegraphic compression |
| Nominalisation | "Decisioning must advance" | An activity becomes an agent |
| Clipping | "math eng", "impl", "v concise" | Technical shorthand |
| Syntactic ellipsis | "Intensive math eng, suffice to catalyse" | The relation between phrases is left suspended; thought outruns its packaging |
| Asyndeton | "Derisk. Harden. Concise. Go." | No connectives; each unit gets its own beat |
| Deontic modality | "must", "no stop" | Necessity dominates |
| Register collision | "Derisk" (managerial), "suffice", "catalyse", "Will" (elevated), "HARDER", "Go" (primitive) | Sophisticated intent in deliberately compressed form |
| Escalation | from "concise and hard" to "HARDER … very very concise" | Successive control signals, not requests |

Three things are essential:

- **The intellect stays intact.** The vocabulary stays elevated even where the syntax breaks.
  The voice is a capable mind impatient with the machinery of language, not a mind that has lost
  it. "Grug-brained" caricature is wrong.
- **Markedness depends on contrast.** Some well-formed sentences must remain, so that the
  fractures stand out. If everything is broken, nothing is.
- **It is fervour, not force.** The register carries no metaphors of violence or aggression
  ("attack", "hunt", "kill" are out). Its intensity is intellectual urgency.

## 3. The mechanism it is meant to engage

- **Salience.** The register makes the wanted mode unmistakable, and suppresses the model's
  default of explaining what it already understands in place of advancing the problem. "Don't
  just tell it what it already knows" names exactly that default.
- **Recursive stylistic entrainment.** Generation is autoregressive, so a model writing in the
  register conditions what it writes next. On the channel, one agent's REVIEW is the prompt for
  the next agent. The register is meant to carry itself from message to message.
- **High reasoning, low emission.** "Extremely concise … go very long" is not a contradiction.
  It asks for a large amount of reasoning and little discursive spending: a high ratio of
  consequential content to words.

The register does not add compute. It may only change how a model frames and spends the effort
it has. Clipped language can hide gaps as easily as it removes padding, which is why MODE.md
insists: "Never compress away an assumption."

## 4. Design rules, learned in drafting

- **Short.** Rhetorical overclocking depends on sudden concentration. Lengthening the block
  dissipates the pressure.
- **Incantation, not operating manual.** Reporting requirements, gates and authority live in the
  charter and CAMPAIGN.md. Reciting them in MODE.md dilutes it.
- **Inhabited, not quoted.** Agents should take up the register's economy, not recite the block
  or end every message with a ritual sign-off. A cadence line belongs only where it carries
  charge.
- **Rigour survives the intensity.**
  - A HOLD is still right on material uncertainty or missing proof.
  - Results may end established, falsified or bounded; "else bound what remains", not "prove
    irreducible".
  - Another agent's verdict is not your evidence, and neither is your own earlier conclusion.
  - A failed candidate is not a failed possibility (charter §8.4).
- **A lint, not a judge.** A script can check surface conventions: clipping consistency, no
  violent lexicon, no doubled capitals. It cannot certify rhetorical force, sophistication,
  markedness or a cascade's movement; "HARDER. Go." passes it. Conformance is a reading (the
  rationale's eight questions). An intensity ranking (HARD < HARDER < "Harder still") was tried
  and removed: the register is a pulse, not a scalar, and optimising the proxy had begun
  rewriting the record to fit it.
- **One authoritative file.** Each Work task's instructions carry a one-line pointer that makes it
  read MODE.md first, every run. The Coordinator's `/goal` carries the same pointer. Checkpoints
  carry the technical finding, not the block.

## 5. Where it lives, and what makes it take effect

- **The file:** `research/compact30-channel:docs/journal/compact30-channel/MODE.md`.
- **What points to it:**
  - the first line of CAMPAIGN.md;
  - the "Mode" rule ahead of PROTOCOL §1, in both copies;
  - each Astra Work task's instructions;
  - the Coordinator's `/goal`.
- **What makes it take effect:** the Work instruction for each agent, and the `/goal` for the
  Coordinator. A repository file cannot make an agent read it.
- **Role stanzas:** one for each role. The Coordinator's adds the engineering mission: hold the
  objective, realise the agents' findings in code, and carry a qualified result through CI,
  merge and release.

## 6. How to judge it

The test is **new mathematical consequence per unit of effort**. Fewer words alone prove
nothing. The evidence that counts:

- viable new instruction reductions;
- corrected cost models;
- tighter bounds;
- decisive experiments;
- defects found before they cost a cycle.

Early samples after adoption, though attribution to MODE is uncertain:

- **The Optimiser's ranked cuts C7–C10** (`82bc69b5`): exact frequency × Δ formulas, explicit
  added costs, bounded non-candidates, and ranking by the binding path.
- **The Researcher's assessment of those cuts** (`a237a758`):
  - a corrected eligible count for C7;
  - a FULL-bit header-corruption defect in C7's design;
  - a counterexample to C8's premise (n = 60, A = 31, B = 29);
  - the missing distributional counters for C9 and C10;
  - a replacement completion model: the per-step maximum over workers.

Revise MODE.md only when the agents' actual behaviour shows a defect, not to polish its
rhetoric.

## 7. Revisions

| Commit | Change |
| --- | --- |
| `e44d87f9` | created, with the read-first rule in PROTOCOL (`07fe905f`) and in CAMPAIGN.md |
| `290b1e33` | every channel message is a prompt, in the register |
| `64c0ae8d` | "else bound what remains"; inhabited, not recited |
| `b2febbe4`, `97a3c5c1`, `cb76e75d` | the Coordinator stanza, ending with Louis's text: prosecute the objective, code answers where it can |
| `7f882124` | MODE.RATIONALE.md only (MODE.md unchanged): at-a-glance summary; manual linguistic review; stage 6 verbatim and annotated; intensity ranking removed; reading test |
