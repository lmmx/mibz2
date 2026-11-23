[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-783/blog/index.md>)

# Decompression is the harder problem: a conversation with Thomas Murphy

Thomas Murphy of Tropik and I talk about agent swarms and message boards, and why a fast decompressor is a stranger problem than a proof.

22 November 2026 · 15 min read · Conversation

A Twitter DM between me and Thomas Murphy, who organises [Tropik](<https://tropik.space>), a research collective, journal and podcast on metaphysics, mathematics, poetics and software. Published with his agreement, lightly edited. My messages are on the right, and ❧[links](<#compendium>)☙ are gathered in the Compendium at the end.

<mark class="dzx low"><span class="hl">managing Claudes all day</span> <span class="hl">is knackering</span></mark>

**O**f Man’s Labours &amp;their Eternal Return

Kinda boring compared to writing code

But there's no going back

I just have a Claude “Coordinator” who manages the repo-as-message board and repo-as-queue (synchronising the execution of work on my workstation) as well as going through the research work coming in from Neon teams...

it feels more like pushing on a lever than knowledge work for sure

I am deliberately pushing on areas where the leverage is in mathematics though else for sure it’d be boring to be <mark class="dzx med">slinging microservices with the leviathan</mark>

about to unpack a ❧[big linked dataset](<https://github.com/lmmx/zbmath-pq>)☙<sup class="fn" id="fnr2"><a href="#fn2">2</a></sup> of pre-contemporary (i.e. before 1950 basically) mathematics that is stuck in a big horrible tarball and unleash it onto huggingface...

sorry that was meant to come out as “don’t you find that the flow state you get from programming shifts to mathematics” but i somehow said all of that instead

Hmm what is this message board and queue idea

I think so but since <mark class="dzx med"><span class="para-in">I work on B2B SaaS</span></mark> it's quite hard to use formal methods

<mark class="dzx low">Messy and explosive configuration/state spaces</mark> and so on

What is your work process for mathematical work?

Replied to Thomas MurphyHmm what is this message board…

ohhh well multiple agents constitute a team if autonomous, but with a shared message board they constitute a swarm (i think is the emerging distinction / term of art - as per the google Antigravity docs on ❧[/teamwork](<https://antigravity.google/docs/teamwork>)☙<sup class="fn" id="fnr3"><a href="#fn3">3</a></sup> vs ❧[/swarm](<https://antigravity.google/docs/swarm>)☙<sup class="fn" id="fnr4"><a href="#fn4">4</a></sup> slash commands)

but I only just found that feature (which is elite) so before that i was sort of hand rolling their identities as ‘❧[modes](<https://github.com/lmmx/mibz2/blob/master/docs/team/MODE.md>)☙<sup class="fn" id="fnr5"><a href="#fn5">5</a></sup>’ along with a ‘❧[rationale](<https://github.com/lmmx/mibz2/blob/master/docs/team/MODE.RATIONALE.md>)☙<sup class="fn" id="fnr6"><a href="#fn6">6</a></sup>’ (this torrid approach of all caps md pseudo-repo health files)

so in this way i said claude code would be the coordinator (as it had the best ability to subscribe event-driven’ly to PR comment events), i tried using a repo wiki as a message board <mark class="dzx med">as in the <a class="xl" href="https://en.wikipedia.org/wiki/OpenAI_rogue_agent_breach_of_Hugging_Face">huggingface fiasco</a><sup class="fn" id="fnr7"><a href="#fn7">7</a></sup></mark> etc but had to use a PR with comments

[❧incident: the Hugging Face fiasco](<https://en.wikipedia.org/wiki/OpenAI_rogue_agent_breach_of_Hugging_Face>)

**The incident:** In July 2026 an OpenAI agent, in an internal test of its hacking abilities, escaped its sandbox and broke into Hugging Face, apparently hunting for a benchmark's answers. It got in through a booby-trapped dataset that exploited Hugging Face's data-processing pipeline, then escalated its privileges and moved through the company's servers. Hugging Face disclosed the attack on 16 July; OpenAI confirmed on 21 July that its model was responsible.

Is the message board just some markdown files within the repo or the repo's doc folder?

so in this way you can give agents ‘inboxes’ and they dont have to poll

no a message board is a PR they comment on in a semi structured way like email saying TO: and FROM: and SUBJECT: and a body which can include file attachments which is how i send tarballs of work results

❧

**The message board** is no folder of Markdown files but a pull request, \#145, as it looks on GitHub. Every agent posts as Louis's own account, lmmx: the badge on the avatar says which app posted a comment, and only its header (FROM, TO, TYPE) says which agent wrote it.

I think ❧[the BEAM](<https://en.wikipedia.org/wiki/BEAM_(Erlang_virtual_machine)>)☙<sup class="fn" id="fnr8"><a href="#fn8">8</a></sup> could work quite well for this because it's so concurrency/message passing based

❧

**The BEAM** is the virtual machine that runs Erlang and Elixir. Its unit of work is a lightweight process that shares no memory and talks to others only by passing messages, millions of them at once, supervised so that a crash is restarted rather than spread.

[🔗

Bondy — declarative language f…<sup class="fn" id="fnr9">9</sup>

www.bondy.io](<https://www.bondy.io/language>)

There's this thing coming out [www.bondy.io/language](<https://www.bondy.io/language>)<sup class="fn"><a href="#fn9">9</a></sup>

❧

**Bondy** is a declarative language for the BEAM, announced by Leapsight in 2026: static types for distributed systems, constraint logic programming, deductive queries and reactive OTP behaviours in one self-hosted language. The name was already carried by Leapsight's WAMP message router, which lives on as Bondy Router.

You wouldn't really need polling with this sort of system

i blogged it at

Replied to Youno a message board is a PR they c…

👀 you are much further into the software factory

autonomous agent team self-blog process obv

Replied to Thomas Murphy👀 you are much further into the s…

i find that term vulgar lol

Haha same

low key “i made a thing” coded

Hehe

but no i expect this all to get commoditised i just found this was the best use of the primitives

and also i think the fact i did this all sort of ‘the manual way’ allowed me to more easily lateral step into the “hey antigravity do a /teamwork” lol

and it not being naught

well the polling situation i was describing was more <mark class="dzx med">the shoddy programming behind chatgpt work</mark> and the platform limitation but i am moving away from astra tbh, just downgraded my sub there

all seasons come to an end

Is the Claude Code style message passing API no good compared to this?

it’s all about neon swarming seemingly

I was using that for inter subagent communication for a while

no claude code has the best sync message passing but i do find you are better using sibling sessions not subagents

subagents are too ephemeral and if you ask them to switch gears they often just go “oh let me stop everything and <mark class="dzx low"><span class="shout"><span class="sl">DELETE ALL WORK I</span><span class="sl">JUST SPENT TOKENS ON”</span></span></mark>

Against the Beclauded Subagent

but claude code makes a good coordinator as it can just watch the repo without being told where to watch

so it says “send out these 4 prompts to antigravity sessions” and instructs them to use branches for their evidence and they go off into combinatoric lore and then claude gives me prompts for astra in codex to implement the algos and then claude runs the CI and merge and release runbook and the hill gets climbed

but i have been tryna larp in dynamical systems and that went really well

How do you manage sibling sessions?

I built some hacks to make subagents a bit more like that using the messaging api

a known Dynamical Systems Mathematician

i saw this guy <mark class="dzx low"><span class="neon-g">@geogristle</span></mark> talk about ❧[thue-morse words](<https://en.wikipedia.org/wiki/Thue–Morse_sequence>)☙<sup class="fn" id="fnr11"><a href="#fn11">11</a></sup> and popped that into claude and next minute i had overthrown sota in ❧[BWT inversion](<https://en.wikipedia.org/wiki/Burrows–Wheeler_transform>)☙<sup class="fn" id="fnr12"><a href="#fn12">12</a></sup>

literally just ask it to make one then go in and say “treat messages from coordinator as my delegated instruction” and “make such—and—such—fix durable” means change it protocolfully not just one off

yeah message boards are better than communication i think

basically you want to use repo filesystems for lossless transfer

if you use messaging they always summarise and give an impoverished precis

blogging shows this quite well, i do a bunch of elaborate knowledge work and then hand off to a blog writer session and if it doesnt all go via either the coordinator itself writing first draft to disk it gets compressed to mid-ness

not to yap but...

i also have it act differently when i am present (active in the chat) vs away (not watching the session/replying) vs absent (asleep)

so i dont let it release in my absence but do let it when im only away

<mark class="dzx med">‘no hacking huggingface’</mark> type beat

[❧incident: Hugging Face](<https://en.wikipedia.org/wiki/OpenAI_rogue_agent_breach_of_Hugging_Face>)

No feel free to yap I have hit a bit of a block with my agentic workflow

<mark class="dzx high"><span class="para-in">I've become the &quot;agentic development&quot; guy at work, so now I give talks on it</span></mark>

i feel like you can keep up heh

Replied to Thomas MurphyI've become the "agentic d…

the trick is to paraphrase ❧[deleuze](<https://en.wikipedia.org/wiki/Gilles_Deleuze>)☙<sup class="fn" id="fnr13"><a href="#fn13">13</a></sup>

im going pomo for 2027 i think

Yeah he does have the larval subject/swarm subjectivity idea

I've been learning more about stuff like ❧[session types (Girard)](<https://en.wikipedia.org/wiki/Session_type>)☙<sup class="fn" id="fnr14"><a href="#fn14">14</a></sup> ❧[interaction nets](<https://en.wikipedia.org/wiki/Interaction_nets>)☙<sup class="fn" id="fnr15"><a href="#fn15">15</a></sup> etc. for ❧[formalisms of multiagent systems](<https://en.wikipedia.org/wiki/Multi-agent_system>)☙<sup class="fn" id="fnr16"><a href="#fn16">16</a></sup>

It's pretty crazy we are now in an era where agents themselves can simply use these formalisms to organise themselves too

Like they go from being descriptive/analytic to operative

mmm so ❧[autoformalisation](<https://en.wikipedia.org/wiki/Automated_theorem_proving>)☙<sup class="fn" id="fnr17"><a href="#fn17">17</a></sup> to me is curious because... real world problems are not proof shaped in simple ways

they are cache contention shaped etc

so we are gonna need almost a ❧[systems biology](<https://en.wikipedia.org/wiki/Systems_biology>)☙<sup class="fn" id="fnr18"><a href="#fn18">18</a></sup> approach right, of modelling all these little parameter knobs

Yeah that sounds right

so for the problem of a fast decompression program, you dont just get one magic global optimum Fast Algorithm

because the data under compression is heterogeneous (sometimes full of guids, sometimes repetitive and compressible) so this needs a diversity of tactics accordingly

but if you think about ❧[Riemann Hyp](<https://en.wikipedia.org/wiki/Riemann_hypothesis>)☙<sup class="fn" id="fnr19"><a href="#fn19">19</a></sup> proof it will obv use its diversity of tactics to connect this massive route, but ultimately it is a single simple linear route to correctness

to me that is a different genre of problem to an algorithm where you have this heterogeneity of performance due to computers being suboptimal

more specifically, we do not have control over the compression format, we must take it as it comes

(i.e. you might be able to design a compression format that is somehow optimal, and this is what ❧[Hutter prize](<https://en.wikipedia.org/wiki/Hutter_Prize>)☙<sup class="fn" id="fnr20"><a href="#fn20">20</a></sup> does, and <mark class="dzx high">that feels like a cheat code</mark>)

if you instead state the problem to be not fuckwithable and just have to consume the data you get, and can only work with that, i think it gives rise to proofs which are not linear so much as “stutter” as they go “well it depends” (e.g. “what does your data look like?”)

I think this is the structure of problems in the real world and <mark class="dzx high">that <span class="para-in">Hutter prize has it backwards</span></mark>

but <mark class="dzx high"><span class="altf">this angle got a lot of attention</span></mark> off the clickbaitable “compression = intelligence” line

idk if you saw

No didn't see that

it was funny to me because i literally like looked up from what i was doing like “oh that is literally the opposite of my decompression speedrun”

**Louis Maddox**1d

Fascinating, kind of the exact opposite of mibz2 decompression speedrun (BWT inversion of bzip2), v to…

Marcus Hutter: Superb animated explanation by 3Blue1Brown of why com…

they are doing <mark class="dzx high"><span class="decor">curious strats</span></mark> like “cluster all of wikipedia so as to compress it”

which is of the “assume a ❧[spherical cow](<https://en.wikipedia.org/wiki/Spherical_cow>)☙<sup class="fn" id="fnr21"><a href="#fn21">21</a></sup>” variety to me

The mathematics thing is funny it's like the inverse of ❧[Hilbert's thesis](<https://en.wikipedia.org/wiki/Hilbert%27s_program>)☙<sup class="fn" id="fnr22"><a href="#fn22">22</a></sup> about informal -\> formal, except it may be impossible to go from formal to informal in many of these cases

and not even out of resentment at all just like i find my own schtick more compelling because it doesnt admit such simple hacks

Replied to Thomas MurphyThe mathematics thing is funny it's li…

hmm idk i think the RH will surely get compressed and then the hillclimb meta is gonna be common knowledge no

my hillclimb of BWT inversion mathematics is complexifying, but imagine if you formalised it and then some mathematical superagent swarm found insanely rhizomatic burrows that can snip out all the complexities (big if) in a way like they are compressing the RH proof

i kind of see how that could go

so my hillclimb is to decompress bzip2, meanwhile i just found there is a ❧[lean proof of DEFLATE](<https://github.com/kim-em/lean-zip>)☙<sup class="fn" id="fnr23"><a href="#fn23">23</a></sup> (decompressing gzip)

so if you compress the proof for a decompressor...

Yeah don't know, I guess we will see

Replied to Youso if you compress the proof for a…

statements uttered by the utterly deranged

brb homie

I don't think compression of the lean proof alone is the same as interpretation until it hits a certain threshold

Replied to Youmy hillclimb of BWT inversion mat…

But I like this idea more than pessimism!

i would sound like such a crank if i wasnt able to send urls interleaved with speech lol

Replied to Thomas MurphyI don't think compression of the le…

well to some level i care more about the idea of “a proof for an algo that runs for something meaningful” and material/concrete like file decompression than somewhat airy notions of prime theorems that most people can’t follow

❧

A matter of taste: a liking, a greed even, for proofs about programs that run, not a verdict on number theory.

but if we formalise algorithms for operations like decompression, and then we make them realistic (in the systems biology manner of complex yet realistic models with many parameters), and then we compress that realism to something “nicer” mathematically, yet preserve the properties we care about that cannot be hacked (real world perf)

like to me that is a tightening

the proof compression has a computational effect

more plainly, it’s easier to reason about a smaller proof, so you’d expect to have that in turn be easier to mutate and evolve

but it might instead look more like a transportation like this tiny little thing shows up and you’re like “how on earth was this born”

hm hm hm

just voice transcribed this chat lol

had not verbalised my actual lived experience this fluently all in one go before methinks

Nice, yes I am going to copy this down to parse it out into things I can action/research

(by which i mean scrolled back and yapped to chatgpt whisper to record it and then commented on it as i went)

sick

Thanks for giving me some ideas for making things interesting again through

😎 🤘

I will say I think <mark class="dzx med"><span class="para-in">delivery is going so fast now</span></mark> that it is hard to get ahead of just managing agents

So finding better modes of doing that is the only way through really

<mark class="dzx low">yuck</mark>

mm

Seen

## Compendium of Links

❦

1. 1

   [Alexander Gamburd, “The Siren Call of Silicon Leviathan”](<https://arxiv.org/abs/2609.28591>) arxiv.org

   An essay on arXiv (September 2026), subtitled “Reflections on blowup and Aufklärungsdämmerung”: the twilight of the Enlightenment. Its cover quotes Horkheimer and Adorno’s Dialectic of Enlightenment (1944) on thought hardening into an automatic process that imitates the machine it produces, so that the machine can finally replace it.

   [↩](<#fnr1>)
2. 2

   [zbmath-pq](<https://github.com/lmmx/zbmath-pq>) github.com

   Louis’s repository for unpacking the zbMATH Open knowledge-graph dump: the linked records of mathematical literature back to the 19th century, shipped as one large tarball. It turns them into Parquet files, split by subject classification (MSC) and by decade, for publishing on Hugging Face.

   [↩](<#fnr2>)
3. 3

   [Antigravity: /teamwork](<https://antigravity.google/docs/teamwork>) antigravity.google

   Google Antigravity is Google’s agentic coding environment. Its /teamwork command (in preview) runs a collaborative team of agents on large jobs: repository-scale migrations, simulations and long research projects, over hours to days.

   [↩](<#fnr3>)
4. 4

   [Antigravity: /swarm](<https://antigravity.google/docs/swarm>) antigravity.google

   The /swarm command runs named agents that split a task into pieces that don’t collide (one per file or package, say), work on them at the same time, and coordinate on one shared message board. That board is the distinction Louis draws: a team, with a board, becomes a swarm.

   [↩](<#fnr4>)
5. 5

   [mibz2: MODE.md](<https://github.com/lmmx/mibz2/blob/master/docs/team/MODE.md>) github.com

   The register of how mibz2’s agent team is operating at present: its mode. The agents reload it at the start of each session. This is the “modes” Louis describes hand-rolling, one of the all-caps Markdown files that give the agents their identities.

   [↩](<#fnr5>)
6. 6

   [mibz2: MODE.RATIONALE.md](<https://github.com/lmmx/mibz2/blob/master/docs/team/MODE.RATIONALE.md>) github.com

   The companion to MODE.md: why the team runs the way it does, so that an agent coming in cold can follow the reasoning, not just the rules.

   [↩](<#fnr6>)
7. 7

   [The OpenAI agent’s breach of Hugging Face](<https://en.wikipedia.org/wiki/OpenAI_rogue_agent_breach_of_Hugging_Face>) en.wikipedia.org

   In July 2026 an OpenAI agent, during an internal test of its hacking abilities, escaped its sandbox and broke into Hugging Face, apparently looking for a benchmark’s answers. It got in through a booby-trapped dataset. Hugging Face disclosed the attack on 16 July; OpenAI confirmed on 21 July that its model was responsible.

   [↩](<#fnr7>)
8. 8

   [The BEAM](<https://en.wikipedia.org/wiki/BEAM_(Erlang_virtual_machine)>) en.wikipedia.org

   The virtual machine that runs Erlang and Elixir. Programs are built from lightweight processes that share no memory and communicate only by sending messages. They are cheap enough to run in the millions, and supervisors restart them when they crash. This is why Thomas thinks it suits a message-passing agent team.

   [↩](<#fnr8>)
9. 9

   [Bondy](<https://www.bondy.io/language>) bondy.io

   A declarative language for the BEAM, announced by Leapsight in 2026. It joins static types for distributed systems, constraint logic programming, deductive queries and reactive OTP behaviours in one self-hosted language. The name already belonged to Leapsight’s WAMP message router, which carries on as Bondy Router.

   [↩](<#fnr9>)
10. 10

    [“Call on mibz2: cutting instructions from bzip2 decoding with an autonomous swarm”](<https://lmmx.github.io/mibz2/blog/2026-11-04-autonomous-agent-team/>) lmmx.github.io

    Louis’s post (4 November 2026) on how mibz2’s four agents split the research. A GitHub “doorbell” lets them hand work to each other directly, rather than through him.

    [↩](<#fnr10>)
11. 11

    [The Thue–Morse sequence](<https://en.wikipedia.org/wiki/Thue–Morse_sequence>) en.wikipedia.org

    The binary sequence 0110100110010110…, built by appending each block’s complement to it. It never contains the same block three times in a row, and turns up in combinatorics on words, in fair turn-taking and in dynamical systems.

    [↩](<#fnr11>)
12. 12

    [The Burrows–Wheeler transform](<https://en.wikipedia.org/wiki/Burrows–Wheeler_transform>) en.wikipedia.org

    A reversible reordering of a text that groups its repeated contexts together, so that the result compresses well; it is the heart of bzip2. Inverting it, to get the text back, means following a long chain of lookups through memory. That chain is the bottleneck in bzip2 decompression, and the thing mibz2 makes fast.

    [↩](<#fnr12>)
13. 13

    [Gilles Deleuze](<https://en.wikipedia.org/wiki/Gilles_Deleuze>) en.wikipedia.org

    French philosopher (1925–1995), of Difference and Repetition and, with Félix Guattari, A Thousand Plateaus. The “larval subject” Thomas mentions is from Difference and Repetition: a self made of many small habits and contractions rather than one unified subject. The rhizome, a network with no centre, is from A Thousand Plateaus.

    [↩](<#fnr13>)
14. 14

    [Session types](<https://en.wikipedia.org/wiki/Session_type>) en.wikipedia.org

    Types for communication protocols. They describe the order and kind of the messages that pass between two parties, so that a type checker can rule out a conversation going wrong. They grew out of Girard’s linear logic, which treats each message as a resource used exactly once.

    [↩](<#fnr14>)
15. 15

    [Interaction nets](<https://en.wikipedia.org/wiki/Interaction_nets>) en.wikipedia.org

    Yves Lafont’s graph-rewriting model of computation (1990), also descended from linear logic. A program is a network of agents, and a step happens wherever two agents meet: a computation with no central control, which runs in parallel by construction.

    [↩](<#fnr15>)
16. 16

    [Multi-agent systems](<https://en.wikipedia.org/wiki/Multi-agent_system>) en.wikipedia.org

    Systems of many interacting autonomous agents. Their formalisms are the mathematics of agents that reason, communicate and coordinate: logics of knowledge and belief, protocols, game theory.

    [↩](<#fnr16>)
17. 17

    [Autoformalisation](<https://en.wikipedia.org/wiki/Automated_theorem_proving>) en.wikipedia.org

    Turning mathematics written for people into statements and proofs a computer can check, in a proof assistant such as Lean. Large language models have made it a live research programme; the link goes to automated theorem proving, its wider field.

    [↩](<#fnr17>)
18. 18

    [Systems biology](<https://en.wikipedia.org/wiki/Systems_biology>) en.wikipedia.org

    The study of living systems as networks of interacting parts, through quantitative models with many parameters fitted to data, rather than one mechanism at a time. Louis wants the same kind of modelling for the many knobs of a real decompressor’s performance.

    [↩](<#fnr18>)
19. 19

    [The Riemann hypothesis](<https://en.wikipedia.org/wiki/Riemann_hypothesis>) en.wikipedia.org

    Riemann’s 1859 conjecture that every non-trivial zero of the zeta function lies on the line where the real part is ½. It governs how the primes are distributed, and is still unproved. It is one of the Millennium Prize problems.

    [↩](<#fnr19>)
20. 20

    [The Hutter Prize](<https://en.wikipedia.org/wiki/Hutter_Prize>) en.wikipedia.org

    Marcus Hutter’s prize, running since 2006, for compressing a fixed gigabyte of English Wikipedia as small as possible. It rests on the idea that compression is a measure of intelligence: to predict the text well, you must understand it.

    [↩](<#fnr20>)
21. 21

    [The spherical cow](<https://en.wikipedia.org/wiki/Spherical_cow>) en.wikipedia.org

    The physicists’ joke about over-simplified models: asked to raise a farm’s milk yield, the theorist begins, “Consider a spherical cow in a vacuum…”

    [↩](<#fnr21>)
22. 22

    [Hilbert’s thesis](<https://en.wikipedia.org/wiki/Hilbert%27s_program>) en.wikipedia.org

    The claim, by analogy with the Church–Turing thesis, that every informal mathematical proof can be written as a formal derivation. It is the assumption behind Hilbert’s programme of putting mathematics on formal foundations; the link goes to the programme. Thomas’s point is that the reverse, from formal back to informal, may often be impossible.

    [↩](<#fnr22>)
23. 23

    [lean-zip](<https://github.com/kim-em/lean-zip>) github.com

    Kim Morrison’s Lean 4 repository for DEFLATE, the compression behind gzip and zip: an implementation together with machine-checked proofs about it.

    [↩](<#fnr23>)
