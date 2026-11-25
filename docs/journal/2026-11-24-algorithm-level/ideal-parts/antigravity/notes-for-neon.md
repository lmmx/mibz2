# Notes for Antigravity (Neon) to consider: optional, not assigned

Leads Louis asked to put in front of you. Consider them where they bear on your current pass, and say in a line whether they do. None of them is a task.

## 1. Helmer: Whitney stratifications, fast (25 November 2026)

Vidit Nanda ported Martin Helmer's Macaulay2 package WhitneyStratifications to C++ ("Helmer", people.maths.ox.ac.uk/nanda/helmer/; `pip install helmer-strat`; GPL-3, built on msolve and FLINT).
- **What it computes:** certified Whitney stratifications of complex and real algebraic varieties, and of polynomial maps, from equations over ℚ.
- **The papers:**
  - Helmer and Nanda, *Conormal Spaces and Whitney Stratifications*, FoCM 2022, arXiv:2106.14555;
  - Helmer, Leykin and Nanda, *Effective Whitney Stratification of Real Algebraic Varieties*, Math. Comp. 2025, arXiv:2307.05427;
  - Helmer and Mohr, arXiv:2406.17122.

**The Coordinator's reading:**
- **For the decoder's arithmetic:** probably not useful, since the decoding steps are discrete, over finite alphabets.
- **For the performance model as measured:** not either, since it is a max-plus, piecewise-linear model.
- **Where it could apply:** a map stratification of a *smooth or rational* performance model over its parameters would be a certified regime map, which ties to the design-space question (Khammash; Ma et al.). That applies only if such a model is written. Examples: a rational MVA-style surrogate, MONODROMY's soft-max, or a polynomial fluid model.

**The question for you:** does any reformulation you propose give a polynomial system where a stratification of the parameter space would be informative?
