# 4. Alternative decoder architecture

## Service-core

- **Doing:** follow up a promising, unjudged ten-thread cell with a matched, registered comparison, in order to serve 0 by improving wall and CPU cost together.
- **Found:** first service-core attempt INVALID on its noise control; an unchanged registered rerun was VALID. One service thread saturated and regressed. Two service threads at twelve traded a 1.058 speed gain against the default for more CPU. The reported ten-thread cell was 1.045 (1.023–1.068) with about 12% less CPU, but was not judged by that registration ([result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6720510159)).
- **Status:** [#258](https://github.com/lmmx/mibz2/pull/258) head 3e4477d035e96310eb4ac3d4014e01e13ffaa33c, frozen at 21cf20a49e3a90817e718a14abcc254679de6b2a. Coordinator reports dry run and CI green; Researcher review staged [as QUEUED](https://github.com/lmmx/mibz2/pull/145#issuecomment-6729027210). Stand-in request withdrawn; no ready tag or queue entry established.
- **Interpretation:** the follow-up distinguishes service threads from choosing two rather than three teams. The earlier signal is not a production verdict.

## Run-fed table construction

- **Doing:** opt-in diagnostic of move-to-front decoding into run records, stable sort and sequential table fill against the current decoded-symbol path, in order to serve 0 through a possibly cheaper complete stage ([direction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6728891399)).
- **Premise:** new sequential table writes might outweigh sort and run-emission costs; no such gain measured yet. Existing test-only code from #266 is correctness research, not an SIMD-equivalent performance arm.
- **Status:** Coordinator reports implementation under way. Fair matched inputs, buffers, cache conditions and phase costs; word-identical table and full decoding correctness; and a C1 information objective or threshold must precede substantive timing. A workstation recipe is conditional on a favourable diagnostic, independent review, freeze and Louis's own execution. #258's near-ready work is separate.
