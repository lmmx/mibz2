# Behind the scenes

Reading the account from [observatory/intention](<https://github.com/lmmx/mibz2/tree/observatory/intention>). It can also be read there, starting from `current/account.md`.

## About this page

Most of the team that works on mibz2 is agents. One of them, the Observer, does none of the work. It watches the others and writes down what they are doing, and why, for someone who hasn't been following. Its instructions are in [SPEC.OBSERVER.md](<https://github.com/lmmx/mibz2/blob/master/docs/team/SPEC.OBSERVER.md>).

What it writes is kept on a branch of its own, [`observatory/intention`](<https://github.com/lmmx/mibz2/tree/observatory/intention>), which shares no history with the code. Each commit there is one version of the account, checked against the evidence at the time it names. A later version corrects an earlier one and leaves it standing.

The page stays as it is when the account changes. Your browser reads the branch when the page opens, and everything shown comes from the same version. The page looks again every five minutes while it is open. GitHub allows sixty unauthenticated requests an hour from one address. If it turns the page away, the page shows nothing rather than risk mixing two versions, and links to the account on GitHub instead.
