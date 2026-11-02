# External research report 142: on the maintainer's archive request: per-run results under target/, one tarball with a manifest, as the default for i9 diagnostics (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** unattributed in the relay; it reads as the guardian's reply to the maintainer's request that results be tarballed under `target/`.
- **In reply to:** the maintainer's request with report 141.

**Evidence cutoff:** 2 November 2026. A convention for future runs; no change to the authorization.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
Good addition. That will make reviewing the results here much easier.
I'd have Claude adopt it as the default for future i9 diagnostics:

* Write generated logs, JSON, counter outputs and summaries under `target/<experiment>/`.
* Create one `.tar.gz` archive under `target/` containing the relevant evidence.
* Include a manifest with the source commit, corpus hashes, executed command and file checksums.
* Keep experimental outputs out of the repository root. Preserve original raw files unchanged.

One small distinction: the archive should contain only the artifacts from that particular run, not the entire `target/` directory or unrelated previous results.
For the next walker probe, ideally you'd receive something like:

```
target/walker-probe-20261102T000606Z.tar.gz
```

Then you can upload that single file here for independent analysis.
I particularly like that this reduces the friction of getting complete evidence into the conversation. It also makes it less likely that a relevant raw `perf` record gets omitted when copying individual files.
No other change to the authorization: Claude should prepare and validate the repaired probe, then stop before asking you to run it.
`````
