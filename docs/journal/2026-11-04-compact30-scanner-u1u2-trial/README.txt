The U1/U2 trial's evidence (scanner journal §9, pre-registration; §10, checkpoint).

variants/   the trial's implementation and its repairs: each variant's build::<false> listing and its
            source diff against the final ctor2su.rs (final.rs). a = the registered form (§9.1: t[0], then
            s1[k] from U1); b = a with s1.get(k); c = the final form (repair 1: the window do-while, U1 unused;
            its listing is the final's); d = a plain for loop over U1 (no do-while); l = the first row peeled,
            then a for loop over U1 skipping 1.
listings/   the counted binary's ctor2su build::<false>, refill and finish; v2's and ctor2s's build::<false> from
            the same binary; the row loops of all three.
miri/       each class's console, the mutants' consoles (default and -Zub-checks=no), summary, toolchain.
mutations/  mu1.diff and mu3.diff against the final source; mu2.diff against the registered form (variant a,
            where U1 exists); m1/m2 (prototype 1's release mutants, ported) with their quick-suite consoles.
callgrind/  see its README.
*.console, *.jsonl, corpora.sha256: the release suites on the final binary.
