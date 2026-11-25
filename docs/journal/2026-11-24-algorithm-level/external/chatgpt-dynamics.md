# Dynamical systems for the decoder: Louis's question and ChatGPT's answer, verbatim

Relayed by Louis on 25 November 2026 and pasted into the Coordinator's session. It continues the Khammash thread (`chatgpt-khammash.md`). Recorded as pasted. The paste lost the hyperlinks (each shows as "⁠￼"), the figures, and inline symbols that rendered as images (for example "Here, is the state of the system, is its discrete operating mode"). The gaps are kept as they came.

## Louis's message to ChatGPT

> Okay, so if we now bring this all back to our situation, the dynamical systems perspective seems to be like they're taking measurements and they're literally trying to model the like decomposed step with you know, like a digital signal on top of an analog one. That seems quite similar to our situation. I haven't really thought that much about what parts of the digital, sorry the dynamical systems toolbox we would be reaching for to do this. But I feel like we're in the right kind of design mode here with what we're talking about here to kind of answer that question potentially for our program. And like I say, we do, we have been developing this experimental design in which to then start to build this on top of. And you know, to maybe introduce these control theory dynamical systems ideas

## ChatGPT's answer

I think what is beginning to emerge here is a way of bringing dynamical systems theory into the programme without abandoning the experimental design we’ve already developed, or prematurely deciding what mathematical form the decoder’s model should take.

In fact, the experimental design might be precisely what makes this approach viable. We are about to obtain measurements of individual activities, their interactions, their resource consumption and their responses to perturbation. Those are the ingredients from which one can construct and interrogate a dynamical system.

But there is a distinction I would make between three ambitions:

* Modelling the dynamics: understanding how the execution of the decoder evolves through time.
* Discovering the organising principles: identifying the mathematical relationships that generate its characteristic behaviour.
* Designing alternative organisations: determining what other arrangements of computation could achieve the same function with better performance.

The third is ultimately where we want to arrive. The first two provide a systematic route towards it, rather than another speculative search for algorithms.

1. The mathematical field we’re looking for is hybrid dynamical systems

Your intuition about digital and analogue behaviour is quite pertinent. There is an established mathematical discipline concerned with systems that exhibit both continuous evolution and discrete transitions.

It’s called hybrid dynamical systems theory.

The foundational modern textbook is Hybrid Dynamical Systems: Modeling, Stability, and Robustness. It develops a mathematical framework for combining differential equations, discrete events, switching conditions and feedback. Ricardo Sanfelice

This is a natural fit for our decoder.

Proposed conceptual model. The computational events change resource demand; resource availability changes when subsequent events occur.

The decoder has discrete events: blocks being scanned, jobs being issued, workers completing their work, buffers being released, admission predicates switching.

But it also has time-evolving physical quantities: temperature, resource utilisation, accumulated work, effective processing rates and memory pressure. Some are genuinely continuous; others admit useful continuous approximations.

Crucially, the two levels influence one another.

In hybrid systems notation, we can represent this using:

\begin{aligned}
\dot{x}&=f_q(x,u,d)\\
x^+&=R_{q\rightarrow q’}(x)
\end{aligned}

Here, is the state of the system, is its discrete operating mode, represents controlled interventions, and represents external disturbances.

Between events, the state evolves according to a dynamical law. When a switching condition is met, the system undergoes a discrete transition.

This is not yet a fitted model of mibz2. But it gives us a much more appropriate mathematical language than attempting to represent the entire decoder using ordinary differential equations alone.

2. We already have the beginnings of governing equations

Think back to the antithetic controller in Khammash’s paper. The important step was identifying a combination of variables whose evolution obeyed a particularly simple mathematical law.

We can begin doing something similar with the decoder.

Let:

* be the cumulative number of issued blocks.
* be the cumulative number of published, completed blocks.
* be the cumulative number of blocks acquired by the reader.

We immediately obtain:

Q_{\mathrm{unfinished}}(t)=A(t)-B(t)Q_{\mathrm{ready}}(t)=B(t)-C(t)

Adding them eliminates the intermediate quantity:

\boxed{Q_{\mathrm{ahead}}(t)=A(t)-C(t)}

This is exactly the issued-minus-taken quantity used by the decoder’s admission mechanism.

In a fluid approximation:

\dot Q_{\mathrm{unfinished}}=a(t)-b(t)\dot Q_{\mathrm{ready}}=b(t)-c(t)

And therefore:

\boxed{\dot Q_{\mathrm{ahead}}=a(t)-c(t)}

The aggregate in-flight population integrates the difference between upstream production and downstream acquisition.

This is a conservation identity, not a conjecture. Its continuous form is an approximation to the exact discrete accounting.

There is a genuine mathematical resemblance to the cancellation in Khammash’s antithetic controller. But it would be premature to conclude that the decoder possesses robust perfect adaptation or integral control in the same sense.

What matters is that we have begun deriving the system’s dynamics from structural relationships, rather than fitting arbitrary equations to measurements.

There will be other such relationships, involving buffer lifetimes, worker occupancy, resource consumption, completed output and accumulated service demand.

Discovering the useful ones is itself an interesting mathematical exercise.

3. What our measurements currently tell us — and what dynamics would add

The existing scanner profile is essentially a characterisation of different operating points.

Measured scanner profile. These are aggregate durations over a complete decode, not instantaneous dynamical trajectories.

We’ve established that different regimes exist.

But the curve does not tell us how the system moves through its internal states at any particular worker count.

Two configurations could exhibit identical total runtime while possessing entirely different dynamics. One might maintain a steady flow of work; another might repeatedly alternate between starvation and saturation.

Hypothetical queue-occupancy trajectories with similar average occupancy. Their transient behaviour and response to perturbations could be very different.

This is why I think introducing a time-dependent perspective is worthwhile. We’re moving from summarising performance to explaining its generation.

4. Which parts of the dynamical systems toolbox should we reach for?

I would be quite selective initially. There are some particularly appropriate branches of the subject.

Mathematical tool	What it would give us
Conservation laws	Exact relationships between populations, work and resources
State-space modelling	A representation of the evolving internal state of the decoder
Hybrid systems	Discrete events, switching, admission and resource-dependent execution
System identification	Estimation of unknown dynamical relationships from experiments
Perturbation analysis	How a change in one activity propagates through the others
Max-plus algebra	Mathematical treatment of timing, dependencies and synchronisation
Model reduction	Discovery of smaller sets of variables sufficient to explain performance
Optimal control	Determination of advantageous resource-allocation trajectories and policies

Of these, three particularly excite me in the context of what we’ve been discussing.

System identification — learning the governing dynamics

There is a substantial field established by researchers such as Lennart Ljung concerned with deriving mathematical models of dynamical systems from experimental input-output measurements.

His paper Perspectives on System Identification⁠￼ is a good methodological entry point. It addresses the relationship between model complexity, the information contained in measurements, and validation. doi.org

This connects directly with Jain.

Jain teaches us how to design informative experiments. System identification tells us how to use those experiments to establish a dynamical model.

There is also a much more recent mathematical approach called SINDy — Sparse Identification of Nonlinear Dynamics, developed by Brunton, Proctor and Kutz in their 2016 paper Discovering Governing Equations from Data⁠￼.

It searches for a parsimonious set of governing terms within a space of candidate mathematical functions. doi.org

I wouldn’t immediately run SINDy on the decoder. Its ordinary smooth-equation formulation is not automatically suited to our switching system. But I would be interested in its underlying philosophy: establish the known conservation structure, then discover the smallest set of additional dynamical relationships required to explain the observations.

Max-plus algebra — dynamical systems without ordinary differential equations

This is potentially enormously relevant.

In ordinary algebra, addition and multiplication are our principal operations.

In max-plus algebra:

a\oplus b=\max(a,b)a\otimes b=a+b

Why is this useful?

Because when an operation depends on multiple preceding operations, its completion time is naturally:

t_{\mathrm{finish}}

\max(t_1,t_2,\ldots,t_n)+d

It waits for all prerequisites, then incurs its own duration.

Consider a simplified decoder with one scanner, one worker and one reader. For block :

s_i=s_{i-1}+a_iw_i=\max(s_i,w_{i-1})+b_ir_i=\max(w_i,r_{i-1})+c_i

These equations describe the timing of the whole pipeline, assuming unlimited buffering and prescribed service durations.

What appears nonlinear in ordinary arithmetic becomes linear in max-plus algebra.

The foundational work is Baccelli, Cohen, Olsder and Quadrat’s Synchronization and Linearity: An Algebra for Discrete Event Systems⁠￼, freely available through TU Delft. It develops an algebraic theory of synchronisation and timed event systems. repository.tudelft.nl

Our full decoder has dynamic dispatch, routing and contention, so it will require extensions or hybridisation of this approach.

Nevertheless, this is a possible route towards manipulating computational organisation algebraically, rather than merely simulating it.

Model reduction — finding the right mathematical coordinates

This is the part that most closely resembles the beautiful mathematics of Khammash’s paper.

Can we find a function of the decoder’s detailed internal state,

z=h(x),

such that its important performance dynamics can be described directly in terms of ?

Ideally:

\dot z=F(z,u,d)

without needing to retain every microscopic state variable in .

This is not simply dimensionality reduction or PCA. The question is whether the reduced variables possess sufficiently self-contained dynamics.

For example, two decoder states can have the same queue occupancy but radically different future behaviour because one contains a long-running head-of-line block.

That would establish that occupancy alone is an inadequate coordinate.

We might discover that occupancy, outstanding work, the remaining demand of the next required block and a measure of resource contention together form a much more explanatory state.

The interesting work is to establish which variables are necessary, which can be eliminated and which combinations reveal simpler laws.

5. How I would augment the experimental design

I would not replace the existing plan. I would introduce one additional dimension to it: the time profile of our interventions.

At present, our proposed interventions largely change the demand of an activity by a specified percentage throughout a decode.

That is excellent for establishing its overall influence. But control theory gives us additional ways of interrogating a system.

Sustained intervention

Establishes the overall response to a changed service demand.

Temporary pulse

Reveals propagation, recovery and transient coupling.

Repeated excitation

Helps distinguish different dynamical timescales and interactions.

For example, imagine temporarily increasing the scanner’s demand by 20% over a specified sequence of blocks.

Instead of measuring only the resulting whole-decoder runtime, we measure:

* How quickly the work queue depletes.
* When worker starvation begins.
* Whether the downstream reader temporarily continues at its original rate.
* How long the system takes to recover.
* Whether the intervention changes resource contention elsewhere.

Then we perform the corresponding experiment on a worker activity or the reader.

This gives us something much richer than a scalar sensitivity coefficient. It gives us a time-dependent response matrix.

G_{ij}(t,\tau)

\frac{\delta y_i(t)}{\delta u_j(\tau)}

Conceptually, this expresses how a perturbation of activity at time influences observable at a later time .

Near switching boundaries, we would initially estimate finite intervention responses rather than assume these derivatives exist.

This is the experimental bridge between Jain and dynamical systems theory.

6. What becomes possible once we have that model?

This is where I would bring Ma et al. and Khammash back into the picture.

Suppose we establish a sufficiently good dynamical model of the decoder. Then we can begin investigating alternative structural organisations, not merely different parameter values.

For example, we could specify a desired functional property:

Complete decoding at close to the machine’s achievable throughput, while maintaining bounded memory, despite changes in the relative demand of the component activities.

We can then ask what mathematical organisation would be necessary for that property.

Is a permanently separated scanner and worker pool optimal? Is dynamic redistribution of work necessary? Can information generated in one activity eliminate work in another? Are there decompositions that make the system intrinsically less sensitive to changes in workload?

Notice that this doesn’t impose biological motifs on the decoder. We would be deriving the relevant structural properties from the decoder’s own mathematical requirements.

And, importantly, the answer might lie at different scales.

Scale	Possible discovery
Whole decoder	A better organisation of computational resources
Between activities	A different factorisation of the decoding process
Within an algorithm	A representation that changes the computational dependencies
Mathematical primitive	A different law of composition, contraction or information propagation

The last two are particularly relevant to our ambition of finding another step comparable to traversal → Rauzy–Veech induction.

We mustn’t assume that a better dynamical model automatically produces a new inversion algorithm. But it could give us a much more principled way of deciding which properties a new algorithm should possess, and where a change in mathematical representation would be valuable.

7. There is already a school of control theory for computing systems

I’ve also found a textbook that could sit very naturally alongside Jain and Harchol-Balter:

Hellerstein, Diao, Parekh and Tilbury (2004) — Feedback Control of Computing Systems⁠￼.

Its contents include:

* Chapter 2: Model construction.
* Chapter 3: Z-transforms and transfer functions.
* Chapter 4: System modelling with block diagrams.
* Chapters 5–6: First-order and higher-order systems.
* Chapter 7: State-space models.
* Chapters 8–10: Proportional, PID and state-space feedback control.

The book grew out of research at IBM into applying control theory to computer-system modelling and resource management. It is not a biological analogy; it is an established application of dynamical systems mathematics directly to computing systems. Wiley +1

I would mine this in much the same way we’ve been mining Jain’s course.

I would initially give less priority to chaos theory, bifurcation analysis, Koopman operators or sophisticated nonlinear control. Those might become interesting once we understand more of the decoder’s dynamics, but we don’t yet have evidence that those particular tools are necessary.

8. The research programme that is emerging

01

Experimental characterisation

Jain

Establish measurements, demands and causal responses.

02

Operational and resource constraints

Harchol-Balter

Establish feasibility bounds and resource relationships.

03

Dynamical characterisation

Ljung · Hellerstein · Goebel · Baccelli

Derive governing equations, switching laws and response dynamics.

04

Structural design-space exploration

Ma · Tang · Sharpe

Investigate alternative mathematical organisations serving the desired function.

05

Explanation of the solution space

Khammash · Araujo · Liotta

Identify necessary properties, sufficient constructions and the relationships between them.

I would introduce the third stage explicitly into our existing programme, without delaying the characterisation experiments already planned.

The immediate practical requirement is to ensure that stage 1 retains properly synchronised event traces, and that stage 2 can accommodate a small number of time-localised interventions. We can decide whether more extensive dynamical experiments are warranted after examining those traces.

We haven’t yet measured the relevant trajectories or established which state representation is sufficient. Everything proposed here is therefore a methodological direction, not a finding about the decoder.

But I think there is something particularly compelling about where this is going.

We are beginning to give the mathematical discovery process an experimental foundation. Not simply measuring an algorithm after somebody invents it, but using measurements to uncover the governing relationships of computation, and using those relationships to inform the discovery of alternative mathematical organisations.

That is much closer to the kind of research I think we’ve been trying to formulate.
