# ChatGPT's first reading of *Hybrid Dynamical Systems* (Goebel, Sanfelice and Teel), verbatim

Relayed by Louis on 25 November 2026 and pasted into the Coordinator's session, with the split chapters of his private copy of the book. Those chapters are **not** in this repository and must not be added; the reading notes made from them are in `../reading/hybrid-dynamical-systems/`. Recorded as pasted. The paste lost hyperlinks, figures and inline symbols that rendered as images, and the gaps are kept as they came. Louis's two messages to ChatGPT come first.

## Louis's messages to ChatGPT

> i have one of the books it recommends, but it's a bit big: can you split it into chapters using the command line (rather than trying to open it) the table of contents is on page 8 and 9. do not attach the pdf to the repo, but you can make indexes and precis of it (this is my private copy do not publish it to the repo). open the table of contents and tell me what you think will be useful in it

> please can you commit your precis of the chapters as well as (separate to these reading notes) your interpretation of their relevance to our work? you have github access to lmmx/mibz2 repo and can do this on a branch. please go through all the chapters and help me understand how i can apply its insights and dynamical systems theory, my goal is not to be stuck in theory but to find the levers that allow me to reorganise, and i think rephrasing the goal and individual work in this way will be a great help. organise your work nicely and do not cram it all into inappropriate file formats (e.g. one big REPORT.md when normal linux-like file names and formats suit)

(Louis to the Coordinator: "it is working now on pro effort...". The second message went to ChatGPT; the Coordinator has done the same work here, in `../reading/hybrid-dynamical-systems/`.)

## ChatGPT's answer to the first message

I’ve split the book into its nine chapters, with the front matter, symbols appendix, bibliography and index as separate PDFs.

All 227 original pages are accounted for. I’ve also verified that their rendered appearance and extracted text match the original, and retained the original page numbering and chapter-local bookmarks.

Nothing has been uploaded or committed to mibz2. Your private copy and its derivatives have been processed locally in this conversation.

Download the complete chapter collection

13 PDFs and a section-level index · 3.3 MB

Chapter files

Chapter	Printed pages	PDF
1. Introduction	1–24	Open
2. The solution concept	25–42	Open
3. Uniform asymptotic stability	43–72	Open
4. Perturbations and generalized solutions	73–96	Open
5. Preliminaries from set-valued analysis	97–116	Open
6. Well-posed hybrid systems	117–138	Open
7. Asymptotic stability, in-depth treatment	139–168	Open
8. Invariance principles	169–184	Open
9. Conical approximation and stability	185–198	Open

Additional files: Front matter · List of symbols · Bibliography · Index.

I’ve also extracted a machine-readable section index (TSV), giving the printed page, original PDF page and page within the corresponding chapter file. That should be useful for directing agents to specific sections without having to ingest the entire book.

Cross-chapter hyperlinks may no longer resolve in the separated PDFs; the original remains unchanged.

My reading of the book

Having examined the contents, preface and selected mathematical sections, I think this is a very good choice, although its contribution is slightly different from what I initially anticipated.

It is fundamentally a book about establishing the mathematical properties of systems in which continuous evolution and discrete decisions coexist. Its emphasis is on solution theory, stability, robustness and the geometry of the state space, rather than experimental system identification.

The authors themselves distinguish two reading routes: one through stability and feedback control, the other through mathematical analysis and the nature of hybrid solutions.

For our purposes, I would combine the two.

2

The solution concept

How to represent an execution as a mathematical trajectory.

Essential

1

Introduction

The modelling framework, switching and hybrid automata.

Essential

4

Perturbations and generalized solutions

How interventions and measurement noise affect behaviour.

Essential

6

Well-posed hybrid systems

Whether the mathematical model has sensible solution properties.

High

8

Invariance principles

Which behaviours are structurally preserved or approached.

High

3

Uniform asymptotic stability

Lyapunov functions and mathematical certificates of stability.

High

5

Set-valued analysis

The mathematical machinery underlying uncertainty and constrained dynamics.

Supporting

7

Asymptotic stability in depth

Robustness guarantees for proposed adaptive controllers.

Conditional

9

Conical approximation

Local approximation and scaling properties near operating states.

Exploratory

These priorities are my assessment for our particular research programme.

Five things I would extract from it

1. Hybrid time — Chapter 2, §2.2

This is immediately relevant to our experimental design.

The authors represent the state of a hybrid system using two independent coordinates:

x(t,j)

where represents elapsed physical time and represents the number of discrete transitions.

Illustration of a hybrid time domain, following Figure 2.1 on printed page 27.

Remember our difficulty with the ordering of Ready, Taken, Enqueued and JobGot events? Physical timestamps alone were insufficient to reconstruct the causal history.

The book gives us a mathematical language for distinguishing the passage of time from discrete state changes. We would still need the sequence numbers and causal ordering proposed in the design review, but now we have a more rigorous way of representing the resulting trajectories.

Read Definitions 2.2–2.7, pages 25–30.

2. Flow and jump geometry — Chapter 1

The book’s fundamental object is remarkably compact:

\mathcal H=(C,F,D,G)

Object	Meaning	Potential decoder application
	Flow set	Conditions permitting ongoing computation
	Flow map	Rates of productive work and physical resource evolution
	Jump set	Conditions enabling discrete events
	Jump map	State changes caused by those events

Source: §1.1, printed pages 1–2.

This might provide the formal specification underneath our discrete-event simulator.

The particularly interesting possibility is that different decoder designs can be expressed by changing different elements of this quadruple.

Changing admission thresholds changes the switching conditions. Changing the organisation of work changes the possible transitions. Changing the mathematics of an activity can change its service demands or introduce entirely different states.

That gives us a precise language for describing structural alternatives rather than merely changes to implementation parameters.

3. Perturbations and generalized solutions — Chapter 4

This chapter may be the most valuable bridge to Jain’s experimental design.

The authors show that arbitrarily small perturbations can generate qualitatively different trajectories when a system operates near discontinuous switching boundaries.

This raises several interesting questions for us. Does a slight delay cause a block to miss an admission opportunity? Does changing the order of completion alter subsequent worker assignments? Can minor timing differences propagate into substantially different resource contention?

The book contains two especially relevant examples:

* Example 4.18, pp. 83–84: robust zero-crossing detection.
* Example 4.19, pp. 84–87: using hysteresis to achieve robustness against measurement noise.

The second would become useful if we investigate adaptive worker allocation. But the first has immediate relevance to reconstructing and comparing our measured executions.

4. Lyapunov functions — Chapters 3 and 8

A Lyapunov function is a mathematical quantity whose evolution can establish properties of an entire system without explicitly solving every trajectory.

For a hybrid system, one examines its change both during flows,

\langle\nabla V(x),F(x)\rangle,

and across jumps,

V(G(x))-V(x).

The book develops sufficient conditions for stability and then relaxes them to accommodate situations where the function sometimes increases but overall progress is preserved.

I would be interested in finding analogous progress measures for our computation.

For instance, suppose we define a potential function representing the total remaining service work in a simplified decoder. Transfers between stages preserve that quantity; productive computation reduces it. Under appropriate assumptions on service rates, its decrease could establish completion-time bounds.

The deeper research question is whether we can find more informative functions that incorporate remaining dependencies, available parallelism and resource constraints.

That might provide a mathematical way of evaluating entire classes of computational organisations without enumerating every possible execution.

One important caveat: the decoder is a terminating computation, so finite-time completion matters more than asymptotic stability. We should not force stability theory onto the wrong objective.

5. Conical approximation — Chapter 9

This is the most exploratory chapter for us, but mathematically rather beautiful.

Ordinary linearisation approximates a nonlinear vector field near an operating point. A hybrid system has something else that needs approximating: the geometry of the regions within which its different dynamics are allowed.

Chapter 9 considers approximating those regions by tangent cones, alongside approximating the flow and jump maps.

Illustrative geometric approximation of a constrained state space.

I would defer this until we have established meaningful state variables and operating regimes. Nevertheless, it suggests that we might ultimately reason about the geometry of our switching boundaries, rather than simply fitting a piecewise performance curve.

What I think this book adds to our programme

There is now a potentially coherent division of mathematical labour:

Source	Contribution
Jain	Experimental design and reliable measurements
Harchol-Balter	Operational laws, resource constraints and queueing models
Goebel, Sanfelice and Teel	Formal dynamical structure, admissible trajectories and robustness
Ma and Tang	Systematic exploration of alternative structures
Khammash, Araujo and Liotta	Mathematical characterisation of the structures capable of satisfying a functional requirement

What I particularly like about the Goebel book is that it could supply a mathematical language in which the alternative designs themselves are expressed.

That is a prerequisite for doing something comparable to Ma et al.’s enumeration. We first need to know what our mathematical objects are, which transformations are admissible, what properties must be preserved, and what constitutes a genuinely different design.

The book doesn’t itself provide an algorithmic design-space enumeration method. But it may give us some of the formal infrastructure needed to construct one.

Durable reading notes

I’ve prepared a separate Markdown précis, including the priority sections, relevant definitions and examples, possible applications to mibz2, and proposed additions to the experimental design.

Hybrid dynamical systems — project reading guide

Original commentary and section references only; no reproduced book pages.

This note is suitable for adding to the repository independently of the private PDFs. I have not committed it.

My recommendation is to start with Chapters 1, 2 and 4, then approach 6 and 8. That would establish the nature of our mathematical object, how its trajectories are represented, and how perturbations affect it — precisely the questions that become relevant as our characterisation experiments begin producing data. I haven’t yet reviewed every proof in the book; this is a targeted assessment rather than a complete mathematical review.
