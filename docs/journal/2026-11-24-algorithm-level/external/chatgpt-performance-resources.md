# ChatGPT Pro on performance-analysis resources (verbatim)

*24 November 2026, pasted by Louis in full. The inline links did not survive the paste (`⁠￼`). The exchange, in Louis's words:*

> chatgpt told me about these resources:
>
> but i rejected many of these as off topic and asked :
>
> > what is in 🗂️ Computer Systems Performance Analysis: Techniques for Experimental Design, Measurement, Simulation, and Modeling https://www.cs.wustl.edu/~jain/iucee/index.html  and what other resources from universities show the sorts of techniques we are using or would be wise to use?
>
> (paste)
>
> ---
>
> no just
>
> 🗂️ Computer Systems Performance Analysis: Techniques for Experimental Design, Measurement, Simulation, and Modeling https://www.cs.wustl.edu/~jain/iucee/index.html
> 🗂️ Analytical Performance Modeling and Design of Computer Systems https://www.cs.cmu.edu/~harchol/Perfclass/class.html
> 🗂️ Theory of Performance Modeling ("Extra theoretical") https://www.cs.cmu.edu/~harchol/Perfclass/classtheory.html
>
> tell me all about the content of these
>
> (with pro effort)
>
> it is still thinking. can you record this in the journal durably [when i say durably i mean properly, full text not glossed] and then answer the same question chatgpt is answering. read the full material don't use subagents that gloss

*Below is ChatGPT Pro's answer to the first question (the "(paste)"), verbatim. Its answer to the narrower second question was still pending when this was recorded. The Coordinator's own answer to that second question, made from the material read in full, is `../methodology/performance-courses.md`.*

---

You've found something very close to an established academic discipline for what we have been developing with mibz2. Computer systems performance analysis is much broader than profiling or benchmarking. It brings together mathematical modelling, experimental science, statistics and computer architecture to establish how a computational system behaves.

There are also several neighbouring university research disciplines that could substantially enrich your methodology, particularly system identification, operational analysis, analytical performance modelling and experimental algorithmics.

1. What's in Raj Jain's course?

The Art of Computer Systems Performance Analysis

Raj Jain · Washington University in St. Louis

720 pages, published in 1991.

Five-day course⁠￼ · Full semester course⁠￼ · Lecture slides⁠￼ · Book contents⁠￼

The page you linked is a condensed course delivered in July 2010 to university educators, based on Jain's textbook. There is also a full fourteen-week university course, which is arguably the more valuable resource for your purposes. cs.wustl.edu +2

The five-day curriculum

Day	Subject	Relevance to mibz2
1	Measurement methodology and workload characterisation	Very high — understanding the decoder's constituent work
2	Statistics, comparisons and regression	Very high — establishing reliable cost laws
3	Experimental design and fractional factorial methods	Exceptional — avoiding combinatorial experiments
4	Queueing networks and operational analysis	Exceptional — modelling the complete decoder
5	Simulation and statistical validation	Very high — predicting behaviour across configurations

The individual subjects map surprisingly closely onto what your research team has been developing.

Day 1 — How to study a system

Choosing what to measure, defining representative workloads, avoiding methodological mistakes and characterising the inputs.

This is essentially the intellectual foundation of your Wikidata block census: establishing which properties of the workload determine computational cost.

Day 2 — How to establish a quantitative relationship

Measurement distributions, confidence intervals, comparisons between implementations, regression and the correct treatment of ratios.

This is relevant to deriving cost laws from quantities such as , retained rows, runs, strips and locality.

Day 3 — How to experiment economically

Factorial experiments, replication, interaction effects and fractional designs.

This directly addresses your question about avoiding an exponential experimental grid. There is an entire mathematical discipline devoted to obtaining useful information without testing every combination.

Day 4 — How to reason about the system

Queues, networks of interacting service centres, conservation laws, utilisation, bottlenecks and mean-value analysis.

This is the closest match to the decoder decomposition exercise.

Day 5 — How to simulate the system

Building simulations, analysing their outputs and representing stochastic behaviour. This supplies the foundation for predicting mibz2's performance at held-out worker counts. cs.wustl.edu +1

2. The university courses I would prioritise

I've selected these for their relevance to your research, rather than simply assembling a list of computer science performance courses.

Carnegie Mellon — Analytical Performance Modeling and Design of Computer Systems

15-857 · Professor Mor Harchol-Balter · Graduate level

Course and syllabus⁠￼ · Textbook and contents⁠￼ · Advanced theoretical course⁠￼

This is probably the strongest mathematical continuation of Jain's course. Harchol-Balter's research involves constructing mathematical models of computer systems and proving theorems about them to inform their redesign. cs.cmu.edu +1

Particularly relevant techniques include:

* Operational laws and modification analysis.
* Networks of queues with different classes of jobs.
* Load-dependent service rates.
* Scheduling with heterogeneous workloads.
* Asymptotic performance bounds.
* The consequences of high variability in job duration.

What I would take into mibz2: modification analysis—the ability to predict the consequences of changing a component without actually implementing the change first.

There is even a special course for systems research students⁠￼ in which students present their own research problems and the class works out how to represent them mathematically. That is remarkably similar to the council exercise we just conducted.

MIT — Identification, Estimation, and Learning

2.160 · Professor Harry Asada · Graduate level

OpenCourseWare⁠￼ · All lecture notes⁠￼

This is where we move beyond traditional performance analysis into the mathematics of discovering how a system works through experimental observation.

I would particularly recommend lectures 15–21:

Lecture	Technique
15	Foundations of system identification
16	Informative datasets and consistency
17	Persistent excitation
18	Uncertainty of estimated parameters
19	Experimental input design
20	Mathematical limits on estimation accuracy
21	Information theory and model selection

MIT OpenCourseWare +1

What I would take into mibz2: the concept of persistent excitation. Instead of simply observing the system operating normally, deliberately vary its inputs and parameters to expose otherwise hidden properties.

Our stage-specific slowdowns are a rudimentary application of this idea. We could become much more sophisticated about choosing interventions specifically to distinguish competing models.

Friedrich-Alexander University Erlangen-Nürnberg — Node-Level Performance Engineering

NHR@FAU · Georg Hager, Gerhard Wellein and colleagues

Course overview⁠￼ · Recordings and teaching materials⁠￼ · Performance modelling research group⁠￼

This might be the most immediately useful resource for the i9 experiments.

It covers the relationship between software and the physical machine: SIMD, cache hierarchy, processor topology, thread placement, counter measurements, memory bottlenecks and architectural microbenchmarks. NHR@FAU +1

Especially interesting is the group's Execution-Cache-Memory model (ECM), which extends Roofline modelling to account for execution and data transfers at different levels of the memory hierarchy.

Instead of simply measuring how long a kernel takes, the approach separates the properties of the algorithm from the properties of the processor, then combines them to predict runtime and multicore scaling. FAU CRIS

That distinction could help address one of your longstanding research problems: developing a meaningful bridge between an algorithm's mathematical cost and its actual machine cost.

Penn State — Design of Experiments

STAT 503 · Department of Statistics · Graduate level

Complete online course⁠￼

This is the deeper statistical treatment of the experimental economy question.

The most relevant lessons are:

Lesson	Topic	Application
4	Randomised blocks	Controlling workstation variability
7	Confounding	Separating competing explanations
8	Fractional factorial designs	Screening multiple interacting parameters
11	Response surface methods	Modelling performance across configurations
13	Random effects	Accounting for variation across blocks and runs
14	Nested and split-plot designs	Experiments with expensive-to-change conditions

Design of Experiments

What I would take into mibz2: response surface methodology and sequential experimental design. Instead of committing to a large fixed experimental programme, you use the first results to choose the next most informative measurements.

Karlsruhe Institute of Technology — Algorithm Engineering

Professor Peter Sanders · Institute of Theoretical Informatics

Research programme⁠￼ · Algorithm Engineering — An Attempt at a Definition⁠￼

This is particularly important because it describes the broader research philosophy of mibz2, not just this latest performance exercise.

Sanders defines algorithm engineering as a scientific feedback process integrating theoretical analysis, realistic modelling, algorithm design, implementation and experimentation, driven by falsifiable hypotheses. An Attempt at a Definition +1

The algorithm engineering research cycle, adapted from Sanders' methodology.

It validates the idea that theoretical algorithm research and empirical research should not be competing approaches. Each informs the other.

Sanders also gave an invited talk in July 2026 entitled Parallel Algorithm Engineering Reconsidered, explicitly revisiting this methodology for parallel algorithms. Talk abstract⁠￼. ACM PODC 2026

3. Foundational papers worth adding to the research library

These are more focused than the courses. Each introduces a methodology that could become part of your permanent research practice.

Paper and institution	What it gives you
Rigorous Benchmarking in Reasonable Time — University of Kent, 2013	How to allocate repetitions economically while establishing statistically credible results.
Producing Wrong Data Without Doing Anything Obviously Wrong — USI and collaborators, 2009	How seemingly irrelevant experimental conditions can produce false performance conclusions.
Coz: Finding Code That Counts with Causal Profiling — UMass Amherst and Grinnell, 2015	Predicting the benefit of optimising code through controlled virtual speedups.
Roofline — UC Berkeley, 2008	Deriving performance ceilings from computational intensity and hardware capacity.
The Operational Analysis of Queueing Network Models — Denning and Buzen, 1978	The mathematical foundations of deriving performance laws from observable system quantities.

kar.kent.ac.uk +4

The first two are especially worth giving to Pathfinder. The last three belong in the mathematical modelling programme.

4. Returning to systems biology — there is another very promising connection

Beyond metabolic control analysis, I found a body of research that seems particularly relevant to your ambition to characterise complex systems without exponential experimental costs.

Cornell University — Sloppy Models and Information Geometry

Research library and short course⁠￼ · Sloppiness, Modeling, and Evolution in Biochemical Networks⁠￼

The research of James Sethna, Ryan Gutenkunst and colleagues investigates why systems with many adjustable parameters can nevertheless exhibit behaviour governed by relatively few combinations of those parameters.

They distinguish:

* Stiff directions: combinations of parameters to which system behaviour is highly sensitive.
* Sloppy directions: combinations that can vary considerably with little observable effect.

sethna.lassp.cornell.edu +1

Illustrative contours of equal model error. Parameters can change substantially along one direction, while small changes in another substantially alter the fit.

The important insight is that the natural dimensions of a system's behaviour need not correspond to its individual components.

We have decomposed mibz2 into activities and resources. But it is conceivable that its global performance is governed by a smaller set of coupled quantities, such as scanner supply, worker demand, live working set and memory contention.

This is something we could investigate mathematically rather than assume.

For example, the perturbation campaign already commissioned could produce a sensitivity matrix:

S_{ij}=\frac{\partial\log y_i}{\partial\log\theta_j},

where the are observable system properties and the are intervention parameters.

A singular value decomposition of an appropriately scaled, uncertainty-weighted matrix could reveal which combinations of interventions matter most.

If there are only a few well-resolved dominant directions, we may be able to reduce the effective dimensionality of the model. That is a hypothesis to test, not something established about mibz2 yet.

There is also an important qualification: low sensitivity is not necessarily the same as lack of parameter identifiability. A useful companion paper is On the relationship between sloppiness and identifiability⁠￼, which discusses the distinction and its consequences for experimental design.

University of California, San Francisco — Metabolic Control Analysis

Systems Biology, BMI 203 — lecture 10⁠￼

This is university teaching material covering the very methodology proposed in your decomposition brief: measuring how changes to individual components affect overall network flux and internal state.

What fascinates me about combining this with Cornell's work is that we could investigate not only where control resides, but also the dimensionality and geometry of control itself.

That would take us beyond ordinary performance engineering into a more general mathematical study of computational systems.

5. Techniques we should consider adding to mibz2's methodology

Some of these we already use informally; others would represent new capabilities.

Technique	Purpose	Priority
Operational analysis	Derive exact accounting laws and bounds	Immediate
Discrete-event simulation	Predict the complete decoder pipeline	Immediate
ECM modelling	Connect algorithmic work to hardware cost	Immediate
Parameter identifiability	Determine whether measurements can distinguish mechanisms	Immediate
Fractional factorial design	Screen interactions economically	High
Causal profiling	Establish counterfactual optimisation benefits	High
Optimal experimental design	Select measurements by expected information	High
Sensitivity matrix decomposition	Discover dominant combinations of parameters	High
Response surface methodology	Characterise nonlinear performance landscapes	High
Information geometry	Investigate the effective dimensionality of models	Exploratory

6. A permanent academic foundation for the research team

I would organise the resources by research purpose rather than university.

research-methodology/

algorithm-engineering/

KIT — scientific algorithm development, realistic models and falsifiable hypotheses.

performance-analysis/

Washington University — measurement, comparison and experimental methodology.

analytical-modelling/

Carnegie Mellon — operational laws, queues, scheduling and bounds.

machine-modelling/

FAU and Berkeley — microarchitecture, ECM and Roofline.

experimental-design/

Penn State and Kent — statistical rigour and experimental economy.

system-identification/

MIT — parameter estimation, model selection and informative interventions.

complex-systems/

Cornell and UCSF — sensitivity, distributed control and information geometry.

I would have the agents extract techniques from these sources into reusable methodological records, with the mathematical assumptions, applicability, limitations and concrete experimental recipes for each. Not simply a bibliography.

This is a curated selection based on the published syllabuses, research descriptions and paper abstracts; I haven't exhaustively reviewed every lecture or textbook chapter.

The bigger picture

What emerges from this reading is that there are three established research traditions that together describe much of what you are trying to achieve:

Discipline	Central objective
Algorithm engineering	Discover better computational methods.
Analytical performance modelling	Explain and predict their physical behaviour.
System identification	Discover the underlying mechanisms through economical experiments.

Systems biology then offers additional mathematical tools for studying the interactions between those mechanisms.

If I were choosing just three resources to begin with, I would take Jain for the experimental methodology, Harchol-Balter for mathematical reasoning, and Sanders for the overall research philosophy. Cornell's work on sloppy models would be my exploratory fourth.

The opportunity is to move from asking whether an algorithm is faster to asking what mathematical structure determines the behaviour of the entire computational system, and how can we transform that structure?

The closest established name for what we've just done is mechanistic system identification, applied to computer systems performance.

More broadly, I would call the exercise mechanistic performance characterisation. That is a descriptive name rather than a single formally standardised methodology, but it captures what distinguishes this exercise from conventional benchmarking or profiling.

We aren't merely asking how fast the decoder is. We're trying to establish a quantitative theory of its behaviour as a complete system.

The disciplines we've brought together

Discipline	What we used it for
Systems thinking	Identifying interacting components, feedback loops and emergent behaviour.
Mechanistic modelling	Building a mathematical model grounded in the actual architecture.
System identification	Determining unknown parameters through informative experiments.
Design of experiments (DoE)	Obtaining maximum information without combinatorial experimental growth.
Sensitivity analysis	Establishing which components control global behaviour under different conditions.
Operational analysis	Deriving capacity bounds and conservation laws rather than measuring everything.
Causal inference	Distinguishing mechanisms through deliberate interventions.
Model validation	Predicting behaviour under previously unseen conditions.

System identification is particularly close to your systems-biology analogy. It formally encompasses experimental planning, model structure selection, parameter estimation and validation. www2.iea.lth.se +1

The methodology we just developed

Failed predictions feed back into the decomposition. The model evolves alongside our understanding of the system.

The hyperthreading observation is a particularly good example of why this exercise is valuable. You were counting workers, whereas the machine schedules all runnable threads, including the scanner, feeder and consumer. SMT contention can consequently appear before ten workers, although the exact onset depends on scheduling and activity.

We discovered a property of the system simply by correcting our conceptual representation of it, before running another experiment.

How to invoke this exercise again

I would retain the following as a reusable research prompt in your methodology.

Mechanistic system characterisation

Conduct a mechanistic system characterisation of this system. The objective is not immediately to optimise it, but to establish a comprehensive, quantitative and predictive understanding of its behaviour.

1. Audit the existing conceptual decomposition. Identify wrongly merged components, hidden dependencies, shared resources, feedback loops and emergent behaviours.
2. Establish which properties can be derived mathematically and which require empirical measurement.
3. Identify the relevant state variables, control parameters, observables and conservation laws.
4. Design a minimally sufficient experimental programme that avoids combinatorial explosion.
5. Introduce controlled perturbations to establish causal relationships and quantify sensitivities.
6. Propose a parsimonious mechanistic model whose parameters can be fitted to experimental data.
7. Specify held-out validation, uncertainty bounds and falsification criteria.
8. Identify changes in operating regime, bottleneck migration, nonlinear interactions and latent opportunities.

Distinguish proved, measured, argued and conjectured claims. Draw liberally from systems biology, control theory, dynamical systems, operations research, statistical experimental design and other relevant disciplines.

The deliverable is not a collection of benchmarks. It is an explanatory and predictive theory of the system, together with an economical experimental programme capable of establishing it.

Further reading

The Art of Computer Systems Performance Analysis

Raj Jain, 1991.

Remarkably close to what we have just undertaken. It brings together experimental design, measurement, simulation, queueing theory and mathematical modelling within a single discipline. profiles.wustl.edu +1

Freely available accompanying course materials⁠￼

I think there is also a distinction worth retaining in mibz2's research methodology:

	Question
Benchmarking	How fast is it?
Profiling	Where is the time spent?
Optimisation	How can we make it faster?
Mechanistic characterisation	Why does it behave as it does?
Structural mathematical research	Can we change the basis upon which it operates?

The fourth is what we have just undertaken, and it provides the scientific foundation for the fifth.

Understanding becomes an independent research objective, rather than merely a by-product of optimisation. That is the essence of your systems-biology analogy, and I would make it a permanent part of how you approach complex software systems.
