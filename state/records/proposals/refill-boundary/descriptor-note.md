# Descriptor-array qualification triage

This is an accounting aid for the already registered descriptor-array prototype, not a new
authorization and not its eventual gate verdict. Its whole-constructor checkpoint ledger remains
the authority. All numbers below are CONDITIONAL until the generated paths and frequencies are
recounted.

The frozen B2 scalar common path contains 12 instructions in all four copies: build listing
at `cd14fa9`, ranges `0x33a00–0x33a29`, `0x34350–0x3437a`, `0x343e0–0x34406`,
`0x344d0–0x344f9`. The two descriptor-address LEAs per row are at `0x33a05/0x33a09`,
`0x34355/0x34359`, `0x343e4/0x343e8`, `0x344d5/0x344d9`. Source is
`examples/c30walk/ctor2.rs` at `8beffe2`, blob
`168c2bac9ad0d8da4d9f82788f009d32f419608e`, descriptor 233–240 and append 538–551.
The Ultra audit `report.txt` blob `0ff2f0e594d9459243de9313e7941267f8476229`, lines 343–368,
provides the four-loop check and archived scalar count `S=38,294,883`.

Let `c` be the new scalar common-path instruction count, when it is the same in all copies.
Let `H` be **signed** net added work everywhere else in the constructor, including callers,
refill, finish, rank snapshots, run paths, initialization, fallback, errors and spills. Positive
`H` is added cost; negative `H` is a separately demonstrated saving. Then

`net saving = (12-c) * 38,294,883 - H`.

If the four copies differ, replace this scalar term with the sum over the copies using their
own observed frequencies; do not multiply the best loop's count by every scalar row.

| Compiled scalar shape | Gross row saving | Condition to reach 40,000,000 net | With H=5,000,000 |
|---|---:|---:|---:|
| 10 instructions | 76,589,766 | H <= 36,589,766 | 71,589,766 |
| 11 instructions | 38,294,883 | H <= -1,705,117 | 33,294,883 |

Thus an 11-instruction path does not qualify merely because the remaining code is unchanged;
it still needs at least 1,705,117 demonstrated non-row savings. Conversely it must not be
rejected on the diagnostic target alone if its full ledger demonstrates those savings. The
five-million allowance in the older proposal is an acceptance cap, not a measured cost or a
fact established by equal descriptor byte size.

Keep these recurring cost multipliers visible when checking `H`: one extra instruction per
snapshot-character costs 935,424; per finished portion 1,795,482; per run 610,521; per
block-character 111,104. The audit bounds nonempty scalar refills below by 1,122,673, but
initial and post-run empty refills must also be counted for costs incurred on **every** refill
call. A loose primary-corpus ceiling is `P + 256*B + R = 2,517,107`: each nonempty call finishes
one unique portion, each character starts empty once, and each run can leave its descriptor
empty at most once on returning. This bound allows all such events, even when later runs
consume the empty state instead of a scalar refill. Exact counted initial, post-run empty,
nonempty and refused call classes are preferable. Do not silently apply the nonempty upper
bound to all calls. Refused malformed-input paths need their own documented execution counts
in the relevant suites, and no invented primary frequency.

Splitting a memory pointer increment into a register increment plus store can eliminate one
explicit pointer read but adds one instruction per scalar row, 38,294,883 instructions on this
corpus. It must be reported as a memory-operation trade and charged to this formula. It does
not eliminate the repeated-character pointer store-to-load recurrence.

The refill-boundary proposal in this directory must not be smuggled into a descriptor-array
attempt: it changes the design and new unsafe pointer obligations require approval. If later
authorized, it overlaps the descriptor prototype's helper ledger. Recount the combined B2
delta; do not add its target to the older 71,589,766 cap without translating helper changes.
