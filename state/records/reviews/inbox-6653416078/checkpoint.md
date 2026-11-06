# Compact parity after #183: checkpoint

Inbox 6653416078; baseline 3c4ed7939d107a7b7b2633ca913f2e640023f15a; spec 0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0.

- #183 fails its registered four-thread benefit rule. Its two PR estimates are -4.2% (CI -10.0,+1.8) and +1.5% (-3.9,+7.2). Fusion-alone reports -1.5% (-2.4,-0.6) at four threads but flags text 2t. This does not establish an exactly zero effect or prove all sequential traffic irrelevant.
- Compact30 does not intrinsically remove random references: ordinary header and field share one line; RAW adds a dependent payload. Smaller footprint may reduce repeated fills. The 1/16 to 1/30 figures are compulsory distinct-line densities, not measured miss probabilities.
- Existing #155 port stays rejected. Old four-physical-core traversal wins exclude the direct builder and do not transfer to current two-core SMT staging.
- Proposed next comparison: current staged compact30 with direct builder, matched against pinned production. Include an equal-byte layout-only control; specify it concretely and keep its inference separate from compact-vs-plain reconstruction costs.
- Split-condvar result supports closure of that candidate locally. Wasted-wake CPU fractions are not strict wall-time bounds on SMT; queue waits do not prove the walk is the sole bottleneck.

Research in progress; no implementation, compilation or timing performed.
