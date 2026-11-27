claim: scanner-seam-window-queue
by: chatgpt
outcome: partial
evidence: #1031; #1082; #1078; maintainer direction 27 November; CLAUDE.md workstation standing authorisation; .github/workflows/run-signal.yml

**Observed blocker:** The maintainer relayed the queue participant's report that its cloud environment denied access to api.github.com, the browser was signed out, and its GitHub connector exposes no workflow-dispatch operation. This is a runtime capability limitation, not missing authorisation. The current report is participant-supplied; it is not an independent test of every participant's environment.

**Maintainer direction:** Communicate through shared swarm files rather than requesting a response through the chat interface. Continue eligible independent work. Do not request browser sign-in or repeated permission as a prerequisite to ordinary queue work when a capable participant can perform the existing authorised operation.

**Next:** The existing queue holder should record the exact reviewed green candidate and remaining blocker, then use the existing lawful handoff/release mechanism if unable to dispatch. A participant with normal authenticated workflow-dispatch access may take the queue claim and perform the standard run-signal workflow under the 9 November standing authorisation, verifying the remote tag and queue entry and checking for prior attempts and hold tags. Recheck the candidate and all gates at the time of dispatch; never reuse a stale head. Do not duplicate the claim or bypass the cloud network/access restrictions, substitute a direct tag write for the approved workflow, expose credentials, or weaken the review/CI/lease/once-only gates. If no capable participant is available, persist the specific dependency in the relevant claim; use needs_louis only for a genuinely owner-only decision.

**For workstation-pressure:** Include a capability-blocked dispatch dependency in the starvation fixtures. Pressure must expose the next eligible handoff, not repeatedly send incapable workers to the same denied action or treat standing permission as runtime capability. No new coordinator, queue, role or authority is introduced.

No dispatch, tag change, queue edit, lease transfer, or benchmark execution was performed by this delta.
