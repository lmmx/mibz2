# ChatGPT Pro on the content of the three performance courses (verbatim)

*24 November 2026, pasted by Louis. It is ChatGPT Pro's answer to Louis's second question: "tell me all about the content of these", about Jain's IUCEE course, CMU 15-857 and CMU 15-849b (see `chatgpt-performance-resources.md` for the full exchange). The inline links did not survive the paste (`⁠￼`). The Coordinator's own answer to the same question, made from the material read in full, is `../methodology/performance-courses.md`; the comparison is in journal §54.*

---

These three courses cover overlapping material, but their centres of gravity are different:

Course	Main emphasis
Jain: Computer Systems Performance Analysis	Conducting a complete performance study: selecting workloads, measuring, analysing data, designing experiments, modelling and simulating.
CMU: Analytical Performance Modeling and Design of Computer Systems	Building mathematical models that explain performance and help choose between system designs.
CMU: Theory of Performance Modeling	Developing and proving the stochastic mathematics behind those models, including techniques for more difficult systems.

There is an important historical distinction: the Jain page is a five-day course from 2010; the main CMU page currently describes Fall 2025; the "extra-theoretical" CMU page describes Fall 2003. The two CMU links are overlapping historical offerings, rather than a present-day introductory course followed by an advanced sequel. The 2025 course is itself mathematically substantial. 

1. Computer Systems Performance Analysis

Raj Jain — experimental design, measurement, simulation and modelling

Course page⁠￼

This is the broadest of the three. It teaches the whole process of producing a defensible performance analysis.

The site contains 30 lecture units across five days, drawing on Jain's textbook. Its stated level is senior undergraduate or beginning postgraduate. The lecture pages provide topic outlines and PDF slides; many also link recorded presentations. The five-day syllabus is a selection from the textbook, rather than the entire book. 

Day 1: Define the system, the question and the workload

The first day covers the course introduction, common methodological mistakes, choosing techniques and metrics, types of workloads, workload selection and workload characterisation.

Framing a performance study

The starting point is deciding exactly what is being evaluated:

* What is the system boundary?
* What services does it provide?
* What constitutes successful service, failure or delay?
* Which metrics represent the objective?
* Which system parameters and workload properties might affect those metrics?
* Which parameters will be varied?
* Is measurement, simulation or analytical modelling the appropriate technique?

The "common mistakes" lecture treats failures such as starting with a preferred conclusion, selecting an unrepresentative workload, overlooking important parameters, using inappropriate detail, ignoring variability, mishandling outliers and omitting assumptions.

Its purpose is to establish that a technically competent measurement can still answer the wrong question. A worked comparison of remote pipes and RPC considers several costs, including processor usage and network traffic. 

Workload selection and characterisation

This part develops a vocabulary for representing the work presented to a system. It considers benchmarks and workload models at different levels of detail, then goes well beyond simply reporting an average request size.

The characterisation techniques include:

Technique	What it captures
Averages and dispersion	Typical demand and how variable it is
Single-variable histograms	The distribution of one workload property
Joint histograms	Relationships between workload properties
Principal component analysis	A smaller set of directions summarising correlated features
Markov models	Ordering, transitions and dependence between successive requests
Clustering	Distinct workload classes and representative examples

The clustering material includes scaling, transformations, outliers, distance metrics, minimum-spanning-tree methods, centroid methods and interpreting the resulting groups.

The underlying issue is representativeness: two workloads with the same average size can have different distributions, correlations and ordering, and consequently different performance. 

Day 2: Interpret measurements and fit relationships

The six units are data presentation, misleading uses of ratios, summarising measurements, comparing systems from samples, simple regression and more general regression.

Presenting and summarising data

This covers choosing useful displays and choosing summary statistics appropriate to the quantity being measured.

The statistical material includes distributions, moments, quantiles, correlation, dispersion and Q–Q plots. It distinguishes arithmetic, geometric, harmonic and weighted means. Those distinctions matter when aggregating elapsed times, rates or relative performance figures. 

"Ratio games"

This is a particularly distinctive part of Jain's course.

It examines how comparative conclusions can change when you:

* Change the reference machine.
* Average individual ratios instead of taking a ratio of totals.
* Give benchmark components different implicit weights.
* Compare percentage improvements measured against different baselines.
* Combine percentages from groups of different sizes.

The worked processor comparisons show that different normalisations of the same observations can reverse the apparent winner.

The lesson is to specify the aggregate quantity you mean. "Average speedup" is incomplete until you define the averaging operation and the workload weighting. 

Comparing systems with uncertainty

The course develops:

* Confidence intervals for means.
* Small-sample treatment.
* Paired and unpaired comparisons.
* One-sided intervals.
* Tests of differences.
* Confidence intervals for proportions.
* Sample-size calculations.

Pairing receives explicit attention: comparing two implementations under matched conditions is statistically different from comparing unrelated sets of observations. The course also distinguishes the information supplied by an interval from the binary outcome of a hypothesis test. 

Regression: from measurements to a cost model

The simple-regression unit teaches least squares, residuals, explained variation, coefficient uncertainty, prediction uncertainty and checks of model assumptions.

A representative interpretation is:

\text{time}=\text{setup cost}+\text{cost per byte}\times\text{bytes}.

The intercept and slope can then acquire operational meanings. But the fitted relationship must still survive residual checks, and uncertainty in an average prediction differs from uncertainty in a future individual observation. 

The following unit adds multiple predictors, categorical variables, analysis of variance, F-tests, multicollinearity, curved relationships and transformations, including Box–Cox transformations.

Multicollinearity is especially important: a model may predict well while its individual coefficients are unstable because the predictors usually change together. The lecture also considers outliers and mistakes in interpreting regression results. 

Day 3: Design experiments that reveal effects and interactions

This is the most concentrated treatment of experimental design among the three resources.

The progression is:

1. Experimental-design terminology.
2. Two-level full factorial designs.
3. Replicated factorial designs.
4. Fractional factorial designs.
5. One-factor experiments.
6. Two-factor experiments without replication.
7. Two-factor experiments with replication.
8. General factorial designs. 

Main effects and interactions

Suppose an experiment varies two factors, A and B. A basic model is

y=\beta_0+\beta_A A+\beta_B B+\beta_{AB}AB+\varepsilon.

The main effects describe what changes when a factor changes. The interaction describes how the effect of one factor depends on the setting of the other.

The course teaches how factorial designs estimate these quantities through contrasts and sign tables, and how response variation can be allocated among effects. 

Replication then allows experimental noise to be estimated separately from systematic effects. The material includes confidence intervals for effects, contrasts and predicted responses, checks of assumptions, and additive versus multiplicative models. 

Fractional factorial designs

With k binary factors, a full factorial design has 2^k configurations. A fractional design uses a selected subset.

Jain explains the information tradeoff explicitly:

* Confounding or aliasing: multiple effects become indistinguishable within the chosen design.
* Alias algebra: determining which effects are mixed together.
* Design resolution: describing the kinds of effects that are confounded.
* Interpretation: deciding which conclusions are justified under assumptions about negligible interactions.

The examples include seven factors studied in eight configurations, plus scheduler and document-processing case studies.

The economical design comes from accepting a controlled loss of information. It does not recover every possible interaction from a handful of runs. 

Day 4: Queues, networks and operational laws

The day moves from queueing terminology and individual queues to networks, operational relationships and mean-value analysis.

Representing a computer system as queues

A resource becomes a service centre; jobs arrive, wait, receive service and move onward. The model specifies arrival behaviour, service requirements, scheduling, queue capacity and the population of jobs.

The network material includes open, closed and mixed systems, central-server and machine-repairman models, and product-form results including BCMP networks. These are families with specific mathematical conditions under which their behaviour can be calculated efficiently. 

Operational laws

This section starts with directly observed quantities such as completions and busy time, then connects them.

For example,

U_i=X_iS_i,\qquad X_i=XV_i,

where U_i is device utilisation, X_i its throughput, S_i average service per visit, X system throughput, and V_i visits to that device per completed system job.

It also develops Little's law, response-time relationships and bottleneck bounds.

These relationships avoid requiring a particular service-time or interarrival-time distribution. They still require appropriate accounting and averaging conditions, such as flow balance. 

Mean-value analysis

Mean-value analysis calculates the average behaviour of suitable closed queueing networks as the circulating population increases.

A characteristic recurrence is

R_i(N)=S_i\bigl[1+Q_i(N-1)\bigr],

for an appropriate service centre and network model. It connects the response time with N jobs to the queue occupancy with one fewer job.

The course develops this into an iterative calculation of throughput, queue lengths and response times, then introduces approximate methods and bounds.

Examples consider additional clients, caching that reduces disk work while adding processor work, and changes to disk organisation. The emphasis is on predicting the consequences of a modification while respecting the model's assumptions. 

Day 5: Build and analyse simulations

The final day covers simulation architecture, output analysis, random-number generation, generator testing and random-variate generation.

Simulation models

The introductory unit distinguishes model types and explains:

* Monte Carlo simulation: repeated sampling to estimate quantities.
* Trace-driven simulation: replaying an observed workload.
* Discrete-event simulation: advancing the simulated clock between events such as arrivals and completions.

These descriptions can combine: a discrete-event simulator can be driven by a real trace.

The implementation topics include state variables, the simulation clock, event sets and event-scheduling algorithms. 

Verification, validation and statistical treatment

The course distinguishes:

* Verification: whether the simulation implements its intended model correctly.
* Validation: whether that model represents the real system adequately for the intended purpose.

It covers comparison with measurements and analytical results, startup-transient removal, terminating versus steady-state simulations, unfinished jobs at the endpoint, independent replications, batch means, regeneration and variance reduction.

A central complication is correlation: successive observations from a simulation are often dependent. Recording a million job delays does not automatically produce a million independent samples. 

Random inputs

The remaining units explain pseudo-random generators, seeds, periods and statistical tests, including distributional and dependence tests.

They then teach how to generate the required workload distributions from uniform random values using inverse transforms, rejection, mixtures and sums. This separates two questions: whether the random source behaves acceptably, and whether it has been transformed into the right input distribution. 

What Jain's course equips you to do: formulate an evaluation, represent its workload, choose and analyse experiments, fit relationships, construct a simulator and explain what the resulting evidence supports.

2. Analytical Performance Modeling and Design of Computer Systems

Mor Harchol-Balter — CMU 15-857

Course page⁠￼ · 2025 schedule and handouts⁠￼

This course concentrates on using stochastic models to make design decisions: how many resources to provide, how to route work, how to schedule it, and how variability changes the answers.

It assumes undergraduate probability—distributions, conditioning, conditional expectation and moments—plus calculus. The 2025 offering satisfies CMU's CS PhD theory-core requirement. 

A. Turn a system into a precise model

The opening material distinguishes:

* Arrival rate from service rate.
* Throughput from capacity.
* Service time from total response time.
* Queue length from total population.
* Utilisation from delay.
* Open systems from closed systems.

In an open system, arrivals are externally generated. In a closed system, a fixed population circulates, so new work can depend on previous completions.

That distinction changes the answer to seemingly simple questions. Speeding a stable open server may reduce response time while leaving completion throughput equal to the unchanged arrival rate. In a closed system, faster completion can itself produce more arrivals.

This is an early lesson in selecting the right model boundary and workload mechanism. 

B. Operational laws and modification analysis

The course proves and applies Little's law, utilisation and forced-flow relationships, device demands and bottleneck laws. It asks how much can be deduced without specifying the complete arrival and service distributions. 

For a suitable closed interactive system, it derives a throughput bound of the form

X\leq
\min\left(
\frac{N}{D+Z},
\frac{1}{D_{\max}}
\right),

where:

* N is the circulating population.
* Z is mean think time outside the central subsystem.
* D is total mean service demand per job.
* D_{\max} is the largest demand at an individual device.

The first term describes a population-limited regime; the second describes a bottleneck-limited regime.

Modification analysis then asks what happens if you speed up one device, redistribute work, add a resource or change the population. It explains why improving a component may have little effect until the controlling bottleneck changes. 

An actual 2025 recitation studies whether increasing an LRU cache's hit ratio can hurt throughput. It compares implementation measurements, a queueing simulation and an analytical bound. In that example, improving the hit ratio can expose and intensify contention in the hit-path list maintenance, producing a rise, plateau and eventual decline in throughput. This illustrates the course's emphasis on the complete system. 

C. Probability, Markov chains and the meaning of long-run averages

The mathematical foundation includes:

* Generating random variables for simulation.
* Convergence in probability and almost-sure convergence.
* Laws of large numbers.
* Time averages and ensemble averages.
* Discrete-time Markov chains.
* Stationary and limiting distributions.
* Finite and infinite state spaces.
* Recurrence and ergodicity.

A time average follows one execution for a long time. An ensemble average averages over possible executions. The course examines when they agree.

It also distinguishes finding a stationary distribution from proving that a process converges to it. Those conditions matter whenever a simulation or mathematical result is presented as "steady-state performance." 

The sequence then introduces exponential distributions, memorylessness, Poisson processes, splitting and merging, continuous-time Markov chains and balance equations. These build the machinery used to derive queueing results. 

D. Individual queues, server pools and capacity provisioning

The notation becomes easier once the components are separated. In these model names, M represents memoryless behaviour, G a general distribution, and the number identifies the server count.

Model	System represented
M/M/1	Poisson arrivals, exponential service, one server
M/M/k	A waiting queue feeding k servers
M/M/k/k	k servers and no waiting room; an arrival finding all busy is rejected
M/G/1	Poisson arrivals, general service times, one server

The server-pool material calculates delay, the probability of waiting and blocking probability, and compares different resource organisations. 

Capacity provisioning asks how much spare capacity is needed for a response-time objective. It introduces scaling regimes and square-root staffing: in a particular many-server asymptotic regime, the safety margin above offered load grows on a square-root scale.

The conceptual point is that utilisation alone does not determine delay. System size and the organisation of the available capacity matter. 

E. Networks of interacting queues

The course develops time reversibility and Burke's theorem, then moves to Jackson networks and networks containing multiple job classes.

The techniques include:

* Solving traffic equations to determine flow through each resource.
* Describing probabilistic routing and feedback.
* Using local balance.
* Obtaining product-form stationary distributions.
* Representing different classes with different routes or demands.

Product form is valuable because it can turn an enormous joint-state calculation into a structured combination of local terms. Its validity depends on the network assumptions; it is not a general independence claim about everything happening inside the system. 

F. Real workloads, heavy tails and service-time variability

The workload section examines job size, age, remaining work, hazard rates, Pareto and bounded-Pareto distributions, and the existence of moments.

One question is how knowing that a job has already run for a long time changes the expected amount of work remaining. For a memoryless distribution it does not; for other distributions it can change substantially.

The course uses these distinctions to reason about migration and resource allocation. 

Phase-type models

A further technique represents complex distributions through networks of exponential phases:

* Erlang constructions can represent low variability.
* Hyperexponential mixtures can represent high variability.
* Coxian and general phase-type models provide more flexible representations.

This allows richer workloads to be analysed using Markov-chain machinery. The 2025 handout also introduces matrix-analytic methods for infinite chains with repeating structure, at a high level. 

Why variability changes waiting time

The M/G/1 material develops renewal-reward theory, the inspection paradox and tagged-job analysis.

For a stable first-come-first-served M/G/1 queue,

\mathbb E[W_q]
=
\frac{\lambda\mathbb E[S^2]}{2(1-\rho)},
\qquad
\rho=\lambda\mathbb E[S].

The second moment \mathbb E[S^2] appears explicitly: average service time alone is insufficient to predict average waiting time. The inspection paradox explains part of the mechanism—an observer arriving at an arbitrary time disproportionately encounters long service intervals. 

For illustration, take a mean service time of 10 ms and utilisation of 80%. The formula gives:

Squared coefficient of variation C_S^2	Mean queueing delay
0: constant service duration	20 ms
1: exponential service duration	40 ms
9: substantially higher variability	200 ms

These are illustrative calculations under that queue's assumptions. They show why equal average work and equal utilisation can coexist with very different delays.

G. Decide which server should receive each job

The task-assignment material compares policies such as:

* Random assignment.
* Round-robin assignment.
* Joining the shortest queue.
* Choosing the server with least unfinished work.
* Using one central queue.
* Routing jobs according to size ranges.

This makes several distinctions concrete. The shortest queue by job count may not have the least work. Separating small and large jobs can change waiting behaviour. A policy that works well for one service-time distribution can perform poorly for another.

The lecture also explains why general multi-server queues are substantially harder than the single-server case: their performance is not determined by just the first two service-time moments. 

H. Scheduling: service order, preemption and fairness

Scheduling determines which waiting or active job receives service.

The course organises policies along two important dimensions:

1. Whether running work can be interrupted.
2. Whether the policy uses information about job size or class.

It studies first-come-first-served, last-come-first-served, random order, priorities, processor sharing and preemptive policies. It compares mean response time, response time conditional on job size, variability and slowdown. Policies can have identical utilisation while giving jobs different experiences. 

The size-based policies include:

Policy	Priority rule
Shortest job first	Run the smallest waiting job next
Preemptive shortest job first	Prioritise by original job size, allowing interruption
Shortest remaining processing time	Continuously prioritise the job with least work remaining

The final handouts derive results using busy periods and transforms. For example, the delay of a selected job can be decomposed into waiting before its first service and subsequent time until completion. 

I. Transforms and busy periods

Laplace transforms and discrete generating functions provide another way to analyse distributions.

They help turn sums and recursive relationships into algebraic equations, from which moments or distributional properties can be recovered.

A busy period is an interval during which a server remains continuously occupied. Its duration includes the initial work and any further work arriving before the server empties. That recursive structure makes busy periods useful in scheduling and power-management analysis. 

What is actually included in the 2025 materials?

There are three different scopes on the website: the general course description, the textbook, and the dated teaching schedule.

The description names further research topics including SOAP, Gittins and Whittle indices, multidimensional resource allocation, c\mu priority and parallel jobs with different speedup functions. The dated schedule documents the core progression above and links SOAP as optional closing reading. 

The actual handouts explicitly say that textbook Chapters 19 and 22 were skipped in 2025: the fuller closed-network/product-form/MVA treatment and BCMP/time-sharing networks. Closed-system operational analysis was still taught earlier. This matters if you are using the textbook contents as a proxy for the course. 

The public material includes lecture handouts, problem sets and optional papers. Many handouts are worksheets with questions and unfinished derivations; the textbook provides the fuller exposition. Studying the course means doing substantial mathematical exercises, rather than only reading finished slides.

3. Theory of Performance Modeling — "Extra theoretical"

Mor Harchol-Balter — CMU 15-849b, Fall 2003

Course page⁠￼

This offering is aimed more explicitly at developing the ability to derive and extend queueing analyses. It assumes comfort with probability and some prior exposure to Markov chains.

Its detailed syllabus is accessible, but the linked 2003 lecture-notes index returned an access error. The coverage below therefore describes the published syllabus; I cannot confirm which planned topics were ultimately delivered.

Its three-part structure

The syllabus divides into:

1. Operational reasoning: proofs of Little's law, response-time and forced-flow laws, bottleneck bounds and modification analysis.
2. Classical stochastic models: Markov chains, reversibility, Poisson/exponential models, single and multiple servers, finite buffers, Jackson and classed networks, closed networks, BCMP and matrix-geometric methods.
3. More general workloads and scheduling: heavy tails, general service times, renewal theory, transforms, supplementary variables, fluid approximations, task assignment and scheduling proofs. 

Much of the foundational material overlaps with the newer course. The distinctive emphasis is the mathematical machinery and its derivation.

A. Proving the accounting relationships

The operational-laws section goes beyond applying identities.

For example, a proof of Little's law can examine one execution trace. Integrating the number of jobs present over time counts accumulated job-time. Summing the residence times of individual jobs counts the same quantity, with appropriate treatment of the interval's boundaries.

This explains both the law's breadth and its conditions. It also explains why it need not assume a particular service distribution or first-come-first-served scheduling. This sample-path style of reasoning is also visible in the newer course's Little's-law handout. 

B. Deriving stationary behaviour through balance and reversibility

A Markov model makes the state explicit and describes transitions between states.

For discrete time, stationary probabilities satisfy

\boldsymbol\pi P=\boldsymbol\pi.

For continuous time, with generator Q,

\boldsymbol\pi Q=0.

These equations need normalisation and appropriate existence or stability conditions.

The deeper questions concern why a solution exists, whether it is unique, whether the process approaches it, and when time averages correspond to its probabilities. Reversibility can simplify the calculation by relating flows between pairs of states.

This is the mathematical foundation for deriving a queue's behaviour from its transition structure.

C. Product-form networks and BCMP

A major topic is determining when a network's stationary distribution can be written using factors associated with individual service centres.

A schematic form is

\pi(n_1,\ldots,n_m)
=
\frac{1}{G}\prod_i f_i(n_i),

with a normalising constant G and the appropriate state-space restrictions.

This can make a large network analytically manageable. In a closed network, however, the fixed total population still couples the queue occupancies; factorisation is not a claim that every aspect of the system is independent.

BCMP extends the range of service disciplines and distributions covered by product-form results, under its specified conditions. The associated material examines which combinations of routing, scheduling and service behaviour preserve tractability. 

D. Matrix-geometric methods

These methods address infinite state spaces with a repeating structure.

Imagine a queue that can contain arbitrarily many jobs, while the server has a finite number of modes at each queue length. The global state space is infinite, but much of it repeats.

Matrix-geometric analysis exploits that repetition: powers of a finite matrix represent probabilities across successive levels. This is a structured analogue of using a geometric sequence, and it avoids separately solving an unrelated equation for every possible queue length.

The instructor's research from this period uses the method to analyse task assignment and cycle stealing, illustrating the kind of systems problem such machinery makes accessible. 

E. General service times: renewal theory, transforms and supplementary variables

These are complementary ways to handle systems whose behaviour cannot be captured by elementary memoryless models.

Renewal reasoning

Renewal theory identifies recurring cycles. Renewal-reward reasoning obtains long-run rates from expected reward per cycle divided by expected cycle duration.

It supplies tools for analysing utilisation, residual work and inspection bias. The latter explains why a randomly timed observation encounters longer intervals more frequently than a uniformly selected completed interval would.

Transform analysis

A Laplace transform such as

\widetilde T(s)=\mathbb E[e^{-sT}]

encodes a nonnegative random variable's distribution. Its derivatives can recover moments when those moments exist.

The theoretical syllabus uses this machinery for busy periods, general-service queues and scheduling results. The objective extends beyond finding one average: it is to characterise distributions and higher moments through equations they satisfy. The newer course's transform handouts illustrate the same style of analysis. 

Supplementary variables

This addresses a modelling problem: the obvious state may omit information needed to predict what happens next.

With exponential service, how long the current job has already run does not change its remaining-time distribution. With general service times, elapsed or remaining work can matter.

A supplementary-variable method expands the state to include that information. The richer model may require differential or integral equations rather than a simple queue-length balance equation.

F. Fluid approximations

Fluid models replace individual arrivals and completions with continuous flows of work.

For a positive backlog, a simple illustrative relationship is

\frac{dq(t)}{dt}
=
\text{incoming work rate}
-
\text{processing rate},

with an additional boundary rule preventing a negative queue.

This style of approximation is useful for studying buildup, drainage and stability in systems that would be cumbersome to model job by job. It sacrifices details of individual events and fluctuations to obtain a tractable description of larger-scale behaviour.

It is a different tool from an exact stationary queueing formula: the question becomes which properties survive the approximation.

G. Scheduling proofs and realistic workloads

The syllabus emphasises deriving scheduling results, including analysis of processor sharing, priorities, preemptive last-come-first-served, shortest-job-first and shortest-remaining-processing-time policies.

The deeper issue is how changing service order redistributes waiting and response time among job classes. Preemption, knowledge of job sizes, and the chosen objective all affect what can be proved.

The published outline also proposes material on confidence intervals and measured correlated or self-similar arrival processes, but explicitly marks those sections as likely to be skipped. They should be treated as optional proposed coverage. 

How their contents fit together

Subject	Jain	CMU Analytical, 2025	CMU Theory, 2003 syllabus
Formulating an empirical evaluation	Major emphasis	Supports modelling decisions	Supports mathematical formulation
Workload representation	Histograms, PCA, clustering, sequence models	Variability, tails, age and remaining work	Heavy tails and general-service models
Confidence intervals and comparisons	Substantial treatment	Not the main emphasis	Optional proposed section
Factorial and fractional experiments	Substantial treatment	Not a central topic	Not a central topic
Simulation methodology	Construction, validation and output statistics	Simulation foundations and averages	Supports stochastic analysis
Operational laws and bottleneck bounds	Substantial treatment	Core material	Core material with proofs
Markov and queueing theory	Introduction and applications	Extensive	Extensive, with stronger proof emphasis
Scheduling and routing	Some examples and models	Major emphasis	Major emphasis with derivations
Advanced analytical machinery	Selected network methods	Transforms, renewal, phase-type introduction	BCMP proofs, matrix-geometric methods, supplementary variables and fluid approximations

Jain develops the skills needed to conduct and interpret a performance investigation. The Analytical course develops the skills needed to turn a design choice into a mathematical model. The Theory course's published programme concentrates on constructing the mathematics needed when the elementary models are insufficient.
