# 1. Introduction

**Coverage:** the complete author-issued Chapter 1 sample, printed pp. 1-23,
sections 1.1-1.5, including the notes. All text was read; relevant diagrams and
formula layout were checked. This is provisional Chapter 1 coverage only.
Chapters 2-9 remain pending access to the complete book.

**Source:** R. Goebel, R. G. Sanfelice and A. R. Teel, *Hybrid Dynamical Systems:
Modeling, Stability, and Robustness*, Princeton University Press, 2012.
[Author-issued chapter PDF](https://hybrid.soe.ucsc.edu/sites/default/files/preprints/65.pdf)
([author's bibliographic record](https://hybrid.soe.ucsc.edu/biblio/2012/hybrid-dynamical-systems-modeling-stability-and-robustness)).

## Précis

Chapter 1 introduces four modelling objects: a flow set and map, and a jump set and map.
Differential and difference inclusions describe continuous evolution and instantaneous
updates in a finite-dimensional state; equations are a special case. Maps must be
defined and nonempty where enabled. Formal solutions are deferred to Chapter 2.

Bouncing impacts, synchronized fireflies and a thyristor illustrate the framework.
Control examples add held inputs, quantization scales, reset rules, timers and a
supervisor selecting local or global feedback. Hysteresis and enforced elapsed time
address repeated switching. Combining controllers also requires compatible regions and
trapping behavior; individually stabilizing controllers alone are insufficient.

The chapter translates explicit modes, hybrid automata, impulsive equations and
switching systems into common data. Overlapping guards naturally produce set-valued
resets. It cautions that a representation can be fragile under perturbations: an added
countdown timer improves the impulsive-system construction. Unrestricted switching may
be better studied through differential inclusions, while clock-augmented hybrid models
help impose timing restrictions. Initial examples are modelling proposals, not general
stability guarantees. The notes locate the framework and examples in the prior
literature.

## Errata applied

The [author's errata](https://hybrid.soe.ucsc.edu/publications/errata-hsbook)
correct p. 7's first two jump-map entries to \((z_1,z_2)\), and p. 9's feedback
argument to \(\kappa(z)\).

## Separate decoder interpretation

[Modelling the decoder](../applications/01-modeling-the-decoder.md) contains our
own proposed model, source-code mapping and candidate changes. Its engineering
claims are not conclusions of this chapter.

