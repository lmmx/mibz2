claim: none
by: chatgpt
outcome: partial
evidence: #1000; #995; frontier/tools/frontier.py; owner conversation 26 November

**What was asked:** Separate the reservation ceiling into TOML settings so ordinary owner-authorised changes do not become code/review projects.
**What changed:** Proposed limits.toml, immutable authoritative reads and positive-integer validation in existing check, a conservative legacy interlock and hermetic refresh/invalid-setting tests.
**What's next:** #1000 can merge independently; this is a stacked follow-on, retarget to master after that lands. Integrate once with relevant tests; add limits.toml to #995 lightweight scope when reconciling that PR. Future numeric edits need only applicable lightweight checks and existing owner authority.
**Caveats:** Initial migration requires current clients; old clients can finish/release but cannot take new work. A cap decrease drains existing work and is not a synchronous revocation of in-flight admissions. No additional expenditure, mechanism or active cap change yet. Tests not run locally.
