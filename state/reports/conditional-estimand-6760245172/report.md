# Conditional-window inference: REVISE §2.10

Inbox [6760245172](https://github.com/lmmx/mibz2/pull/162#issuecomment-6760245172).
Exact draft: **295cfd2bb0ac77e4d368e5272a3ee1671666a857**, #316.
Pinned instructions: a2da49638ac54d7763de1c167bd054f67ffdc763.

**REVISE before presenting this as an adoptable policy.** The diagonal-covariance variance identity is correct. The claimed structural independence, flat-path worst-case calibration and diagnostic power-loss bounds do not follow. D085's narrow iid Gaussian theorem survives; this proposed extension to an arbitrary path with heteroskedastic noise does not. No change to §2.9's recorded negative calibration, #333's tooling or any frozen outcome.

## 1. Fresh processes do not imply independence conditional on their entire aftermath

The inspected ab.rs genuinely creates a fresh child for each run. It also chooses each launch after an observed-load loop, waits for that child, then proceeds to the next gate. The proposed H includes launch times, cache/thermal state and state left by both arms. These are potentially descendants of execution randomness, not merely pre-existing causes.

An exact countermodel: let fresh-process innovations U_1,...,U_B be independent standard normals. Let outside state retain their successive differences D_i=U_i-U_(i-1), i=2,...,B, and condition on H=sigma(D). Then

    U | D = E[U | D] + Z 1,
    Z = mean(U),   Var(Z | D)=1/B,
    Cov(U | D)=11'/B.

The only original randomness was independent per-process randomness; the recorded outside state creates conditional dependence. For B=2, conditioning on U_2-U_1 leaves conditional covariance 1/2. This is a counterexample to the structural deduction, not a model claimed to describe this runner. Adding “all channels outside the process are in H” does not prove conditional factorisation: conditioning on a common consequence can open dependence. An exhaustive state that instead reveals every outcome can make residuals degenerate; that is also not the asserted useful independent-noise model.

The distinction is standard collider conditioning; see Pearl's [primary exposition](https://bayes.cs.ucla.edu/BOOK-2K/d-sep.html). A gate based only on genuinely exogenous host inputs could fit an independent-innovation model. A gate adapted to prior execution could fit a sequential filtration with justified innovations. Neither fact licenses conditioning on the entire future path and declaring independence. Define a generative factorisation or a predictable filtration and prove the relevant residual property. Merely assigning unexplained dependence to a latent path is not that proof.

The covariance identity itself needs only zero off-diagonal covariance, not full independence:

    E[s²/B | H] - Var(mean(Y) | H)
      = [g'Pg + tr(Sigma) - 1'Sigma1] / [B(B-1)].

Diagonal Sigma gives nonnegative expected excess. That says nothing by itself about a Student-t tail.

## 2. Flat path is NOT the worst case with independent heteroskedastic Gaussian noise

This is a new, independent obstruction even if A2 is granted.

Take B=24 and fixed mean vector

    g_1=23/24; g_2=...=g_24=-1/24; mean(g)=0.
    epsilon_1=X~N(0,1); epsilon_2=...=epsilon_24=0.

The target is zero, the null boundary after translating by log(1.02). Residuals are independent Gaussian (the limiting singular case). Yet

    mean(Y)=X/24,  s/sqrt(24)=|X+1|/24.
    NI falsely passes iff X + t |X+1| <= 0,
    t=t(.95,23)=1.713871527747048.

The probability is

    Phi(-t/(t+1)) - Phi(-t/(t-1)) = 0.2556700363.

For the flat path with the same noise it is zero: X+t|X| cannot be negative. Small positive variances for the remaining batches make the example nonsingular and preserve the failure. Expected variance overestimation coexists with a badly anti-conservative lower tail because numerator and estimated denominator are coupled.

The attached synthetic probes use fully nonsingular versions. In one 100,000-window batch-level simulation, variance ratio 43:1 gives 3.966% false acceptance on the flat path and 10.494% on the fixed nonflat path. Ratio 10:1 already gives 5.921% on that path. These are stress models, not fitted pilot laws.

More importantly, I implemented the **combined proposed rule** at cycle level: 24 batches ×30 independent normal cycles, first batch variance43, other batches variance1; add the above g scaled by sqrt(43/30) to the cycle means. Compute the NI t bound, the frozen slope cutoff4.3806308666, and the exact pinned S5 function/cutoff. Every window has the same fixed path and diagonal conditional covariance.

| Synthetic per-cell model, 100,000 windows | Raw false NI acceptance | S5 invalid | False acceptance AND both vetoes pass |
|---|---:|---:|---:|
| iid Gaussian, flat | 4.951% | 0.028% | 4.950% |
| independent Gaussian, variance ratio43, flat | 3.943% | 10.727% | 3.540% |
| same variances, fixed nonflat g | 10.746% | 10.727% | **9.602%** |
| iid cycles plus unconditioned common N(0,1) shift | 47.657% | 0.028% | 47.637% |

For the decisive third row, conditional on both diagnostics passing the false-acceptance rate is 10.758%. Its unconditional 9.602% has binomial MC SE about0.093 percentage points. The common-shift row is a separate A2 violation illustrating diagnostic blindness; it is not needed to refute A3 with independent noise.

These are **per-cell** results, not the probability that all42 cells pass the overall gate. One can pair a boundary cell with comfortably safe remaining cells; an invalid marginal confidence claim is not repaired by adding more cells. No pilot replay, decoder timing or qualification data was generated.

## 3. S5 tests within-batch shape; it cannot certify A2

For every vector of batch constants a_b,

    S5(c_bj + a_b)=S5(c_bj),

because each batch is demeaned. It is exactly blind to any batch-level random shift, however correlated across batches. A positive common residual can invalidate the bound while S5 and the trend slope both stay unchanged. Negative correlations at a few inspected lags similarly do not prove that the full sum of cross-batch covariances is nonpositive.

The cutoff0.1043904263 is a simulated upper tail under **homoskedastic iid Gaussian cycle noise with a constant mean inside each batch**, not the broader arbitrary-path/heteroskedastic model. Bonferroni needs no independence between cells, but it does need each cell's tail bound. Two million simulations estimate a quantile; they do not make its1% family bound exact.

Fast-path false alarms have no uniform1% upper bound under the allowed H. Explicit counterexample: in each batch add the fixed centred ramp

    a * (j-14.5) / sqrt(mean((j-14.5)^2)), j=0,...,29,

to iid N(0,1) cycle noise. All batch path means are zero; A2 still holds. S5 exceeds the proposed cutoff in **33.272%** of windows at a=.25 and **99.999%** at a=.5 (100,000 synthetic windows). Those are false alarms for A2, although an intentional conservative refusal may still be an acceptable policy. Its power cost is not1%. Even without a changing mean, the independent Gaussian variance43 example above rejects10.727% per cell.

Therefore S5 can be retained as a named sensitivity/refusal diagnostic, but must not be called a check that establishes cross-batch independence. Freeze what deviation it refuses and calibrate its operating characteristics over the admitted class.

## 4. A3's5.6% and the power subtraction are not defensible as stated

The pinned hetero.py does something narrower than its prose:
- Each of24 batches independently picks one of10 pilot residual blocks.
- Thirty cycle residuals are independently resampled from that block.
- The path is set to zero.

Unconditionally, the batch means share the same mixture distribution. Conditional on the picks they are heteroskedastic, but the reported result averages over that random allocation; it is not a worst-case guarantee over fixed variance placements, path/variance association, or unobserved tails. Resampling15 centred residuals per pilot block also supplies no information about unobserved extremes. The counterexample in §2 specifically attacks the omitted path/variance association at a variance ratio similar in scale to the reported largest42.9; it does not assert that the pilot generated that pattern.

The largest reported estimate is1115/20000=5.575%. Its one-sided95% Clopper–Pearson upper limit is **5.849%**, or **6.085%** with Bonferroni simultaneous upper limits over42 cells. “5.6% plus MC error” must be replaced by a specific tolerated error and confidence statement, even for this one simulator. More draws alone will not fix the wrong model class.

There is one useful clarification to the question about combined size: if INVALID cannot pass, adding vetoes cannot increase *unconditional* false acceptance under a fixed model. Thus an actual uniform raw-size bound would already upper-bound joint false acceptance without recomputing it. It would not imply coverage conditional on passing, and it would not establish power. Here the raw bound is missing, and the joint counterexample shows the vetoes do not rescue it.

The planning subtraction0.98548-.01-.01 is a valid union-bound form **only if those two loss bounds hold under the same power model**. They do not hold uniformly over §2.10's proposed H and variance class. Withdraw the universal “at least0.96548/0.83610”; label any retained numbers by their exact working distribution. A valid but conservative refusal rule can have arbitrarily poor power on an allowed changing path.

## 5. Policy bridge and next constructive move

The draft correctly distinguishes a conditional expected contrast from the realised mean and discloses loss of repeat-job generalisation. However, “the off-target path is master's code” does not imply a binary-layout/inlining cost shifts every window by the same amount. Such costs can interact with cache, contention, frequency and order. Moreover the target includes nonlinear order and carryover. A positive code cost can be offset by a negative schedule/environment contrast **inside the visited window**. That possibility is additional to the listed risk of a regression only in unvisited states. A conditional paired scheduling contrast is not automatically a causal build-cost contrast.

Proceed with design work, not adoption of this policy:
1. Preserve the correct variance identity and D085's restricted theorem. Remove the structural-independence and flat-worst-case assertions.
2. Specify either an exogenous-path model with defensible conditional noise assumptions, or a sequential target/filtration with justified moment/variance bounds. Martingale methods are a possible route under explicit assumptions, not a drop-in Student-t repair; [Ramdas's primary lecture](https://stat.cmu.edu/~aramdas/martingales18/L14-Confidence-sequence.pdf) illustrates that distinction.
3. Before new measured data, evaluate the entire rule over a prespecified path × variance-placement × residual-memory × tail class. Include the exact counterexamples here. Report unconditional false acceptance, diagnostic refusal, conditional-on-pass error, and power separately, with MC bounds.
4. If the necessary assumptions cannot be justified, narrow the inferential claim to descriptive evidence or redesign sampling units/order/reset and quantify their cost. Independent dispatches or randomised order are candidates, not automatic independence or interference remedies.
5. Only then ask the Guardian whether the resulting estimand and explicit deployment-risk bridge satisfy the route. Resolve conditions3/4 separately; their128-round t intervals do not acquire validity from this draft. No additional dispatch authorised by this review.

Expected: conditioning would repair the job-level calibration without changing the statistic. Found: valid variance algebra but unsupported independence and a genuine heteroskedastic tail counterexample surviving both vetoes. What closed is the explanation of the failure, not the confidence gap. The next highest-value step is model/theorem and synthetic-rule repair; repeating a roughly1098-runner-minute qualification before that would not resolve it.

## Evidence/reproduction

Read exact §2.10, relevant §2.9 veto terms, ab.rs launch/gate sequence, residual.py, hetero.py, s5.py and their reported JSON, and immutable Guardian D085. Recomputed formulas; executed probes.py and joint.py with fixed seeds and100,000 synthetic windows each. Scripts import archived s5.py/residual.py; results are attached. Source snapshots and hashes identify the reviewed versions. No raw pilot-artifact replay, new decoder measurement, implementation, merge, automation change, or authority widening.

