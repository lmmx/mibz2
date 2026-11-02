# External research report 181: the guardian approves the capture with the pinned toolchain, and on Rust 1.98 (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with the maintainer's question about upgrading
  and the guardian's answer.
- **Source:** the guardian.
- **In reply to:** the toolchain repair at branch head `0d85566` (PR #134), source pin
  `9267433`.

**Evidence cutoff:** 2 November 2026. A reading of the recipe at `0d85566`, compared with the
previous checkpoint.

**This authorises:** one capture, `just bench-c30-traversal`, on the workstation at branch head
`0d85566`. Rust 1.97.0 is installed beside the default for that one run.

**On the outcome:**
- Upload the archive whether the run passes or fails.
- No automatic retries and no threshold changes.
- The earlier G3 failure's archive is kept.

**On the toolchain:** 1.97.0 is used for this frozen measurement only. Development stays on
the newer default, and 1.98's code generation can be evaluated separately later.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
Approved. Claude has fixed the toolchain-reproducibility problem, and I found no reason to reopen the compact30 review.
I inspected the updated recipe in [PR #134](https://github.com/lmmx/mibz2/pull/134?utm_source=chatgpt.com) at `0d85566` and compared it with the previous checkpoint.
The change is appropriately narrow: it explicitly selects Rust 1.97.0, verifies the compiler commit and LLVM 22.1.6, clears inherited Rust compiler flags, and retains the strict machine-code identity check. The experiment itself remains pinned at `9267433`.
Claude has also tested the important cases: a newer default compiler, a missing toolchain and an incorrectly identified toolchain. Those are exactly the circumstances that caused—or could have concealed—the earlier failure.
You can run the capture now
From your `~/dev/mibz2` directory:

```
rustup toolchain install 1.97.0 --profile minimal

git switch research/compact30-walker
git pull --ff-only
git rev-parse --short HEAD

just bench-c30-traversal
```

The branch should report `0d85566` or its full equivalent. The benchmark recipe will use the reviewed compiler without changing your default Rust installation.
Your earlier failed archive remains useful evidence of G3 doing its job. Do not delete it.
When the new run finishes, upload the `target/c30-traversal-<UTC>.tar.gz` archive. If it fails again, preserve that archive too—no automatic retries or threshold changes.
And the blog
It's good that Claude completed the blog independently while we resolved the experiment's environment issue. Congratulations on getting the MLP-versus-ILP piece published.
The research branch and benchmark are separate from that publication, so there's no need to hold either one up now.
The next meaningful decision is finally the measured four-core result.
`````

## The maintainer's question and the guardian's answer, as relayed

`````text
> why arent we upgrading our toolchain to 1.98... sigh... ok fine

We absolutely should upgrade to Rust 1.98 for ongoing development. I agree with you.
The reason I'm insisting on 1.97 for this particular experiment is that we have already frozen and independently reviewed the candidate's machine code under that compiler.
Rust 1.98 generated different assembly on your i9. If we switch compilers now, we're no longer measuring precisely the implementation we reviewed. We'd need to inspect the new assembly, establish its instruction and dependency structure, and update the experimental pin.
That isn't inherently bad. In fact, Rust 1.98 might generate better code. But changing compilers just before the first performance measurement adds another variable.
Here's the distinction I should have made clearer:

* Development: Keep Rust 1.98 as your default. Future experiments should use it.
* This one frozen benchmark: Use Rust 1.97 to preserve the reviewed machine-code baseline.
* After this benchmark: We can evaluate Rust 1.98's code generation separately, particularly if compact30 earns further development.

We're not downgrading your project or your workstation. We're installing 1.97 alongside 1.98 for one reproducible experiment.
And you're right to be impatient with the process. We've spent a disproportionate amount of effort on experimental validation. That was worthwhile for preventing misleading results, but we shouldn't turn every compiler update into a research programme.
Run the frozen comparison on 1.97, keep developing on 1.98, and move forward.
`````
