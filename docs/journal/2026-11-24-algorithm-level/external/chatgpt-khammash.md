# Khammash, "Perfect adaptation in biology" (Cell Systems, 2021): ChatGPT's explanation, verbatim

Relayed by Louis on 25 November 2026 and pasted into the Coordinator's session. It continues the Alon thread (`chatgpt-alon-motifs.md`, three answers). Recorded as pasted. The paste lost the hyperlinks (each shows as "⁠￼"), inline symbols that rendered as images (for example "the dynamics of active vitamin D, , as"), and the interactive figure's controls (only their labels remain). The gaps are kept as they came. The explanation is ChatGPT's reading of the paper, not the Coordinator's: the paper is to be read in full before any of it is relied on.

## Louis's message to ChatGPT

> oh nice ok so I am reading this one - Perfect adaptation in biology https://www.cell.com/cell-systems/fulltext/S2405-4712(21)00206-4  Can you explain it to me? I am very excited at the presence of dynamical systems modelling… but also I do entirely agree with your framing now of explaining the solution space, this is brilliant… but explain this paper to me properly, it’s open access. i see this figure in particular

(The figure Louis means is the paper's Figure 5, the five controller motifs.)

## ChatGPT's answer

The paper is Perfect Adaptation in Biology⁠￼, by Mustafa H. Khammash, published in Cell Systems in June 2021. Khammash works at the intersection of control theory, dynamical systems, systems biology and synthetic biology.

What makes this paper particularly relevant to our discussion is that it goes beyond identifying successful network topologies. It asks whether a desired behaviour imposes mathematically necessary constraints on the organisation of any system capable of producing it.

That is an extraordinarily interesting change in perspective.

Ma et al. explored the space of possible networks computationally. Khammash approaches the problem through dynamical systems, algebra and control theory. His objective is to identify what different successful mechanisms are secretly doing in common, even when their physical implementations look completely different.

I obtained the full open-access manuscript from ETH Zürich⁠￼, since the publisher’s page blocked direct access.

1. What exactly is perfect adaptation?

Imagine a system that maintains an output at a particular value. Something in its environment changes permanently. Its output initially responds, but eventually returns exactly to its original value, despite the disturbance remaining present.

Illustrative dynamics: the environmental change persists, but its long-term effect on the perfectly adapting output disappears.

There are actually three distinct properties:

Property	Meaning
Homeostasis	The output remains approximately within a desirable range.
Perfect adaptation (PA)	The output returns exactly to its setpoint following a persistent disturbance.
Robust perfect adaptation (RPA)	Perfect adaptation survives a specified class of changes to the system’s own parameters or dynamics.

That third property is the subject of the paper. It means that the behaviour is not merely an accidental consequence of well-chosen parameters. There is something about the system’s mathematical organisation that protects the behaviour. research-collection.ethz.ch +1

2. The biological starting point: two seemingly unrelated systems

Khammash begins with two examples operating at completely different biological scales. The purpose is to show that they embody the same mathematical principle.

Bacterial chemotaxis — Figure 1

When an E. coli bacterium detects an attractant, it temporarily reduces how frequently it tumbles, allowing it to travel further in a favourable direction.

But if the attractant concentration subsequently remains constant, the bacterium needs to recover its original sensitivity. Otherwise, it would remain permanently stimulated and lose its ability to detect further improvements in its environment.

The underlying mechanism involves methylation and demethylation of receptors by two enzymes, CheR and CheB.

The interesting mathematical step is to stop considering active and inactive methylated receptors separately and instead introduce their sum:

X_T=X_m+X_m^*

Adding their dynamical equations causes several reaction terms to cancel, leaving:

\frac{dX_T}{dt}=V_RR-k_BBX_m^*

At equilibrium, the derivative vanishes. Therefore:

\boxed{X_m^*=\frac{V_RR}{k_BB}}

Notice what is absent from this expression: the attractant concentration.

The equilibrium receptor activity is independent of the stimulus, under the model’s assumptions. The integral feedback explanation of this mechanism was developed by Yi, Huang, Simon and Doyle in 2000, building on the work of Barkai, Leibler and Alon. pubmed.ncbi.nlm.nih.gov +1

Calcium homeostasis — Figure 2

The second example concerns calcium concentration in mammalian blood. Even during prolonged calcium loss, such as lactation, regulatory mechanisms compensate through intestinal absorption and bone resorption.

The simplified model in the paper gives the dynamics of active vitamin D, , as:

\frac{dD}{dt}=k_0k_3(R_0-Ca)

where is the desired calcium concentration.

At equilibrium:

0=k_0k_3(R_0-Ca)

Hence, assuming positive rate constants:

\boxed{Ca=R_0}

Again, the external disturbance disappears from the equilibrium condition. pubmed.ncbi.nlm.nih.gov +1

These examples are interesting together because their physical mechanisms have very little in common. But their mathematical structures are essentially identical:

\dot z=k(r-y)

One system regulates bacterial receptor activity, the other blood calcium. Both implement integration of a deviation from a desired state.

The biological complexity conceals a common mathematical computation.

3. Why integral feedback is so powerful

This is worth understanding properly before looking at Figure 5.

Suppose we have a system with output and desired value . Define its error as:

e(t)=r-y(t)

Now introduce an auxiliary dynamical variable :

\dot z=e(t)

Integrating this equation gives:

z(t)=z(0)+\int_0^tr-y(s)\,ds

The variable contains a memory of all previous deviations from the desired output.

If acts back upon the system so as to correct the error, we have integral feedback.

The integrator accumulates error and uses its stored state to regulate the system.

The reason this produces perfect adaptation is remarkably simple.

If the system settles to an equilibrium, then must stop changing. But its derivative is precisely the error:

\dot z=0\implies r-y=0

Consequently:

\boxed{y=r}

It does not matter how complicated the system being controlled is. Provided the integral structure survives and the interconnected system reaches a suitable equilibrium, this equation constrains its output.

That is an example of a structural guarantee rather than a numerical optimisation.

An interactive example

I’ve constructed a simple illustrative dynamical system below to demonstrate the principle. This is not experimental data from the paper.

The system is governed by:

\dot y=-y+v+d

where is a persistent disturbance and is the control action.

You can vary the disturbance and feedback strength, and compare proportional control with integral control.

CompareProportionalIntegral

Proportional

Integral

Dashed: target = 1

Persistent disturbance

2.0

Feedback gain

1.0

Proportional equilibrium

2.000

Integral equilibrium

1.000

Increasing proportional gain reduces the residual error. Integral control eliminates it asymptotically, although its transient response may oscillate.

Try increasing the gain substantially. You’ll notice that the integral controller’s transient behaviour changes considerably, but its eventual equilibrium remains the same.

This distinction between the invariant function of a system and its variable dynamical performance becomes important later in the paper.

4. The mathematical characterisation — Section 3

This is where Khammash begins to explain the solution space itself.

Consider a general nonlinear dynamical system:

\dot{\mathbf{x}}=f(\mathbf{x},u)y=g(\mathbf{x})

Here:

Symbol	Meaning
	Vector of all internal state variables
	Environmental input or disturbance
	Output of interest
	Internal dynamical laws
	Function extracting the output

For each constant input , suppose there exists a stable, regular equilibrium .

Then:

f(\mathbf{x}_u,u)=0

Perfect adaptation means:

g(\mathbf{x}_u)=r

for every admissible constant input.

This can be expressed as a sensitivity condition:

\boxed{\frac{dy^*}{du}=0}

In other words, changing the input moves the internal equilibrium, but does not change the equilibrium output.

The Jacobian enters

Differentiate the equilibrium equation with respect to :

\frac{\partial f}{\partial \mathbf{x}}
\frac{d\mathbf{x}_u}{du}
+
\frac{\partial f}{\partial u}
=0

Introduce:

A=\frac{\partial f}{\partial \mathbf{x}},
\qquad
b=\frac{\partial f}{\partial u},
\qquad
c=\frac{\partial g}{\partial \mathbf{x}}

Here is the Jacobian of the system at equilibrium.

Assuming is nonsingular:

\frac{d\mathbf{x}_u}{du}=-A^{-1}b

Therefore:

\frac{dy^*}{du}=-cA^{-1}b

The condition for perfect adaptation becomes:

\boxed{cA^{-1}b=0}

Or equivalently, in determinant form:

\boxed{
\det
\begin{pmatrix}
A&b\\
c&0
\end{pmatrix}=0
}

This condition must hold throughout the relevant equilibrium family.

The determinant formulation links Khammash’s treatment to the work of Araujo and Liotta. research-collection.ethz.ch +1

This is the critical methodological step. We started with a desired behaviour expressed in ordinary language, and we have derived an algebraic constraint that the underlying dynamical system must satisfy.

We now have something to analyse, rather than merely something to simulate.

Structural robustness versus accidental equality — Figure 3

Suppose two systems display exactly the same response to every input in their nominal configuration.

Khammash shows that one can be robustly adaptive while the other is merely finely tuned.

Consider the following two systems from Figure 3.

System A

\dot x_1=x_2\dot x_2=-2x_1-3x_2+uy=2x_2

System B

\dot x_1=-2x_1+2u\dot x_2=-x_2+2uy=2x_1-x_2

Both systems have the same nominal transfer function:

G(s)=\frac{2s}{s^2+3s+2}They are indistinguishable by their nominal input-output behaviour, assuming zero initial conditions.

But inspect their equilibrium conditions.

In System A:

\dot x_1=0\implies x_2=0\implies y=0

This follows from the form of the equations.

In System B:

x_1=u,\qquad x_2=2u

so:

y=2u-2u=0

This follows from numerical cancellation.

Change the input coefficient in System B’s second equation from 2 to 2.2, and suddenly:

y^*=-0.2u

Its perfect adaptation disappears.

Two systems can implement exactly the same nominal function while possessing fundamentally different structural properties.

This is particularly relevant to the study of equivalent algorithms. Merely demonstrating equivalent behaviour does not establish that two implementations have the same robustness, cost structure or response to perturbations.

5. Your image — Figure 5, explained in full

This figure is the physical realisation of the mathematical argument.

The grey cloud represents an arbitrarily complicated biological network. The purple area represents a comparatively small molecular controller attached to that network.

The question is:

How can biochemical reactions implement the mathematical operation of integration?

The difficulty is that molecules have nonnegative concentrations. A cell cannot simply allocate a signed floating-point variable and integrate an error into it.

The five panels represent different ways of implementing that computation using permissible biological processes.

For every panel, let:

r=\frac{\mu}{\theta}

where encodes the reference and is the sensing coefficient.

Incidentally, there appears to be a small notation error in the caption of your screenshot: it says , but the equations and the ETH manuscript give .

Top left — Outflow-driven zero-order controller

A molecule is produced at constant rate , while the output stimulates its removal.

\dot Z=\mu-\theta X_L

Rearranging:

\dot Z=\theta(r-X_L)

Therefore, directly integrates the error.

If the output is too low, accumulates. If the output is too high, decreases.

At equilibrium:

X_L=\frac{\mu}{\theta}

The crucial physical requirement is that the removal process be zero-order with respect to : its rate must not depend on the amount of present over the relevant operating range.

An enzyme operating near saturation can approximate this behaviour.

Top middle — Inflow-driven zero-order controller

This is the complementary arrangement.

The output stimulates the production of , while is removed at a constant rate.

\dot Z=\theta X_L-\mu

Equivalently:

\dot Z=-\theta(r-X_L)

This implements integration with the opposite sign. The surrounding feedback must have the appropriate regulatory direction.

The equilibrium is unchanged:

X_L=\frac{\mu}{\theta}

The physical mechanism is different, but its essential mathematical operation is equivalent.

Top right — Autocatalytic controller

This one is considerably more interesting.

Here stimulates its own production and is degraded through a reaction involving :

\dot Z=\mu Z-\theta X_LZ

Factorise:

\dot Z=Z(\mu-\theta X_L)

At first glance, this does not look like the previous integrators. The dynamics are multiplicative rather than additive.

But for , divide by :

\frac{\dot Z}{Z}=\mu-\theta X_L

And recognise that:

\frac{d}{dt}\log Z=\frac{\dot Z}{Z}

Therefore:

\boxed{
\frac{d}{dt}\log Z=\mu-\theta X_L
}

The system integrates the error in logarithmic coordinates.

This is a genuinely beautiful mathematical observation. The physical variable is not itself the integral; an appropriate transformation of the variable reveals the integral.

This is precisely the kind of thinking about mathematical representations that we’ve been discussing in the algorithmic work.

There is, however, a serious qualification: the logarithmic coordinate ceases to exist at . We will return to this when considering stochastic dynamics.

Bottom left and bottom right — Antithetic integral feedback

These are the most important motifs in the paper.

Rather than using one controller molecule, introduce two complementary molecular species:

* , produced at a constant reference rate .
* , produced at a rate proportional to the observed output, .

The two molecules bind or annihilate one another in a one-to-one reaction.

Both controller species are individually nonnegative, but their difference can represent a signed quantity.

The governing equations are:

\dot Z_1=\mu-\eta Z_1Z_2\dot Z_2=\theta X_L-\eta Z_1Z_2

They look nonlinear.

But now subtract them:

\dot Z_1-\dot Z_2

\mu-\theta X_L

The nonlinear reaction term vanishes entirely.

Define a new coordinate:

z=Z_1-Z_2

Then:

\boxed{\dot z=\mu-\theta X_L}

We have constructed an exact integrator using two nonnegative molecular populations and a mutual annihilation reaction.

Furthermore, , the reaction rate of mutual annihilation, disappears from the integral equation. It can affect the dynamics without changing the setpoint, provided stability is preserved.

The N-type and P-type variants use different species to actuate networks with different net regulatory signs. Their underlying integration mathematics is identical. doi.org +1

What the five motifs have in common

Motif	Coordinate containing the integral
Outflow-driven	
Inflow-driven	
Autocatalytic	
N-type antithetic	
P-type antithetic	

All five can be expressed in the common mathematical form:

\boxed{
\frac{d}{dt}h(\mathbf{x})=\mu-\theta X_L
}

That function is the hidden mathematical object.

Instead of looking for a particular molecular mechanism, we can look for functions of the system’s state that possess particular dynamical properties.

This is what makes Figure 5 more than a taxonomy. It is a collection of different mathematical realisations of the same functional principle.

6. The deeper theorem — The internal model principle

Section 4.1 moves from sufficiency to necessity.

Until now, we have established that integral feedback can produce robust perfect adaptation.

But is it possible to achieve the same behaviour by an entirely different feedback mechanism?

For the specified class of linear feedback systems, Khammash demonstrates that an integrator is not merely one convenient solution. It is necessary.

This is connected to one of the foundational theorems of control theory:

Francis and Wonham (1976), The Internal Model Principle of Control Theory⁠￼.

The intuition is that if a controller is to robustly reject a disturbance belonging to a particular class, its own dynamics must incorporate an appropriate model of that class.

Disturbance to reject	Required internal dynamics in the classical setting
Constant disturbance	Integrator
Linearly increasing disturbance	Repeated integration
Sinusoidal disturbance	Oscillatory dynamics at the disturbance frequency

An integrator can maintain a nonzero corrective action even when the measured error has become zero. It remembers the correction accumulated while the error was present.

This is why the controller need not know the disturbance explicitly.

The full internal model principle has specific hypotheses, and the paper’s necessity proof is for a particular linear feedback configuration. It should not be read as a theorem that every adaptive nonlinear system must contain a literal feedback integrator.

Nevertheless, consider the conceptual power of the result:

Knowing the function that a system performs can tell us something mathematically necessary about its internal organisation.

One can begin with the desired behaviour and work backwards to the permissible mechanisms.

7. An alternative mathematical solution — Incoherent feedforward

Section 5 provides a second architecture for achieving adaptation.

Instead of waiting to detect a deviation in the output, the system receives the input through two opposing pathways.

One pathway stimulates the output. The other counteracts its effect.

Take the following model of the sniffer circuit discussed in Figure 6:

\dot x=\alpha u-\delta x\dot y=\beta u-\gamma xy

The steady-state value of is:

x^*=\frac{\alpha u}{\delta}

Substituting into the second equation:

0=\beta u-\gamma\frac{\alpha u}{\delta}y^*

Cancel , for positive inputs:

\boxed{y^*=\frac{\beta\delta}{\alpha\gamma}}

Again, the input has disappeared.

But the mechanism differs from ordinary feedback. The disturbance propagates through two pathways whose effects counterbalance one another.

The structure requires compatible functional dependencies in those pathways. It is not robust to arbitrary changes in every possible reaction law.

There is a particularly interesting distinction here: the equilibrium can depend on the model’s rate constants, while remaining independent of the input for a range of those constants.

Thus, robustness need not mean that every parameter leaves the setpoint unchanged. One must specify exactly which property is preserved and against which class of perturbations.

8. Figure 7 — Explaining the entire solution space

I suspect this is the section you will find most exciting.

Khammash explicitly discusses both Ma et al. (2009) and Araujo and Liotta (2018).

The difficulty with Ma et al.’s exhaustive approach is combinatorial explosion. For labelled nodes, allowing positive, negative or absent connections, including self-connections, the nominal topology count is:

3^{n^2}

Nodes	Possible signed topologies
2	81
3	19,683
4	43 million
5	847 billion
6	150 quadrillion

Enumeration rapidly becomes impractical.

Araujo and Liotta take a different approach. They examine the algebraic adaptation condition and derive restrictions on how the underlying network can be organised.

Their classification identifies two architectural families, which they describe as a topological basis.

Opposer module

A feedback architecture incorporating integral action through specialised nodes.

Balancer module

Multiple pathways whose contributions combine in a structurally adaptive manner.

Simplified illustrations of the two module classes, not the complete connectivity and kinetic requirements of the original theorem.

These generalise the feedback and feedforward families discovered by Ma et al. Under the deterministic mathematical framework of the 2018 paper, suitable interconnections of these modules characterise the space of adaptive network architectures, including networks much larger than could be enumerated. nature.com +1

The modules also contain requirements on reaction kinetics. They are not just abstract graph shapes.

Look at the progression of the research:

Stage	Achievement
Observation	Biological systems possess perfect adaptation.
Modelling	Express their behaviour through dynamical equations.
Enumeration	Discover which small structures exhibit adaptation.
Algebraic characterisation	Derive the determinant condition.
Classification	Identify the underlying families of solutions.
Composition	Establish how those families can form larger systems.

Enumeration was a means of discovery, not the final intellectual product.

The final product is an explanatory theory of the solution space.

9. Stochastic dynamics change the solution space again

Section 7 introduces an additional physical constraint: molecules are discrete, and their reactions occur randomly.

A deterministic differential equation describes concentrations as smoothly varying real numbers. Actual cells may contain only a handful of copies of a particular molecule.

This matters enormously.

Consider the autocatalytic controller:

\dot Z=Z(\mu-\theta X_L)

In a deterministic model, a positive initial value can remain positive at every finite time. But in the corresponding stochastic reaction system, the molecular population can reach zero.

Once it does, autocatalytic production cannot restart because there is no molecule left to catalyse its own production.

Zero becomes an absorbing state.

Deterministic concentration

Stochastic molecular population

Illustrative trajectories. A continuous mathematical representation can conceal discrete failure mechanisms.

The antithetic controller, however, has a stronger property.

Think about its individual reaction events rather than its differential equations:

Reaction	Change in
Production of one molecule	+1
Production of one molecule	−1
Annihilation of one molecule of each species	0

The cancellation survives at the level of individual events. It is not an artefact of averaging concentrations.

Taking expectations:

\frac{d}{dt}\mathbb EZ_1-Z_2

\mu-\theta\mathbb EX_L

Assuming the stochastic system has the required stationary behaviour and finite moments:

\boxed{\mathbb EX_L=\frac{\mu}{\theta}}

The controller achieves perfect adaptation in the mean.

Under appropriate ergodicity conditions, it also achieves the setpoint as the long-term time average of an individual cell’s fluctuating output.

The particularly strong result, from Aoki et al. (2019)⁠￼, is that the antithetic architecture is necessary within their specified class of stochastic biomolecular feedback controllers achieving robust perfect adaptation. nature.com +1

This means that adding a more realistic constraint on the system changes which mathematical solutions are admissible.

Some structures that work in the continuous abstraction fail in the actual physical setting. Others retain their essential properties.

There is a remarkable parallel here with algorithm research. Mathematical equivalence in an abstract computational model need not imply equivalence under realistic machine constraints.

10. From mathematical guarantees to engineering optimality

The final part of the paper turns to synthetic biology.

Researchers constructed antithetic controllers in living cells and tested them against persistent disturbances. The experiments discussed around Figure 9 demonstrate the difference between controlled and uncontrolled gene expression.

But Khammash makes an important further distinction.

Achieving perfect adaptation does not necessarily make a controller optimal.

Two networks can both converge to the correct value while differing enormously in:

* Speed of convergence.
* Overshoot and oscillation.
* Sensitivity to transient disturbances.
* Magnitude of stochastic fluctuations.
* Physical resource consumption.
* Stability margins.

The mathematical architecture determines whether the system possesses the required functional property. Its particular realisation determines how well it performs that function.

This opens a second design problem. Once you have characterised the admissible structural families, you can investigate their performance trade-offs.

A controller combining proportional and integral feedback, for example, may improve the transient response without sacrificing the equilibrium guarantee. The paper also discusses fundamental limitations involving speed, noise and robustness. research-collection.ethz.ch +1

This is where I think the resonance with Jain becomes particularly meaningful.

Jain supplies a systematic methodology for investigating performance. Khammash supplies a methodology for characterising the structural families within which that performance investigation takes place.

11. What Khammash is proposing philosophically

The concluding section is worth reading particularly carefully.

He is questioning whether mechanistic understanding alone constitutes a sufficient explanation of a complex system.

You could identify every molecule, every regulatory reaction and every kinetic parameter within a biological network, yet still fail to understand why that network possesses its characteristic behaviour.

Conversely, identifying the mathematical constraint responsible for the behaviour can explain something that remains obscure even in a complete mechanistic description.

He calls for:

a new explanatory paradigm for biology. research-collection.ethz.ch

He also explicitly raises information theory and filtering theory as possible sources of analogous structural constraints for other biological functions.

So he is not merely proposing integral control as a useful biological technique. He is proposing a more general programme of scientific enquiry.

Specify the desired function

What precisely must the system achieve?

Derive necessary mathematical conditions

What must hold for that function to be possible?

Characterise the solution space

What structurally distinct mechanisms satisfy those conditions?

Investigate physical realisability

Which mechanisms survive the constraints of the actual system?

Establish performance and optimality

Which admissible realisations best serve the purpose?

12. What I would take from this for our algorithmic research

I don’t think we should immediately try to implement integral feedback in mibz2. That would risk trivialising the connection again.

There might be applications to adaptive worker allocation and backpressure, but those would be comparatively conventional control-engineering problems.

The deeper opportunity is methodological.

Consider what Khammash did with the autocatalytic controller:

\dot Z=Z(\mu-\theta X_L)

He changed coordinates:

h(Z)=\log Z

And revealed a mathematical operation that was not apparent in the original formulation.

Or consider what he did with the antithetic controller:

(Z_1,Z_2)\longmapsto Z_1-Z_2

He found a representation in which unwanted complexity cancelled exactly.

In both cases, the key operation was not apparent from the mechanistic decomposition. It emerged through mathematical reorganisation.

This feels particularly relevant to our transition from LF traversal to Rauzy–Veech induction, where a different mathematical description of the same underlying permutation made a different form of computation accessible.

But there is another, more ambitious possibility.

Rather than asking exclusively:

What mathematical framework might give us a faster inversion algorithm?

We could also ask:

What mathematical properties would every sufficiently efficient inversion algorithm have to possess, and can we characterise the different ways those properties might be realised?

That does not require us to commit to network motifs, feedback, universality or biological analogies. It requires us to identify the appropriate mathematical conditions for our particular algorithmic purpose.

We have not established those conditions yet. But Khammash gives a concrete example of how productive such an investigation can become.

13. The next papers I would read

There is actually a subsequent development of this research that goes beyond the 2021 perspective.

Xiao and Doyle (2018)

Robust Perfect Adaptation in Biomolecular Reaction Networks⁠￼

This is explicitly concerned with finding mathematically suitable integral variables under the physical constraints of biochemical reaction networks. Khammash cites it as a systematic route to generating additional motifs. chemaoxfz.github.io

Araujo and Liotta (2018)

The Topological Requirements for Robust Perfect Adaptation in Networks of Any Size⁠￼

The detailed derivation of the topological basis. This is the natural next reading if Figure 7 and the classification of the solution space interest you.

Gupta and Khammash (2022)

Universal Structural Requirements for Maximal Robust Perfect Adaptation in Biomolecular Networks⁠￼

This might be the most important follow-up. They introduce maximal robust perfect adaptation and characterise its requirements through linear algebraic conditions on reaction stoichiometry, distinguishing the deterministic and stochastic cases. pmc.ncbi.nlm.nih.gov

Briat, Gupta and Khammash (2016)

Antithetic Integral Feedback Ensures Robust Perfect Adaptation in Noisy Biomolecular Networks⁠￼

The original mathematical investigation of the two-species controller. Particularly worthwhile if you want to understand the stochastic dynamical systems theory underpinning the bottom two panels of your image.

Of these, I am especially interested in Xiao and Doyle and the 2022 Gupta and Khammash paper.

What we’ve uncovered here is a research tradition that doesn’t merely propose mathematical models of complex systems. It uses mathematics to discover the permissible forms of organisation through which a function can be achieved.

That is a much stronger idea than modelling an existing implementation, and I think it deserves serious consideration as a methodological direction for our research.
