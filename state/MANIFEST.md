# Migration manifest: researcher

- **Source:** `review/compact30-adversarial-scanner-146` at `1283b969e9e295e268257cd1b6be3ad7da29e6dd` (its exact final head, every inbox comment finished).
- **Destination:** `team-state/researcher`, a continuation of that commit.
- **Moves:** `docs/journal/compact30-adversarial` → `state/reports`; `state/reports/scanner-146/review-cursor.json` → `state/cursor.json`
- **Removed:** every other tracked file; they remain on `review/compact30-adversarial-scanner-146` at `1283b969e9e295e268257cd1b6be3ad7da29e6dd`.
- **Changed in content:** only `state/cursor.json`, which gains `migrated_from`.

## Files (path, blob at migration; unchanged from the source except the cursor)

- `state/cursor.json` `af82cf9c33e024959ba33db6806dacaebc115fbd`
- `state/reports/blog-agent-team/CHANGE_NOTE.md` `39a9850bf8fbe0715f89720c781d377c3665430f`
- `state/reports/blog-agent-team/reader-tests.md` `71b77607be3a8c649829c026193f3523db56aadf`
- `state/reports/scanner-146/README.md` `f7aab5ba50c3463a5b10d60c610b0d26d9d3ae30`
- `state/reports/scanner-146/a6-reconciliation-6640266327.md` `40e281b2716fb83c3dbef0e86ad1dc8489f3fff9`
- `state/reports/scanner-146/accounting/ACCOUNTING.md` `b4adbcec6b8bd5385591360d9544371b212458fe`
- `state/reports/scanner-146/accounting/reconstruct.py` `a67ac9669f200b50cad08729a3a39d7aefc3c94b`
- `state/reports/scanner-146/accounting/reconstruction.json` `b8bc1a486254186275040b8d2ea47a2c8a9e07ef`
- `state/reports/scanner-146/assembly1-review-6639998280/audit.json` `64c43c054b8a8e562190942c0786cbb00b17ec31`
- `state/reports/scanner-146/assembly1-review-6639998280/report.md` `7f79cb88644f57e041435dbc35d1342badfb82c5`
- `state/reports/scanner-146/bridge-review-6640153383/audit.json` `45bc526d943203d1bb1453186567b01b81960687`
- `state/reports/scanner-146/bridge-review-6640153383/report.md` `a3dc2c89bc04712d02a1bc387b4e389b79337792`
- `state/reports/scanner-146/c6-review-6634998636/b2-refill.asm` `1f3b4b5a91e889c344a18ee914b3f2d156febbb6`
- `state/reports/scanner-146/c6-review-6634998636/descriptor-refill.asm` `60ad7562e7dd563f018a83daf84d0d5b1af3da97`
- `state/reports/scanner-146/c6-review-6634998636/figures.json` `319c9dc0b166d8368aae6cefcc12038245ba4417`
- `state/reports/scanner-146/c6-review-6634998636/replay.py` `b3d92412b68a27e5576e6afbdc88e613d5c29465`
- `state/reports/scanner-146/c6-review-6634998636/report.md` `0f3a34d095a8243aa08c811ef796f20bb627578d`
- `state/reports/scanner-146/c7-c10-review-6639756189/audit.json` `3a0105a6c9b83c38227e3a2b3107be5f262da0eb`
- `state/reports/scanner-146/c7-c10-review-6639756189/report.md` `4aa747aae693c03b64a5360c057fa9afa9dc0f38`
- `state/reports/scanner-146/causal-scope-6644912625/audit.json` `faddc358d11d4697777a8aeac96559ff1a485302`
- `state/reports/scanner-146/causal-scope-6644912625/report.md` `9412e2cfcf2e6adc224a2608f603422de5e6e38e`
- `state/reports/scanner-146/conditional-refit-6644115799/audit.json` `0702da9e93fd13d9369fb17a926ba36f906b405f`
- `state/reports/scanner-146/conditional-refit-6644115799/report.md` `0449a233297b75b6cdf39b7fa44d1610520ec246`
- `state/reports/scanner-146/coordinator-goal-6637485148/goal.txt` `7d8038e039a3f508b03ca77cbc7ddba70a6ba754`
- `state/reports/scanner-146/coordinator-goal-6637485148/reader-tests.md` `a4d65ce4a777693479dde3fb6a6f885401e93015`
- `state/reports/scanner-146/coordinator-goal-6637485148/report.md` `337e9a9a0342102d08d3314d1c34b409ecb1aa9a`
- `state/reports/scanner-146/copy-predictions-6643992749/audit.json` `66def40ce5339eebb31ed2b8bc640ffdee4dd810`
- `state/reports/scanner-146/copy-predictions-6643992749/bench-snapshots.json` `4a0093a652b8cd3a17f02dcbfaf51a92adcd4b5d`
- `state/reports/scanner-146/copy-predictions-6643992749/report.md` `8a202b3757f7a76f5c0279f69dfde5d8a5f421b4`
- `state/reports/scanner-146/d011-reconciliation-6635620697.md` `9e94000d30f319fb4fb8111e5c6001615ae198ea`
- `state/reports/scanner-146/d013-reconciliation-6640069360.md` `7a6ceed4098f239f2953682e02bab7ce72c81992`
- `state/reports/scanner-146/d016-reconciliation-6641768409.md` `79779f51bfeff57a2d3144620e2ae9bd0e2d522d`
- `state/reports/scanner-146/d018-reconciliation-6643850845.md` `aeb87ed037c3ff4b412c9d35e94f812c8b8d2c78`
- `state/reports/scanner-146/d019-model-provenance-6643927959.md` `9ba1a801e80944c723213c2b25d7c885ae80b15e`
- `state/reports/scanner-146/d020-reconciliation-6644051935.md` `efb7533a8bb070eacde4901be867d7c83818901b`
- `state/reports/scanner-146/deferred-handover-6643481606/report.md` `db3e167b98bd851dee9b48bf94f5e03188681465`
- `state/reports/scanner-146/deferred-handover-6643481606/schedule-check.js` `ef392a7c3c3baf84aecdbdb1092cd029a42c90b6`
- `state/reports/scanner-146/f01-review-6635419092/audit.json` `c6a638aec24867127c3eb19f912d264896ec6882`
- `state/reports/scanner-146/f01-review-6635419092/audit.py` `8035ecef94aab6fa77963b641d5d968dc417b48c`
- `state/reports/scanner-146/f01-review-6635419092/inputs.json` `ccb4e29fe8e9d7bf49a81329b1dc0382bb925205`
- `state/reports/scanner-146/f01-review-6635419092/report.md` `5267dac6d44eec7c25a27188a75921e287f46915`
- `state/reports/scanner-146/fresh-check/REVIEW.md` `c550789e877266bdfb689b84cfb8a98965ba1079`
- `state/reports/scanner-146/fresh-check/check_totals.json` `f8bd27b18953ab0b73b06f7f2accc3d0fb9d2300`
- `state/reports/scanner-146/fresh-check/check_totals.py` `7608feac79177fc4bdab26cd008e52cbe4937db2`
- `state/reports/scanner-146/input-manifest.json` `9f7be2f34bc494c20393feb43eb8cf98da75bac6`
- `state/reports/scanner-146/messages.json` `6f2487e2a8958ee11cf57f77597f899eccbc08c4`
- `state/reports/scanner-146/mode-receipt-6639428570.md` `d46e9402f398db9beccf89dddc14fc1ebdf764b7`
- `state/reports/scanner-146/oracle-6683787130/counterexample.json` `82104287044b9ae0e8212fda63630f49db2aecf8`
- `state/reports/scanner-146/oracle-6683787130/report.md` `1e94d1065ac1331b44fe9024d14ab198f6208c21`
- `state/reports/scanner-146/oracle-precis-6684137403/report.md` `cc53d84cd241070652a7c96d3034d572bfce0633`
- `state/reports/scanner-146/p87-registration-6645922140.md` `960766f21146d706219cf16d44ea265ef650a58a`
- `state/reports/scanner-146/p87-results-6646250444/interval-audit.json` `30e96afbfb090100af15372b7d8a5b68fab681a3`
- `state/reports/scanner-146/p87-results-6646250444/report.md` `f3d572cf8159a5cacbcf43639a6fac9a732991ee`
- `state/reports/scanner-146/phase/model.py` `a560f07c52f3366eaf4203b89d244c2c6aa63adb`
- `state/reports/scanner-146/phase/proposal.md` `dd0d567246f7f7cb17203a3239c8428a63487aa5`
- `state/reports/scanner-146/phase/results.json` `349369acb274a606a76c92b5699a33bfb80b22d8`
- `state/reports/scanner-146/pivot-review-6643744616/audit.json` `216a5a36b31c663cd132acdae45a69d20e7fee68`
- `state/reports/scanner-146/pivot-review-6643744616/report.md` `d3821a7869cd8e19b9e0aabed71502814efb1aba`
- `state/reports/scanner-146/policy-receipt-6637300905.md` `f106feed975b819859c0c99d5d579d15f043f9bb`
- `state/reports/scanner-146/published-model-6643809123/audit.json` `bd8cc3483b2833dd4a8290c478b16d093d5e5e64`
- `state/reports/scanner-146/published-model-6643809123/report.md` `20684a3a31aee8b93a5ed743304a0d822b7282a0`
- `state/reports/scanner-146/replay_all.py` `898c3d1bc5be12a42877c978ec9ac32e9a30e50e`
- `state/reports/scanner-146/report.md` `1e48135e08ae3244917a1d323659ca8f12b9d59d`
- `state/reports/scanner-146/s62-ack-6633657739.md` `95108ba0f591479f0701840e15585ab0da8ec016`
- `state/reports/scanner-146/s62-completion-6634296412.json` `62d2f6f0a3eb901f93a039b9bd209723d00e080e`
- `state/reports/scanner-146/s62-intake-6633302554.md` `a1a749db02ff63f13aea009a6db1503857cffd81`
- `state/reports/scanner-146/s62-launch-6633793271.json` `f62ac438ca28a7961bd65e10a875455279f82e29`
- `state/reports/scanner-146/safety/SAFETY.md` `7caca2b65d49a87d4cc0088cfbba5306557bd388`
- `state/reports/scanner-146/safety/replay_safety.py` `de85f44ff9f9286e57eb9768df12fb6d015dcc69`
- `state/reports/scanner-146/safety/results.json` `745f449cdf22833dd8be9e39e383475c9d97fad6`
- `state/reports/scanner-146/safety/source-first-note.txt` `03f70c0ba682aead1ea64ac6c9234e846cdd09d8`
- `state/reports/scanner-146/team-neutral-6684938042/report.md` `631de22dee12901f362f8f0595a31b3addcca152`
- `state/reports/scanner-146/team-size-6681848572/report.md` `9ecd24dfe0de7b78b8ad14fd088031b793331a64`
- `state/reports/scanner-146/team-size-amendment-6682105521/report.md` `23571bc4ed9443e488e21e672af6aa38450296c4`
- `state/reports/scanner-146/three-tables-6682938532/paired-reconstruction.json` `33e430d44b031ea750708c695c837df188002cde`
- `state/reports/scanner-146/three-tables-6682938532/report.md` `55471878aae8abf7aed20025f4881536d77bd124`
- `state/reports/scanner-146/topology-review-6640324039/audit.json` `29d03188afa268fadd09e116975a7cd33aa2ef1a`
- `state/reports/scanner-146/topology-review-6640324039/report.md` `5b1f76fb04ec6db0e460c29287f95d8ab74160ec`
- `state/reports/scanner-146/validation.json` `97196d54792598d0959469070784f1316c5409f4`

## Queue state carried over (from the cursor, verbatim)

```json
{
 "processed_comment_ids": [
  6633302554,
  6633657739,
  6633675410,
  6633777567,
  6633793271,
  6633918984,
  6634250201,
  6634296412,
  6634998636,
  6635419092,
  6635620697,
  6637300905,
  6637485148,
  6639428570,
  6639756189,
  6639998280,
  6640069360,
  6640153383,
  6640266327,
  6640324039,
  6641768409,
  6643481606,
  6643744616,
  6643809123,
  6643850845,
  6643927959,
  6643992749,
  6644051935,
  6644115799,
  6644912625,
  6645922140,
  6646250444,
  6681848572,
  6682105521,
  6682938532,
  6683787130,
  6684137403,
  6684938042
 ],
 "comment_reviews": [
  {
   "trigger_comment_id": 6633302554,
   "status": null
  },
  {
   "trigger_comment_id": 6633657739,
   "status": null
  },
  {
   "trigger_comment_id": null,
   "status": null
  },
  {
   "trigger_comment_id": null,
   "status": null
  },
  {
   "trigger_comment_id": 6634250201,
   "status": null
  },
  {
   "trigger_comment_id": 6634296412,
   "status": null
  },
  {
   "trigger_comment_id": 6634998636,
   "status": null
  },
  {
   "trigger_comment_id": 6635419092,
   "status": null
  },
  {
   "trigger_comment_id": 6635620697,
   "status": null
  },
  {
   "trigger_comment_id": 6637300905,
   "status": null
  },
  {
   "trigger_comment_id": 6637485148,
   "status": null
  },
  {
   "trigger_comment_id": 6639428570,
   "status": null
  },
  {
   "trigger_comment_id": 6639756189,
   "status": null
  },
  {
   "trigger_comment_id": 6639998280,
   "status": null
  },
  {
   "trigger_comment_id": 6640069360,
   "status": null
  },
  {
   "trigger_comment_id": 6640153383,
   "status": null
  },
  {
   "trigger_comment_id": 6640266327,
   "status": null
  },
  {
   "trigger_comment_id": 6640324039,
   "status": null
  },
  {
   "trigger_comment_id": 6641768409,
   "status": null
  },
  {
   "trigger_comment_id": 6643481606,
   "status": null
  },
  {
   "trigger_comment_id": 6643744616,
   "status": null
  },
  {
   "trigger_comment_id": 6643809123,
   "status": null
  },
  {
   "trigger_comment_id": 6643850845,
   "status": null
  },
  {
   "trigger_comment_id": 6643927959,
   "status": null
  },
  {
   "trigger_comment_id": 6643992749,
   "status": null
  },
  {
   "trigger_comment_id": 6644051935,
   "status": null
  },
  {
   "trigger_comment_id": 6644115799,
   "status": null
  },
  {
   "trigger_comment_id": 6644912625,
   "status": null
  },
  {
   "trigger_comment_id": 6645922140,
   "status": null
  },
  {
   "trigger_comment_id": 6646250444,
   "status": "completed"
  },
  {
   "trigger_comment_id": 6681848572,
   "status": "finished"
  },
  {
   "trigger_comment_id": 6682105521,
   "status": "finished"
  },
  {
   "trigger_comment_id": 6682938532,
   "status": "finished"
  },
  {
   "trigger_comment_id": 6683787130,
   "status": "finished"
  },
  {
   "trigger_comment_id": 6684137403,
   "status": "finished"
  },
  {
   "trigger_comment_id": 6684938042,
   "status": "finished"
  }
 ]
}
```
