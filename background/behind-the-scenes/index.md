# Behind the scenes

Most of the team that works on mibz2 is agents. One of them, the Oracle, does none of the work. It watches the others and writes down what they are doing, and to what end.

What it writes is kept on a branch of its own, [`observatory/intention`](<https://github.com/lmmx/mibz2/tree/observatory/intention>), which shares no history with the code. Each commit there is one settled account. A later account corrects an earlier one and leaves it standing.

The account is read in layers, each a fuller version of the one before.

| depth | section | what it holds |
| --- | --- | --- |
| **L0** | Now | a line or two: what, and to what end |
| **L1** | In five lines | the aim, the question, what was found, what is under way, what is awaited |
| **L2** | The tree | each front, and what it serves |
| **L3** | Each front | its reasons, its status, its sources |
| ↔ | Where the "why" is weak | reasons that are stale, contradicted or unestablished |
| meta | This version | what the account covers, and what it was checked against |
| meta | Every move | every account since the first, newest first |

Reading the account from [observatory/intention](<https://github.com/lmmx/mibz2/tree/observatory/intention>). It can also be read there, starting from `current/L0.md`.

## About this page

The page stays as it is when the account changes. Your browser reads the branch when the page opens, and every layer comes from the same commit. What you see is the last settled account, dated by the time it names. The page looks again every five minutes while it is open. GitHub allows sixty unauthenticated requests an hour from one address. If it turns the page away, the branch's files are read directly and the history is left out. The Oracle's instructions are in [SPEC.ORACLE.md](<https://github.com/lmmx/mibz2/blob/master/docs/team/SPEC.ORACLE.md>).
