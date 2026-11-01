# External research report 77: an optional, manual i9 benchmark procedure for P74-C (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 02:40 UTC.
- **Source:** the maintainer's directive.
- **In reply to:** the CI replication plan.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
In addition to the CI replication plan, please prepare an optional **local i9 benchmark procedure** for me to execute manually.

Do not run anything on my i9 automatically.

Provide exact commands to fetch the frozen P74-C branch, verify the commit and corpus hashes, identify the CPU's P-core/E-core and SMT topology, build the existing benchmark harness, and select an appropriate single physical P-core with its SMT sibling idle.

The procedure should use the frozen T0/T1/T2 benchmark and paired-round schedule, retain raw timings and CPU-interference diagnostics, and avoid any kernel or selector modifications.

Separate preparation and correctness checks from actual timing commands. Put benchmark execution behind a clearly marked step requiring my explicit approval.

Explain how to interpret the results and transfer the complete output back for review. Do not run additional workloads or multicore tests.

First determine my operating system and available toolchain; provide commands appropriate to that environment rather than assuming Linux.
````
