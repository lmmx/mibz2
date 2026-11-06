# Migration manifest: optimizer

- **Source:** `review/compact30-instruction-ledger` at `e6eea5889432aef28f4d9e450cee87dd5bf7bacf` (its exact final head, every inbox comment finished).
- **Destination:** `team-state/optimizer`, a continuation of that commit.
- **Moves:** `docs/journal/compact30-research` → `state/records`; `state/records/review-cursor.json` → `state/cursor.json`; `state/records/instruction-ledger.md` → `state/ledger.md`
- **Removed:** every other tracked file; they remain on `review/compact30-instruction-ledger` at `e6eea5889432aef28f4d9e450cee87dd5bf7bacf`.
- **Changed in content:** only `state/cursor.json`, which gains `migrated_from`.

## Files (path, blob at migration; unchanged from the source except the cursor)

- `state/cursor.json` `0d3550ea9fb9aabdc766ebbb93d5b5ec6404785e`
- `state/ledger.md` `f4485c3f45e4e99399711bc6601279cfc0697d9f`
- `state/records/INDEX.md` `82341eb835284a9a31499981b62a4ae8b297e450`
- `state/records/baseline-replay.json` `0769071967259013ea3c31291881c736f60b226e`
- `state/records/proposals/ranked-next-cuts-6639496192/report.md` `0b3cf15f01981c2bcdf5421d98b29d426ba4451b`
- `state/records/proposals/refill-boundary/descriptor-note.md` `af387a65c151081ba045dc59608b9487319190e0`
- `state/records/proposals/refill-boundary/model.py` `c1a7890ec76b4de523aabcba91c0826da6539f57`
- `state/records/proposals/refill-boundary/proposal.md` `67da104d8a05efd82214e6c91046b5ff3282b801`
- `state/records/proposals/refill-boundary/results.json` `dfef05f978add53e12b14d7b4592f7afc3025ed7`
- `state/records/recount_baseline.py` `3487e2817ab52943070b09faffbfb2d2038ab0ff`
- `state/records/replay_structure.py` `4cb3195099abdf7f92aa2e5babc58fb93e1fd046`
- `state/records/research-brief.md` `7cfe4d08c7f2841b1a16721b7ce2ba4d22bdaee2`
- `state/records/reviews/checkpoint-6633116288/audit-input.json` `40f7a95ee91f64678f031734f2ebf3eefad8dd11`
- `state/records/reviews/checkpoint-6633116288/evidence-manifest.json` `a6385a9a3e9cf66d80b0d3453571f0c42b18543c`
- `state/records/reviews/checkpoint-6633116288/replay-results.json` `5763cd473207bc89bbc945b27ede4650329853aa`
- `state/records/reviews/checkpoint-6633116288/replay-review.py` `c297986dadfe4138719876755ed5733cd5cd1bf6`
- `state/records/reviews/checkpoint-6633116288/report.md` `3da200d66888b95ae0748dba9f73d9549848e793`
- `state/records/reviews/checkpoint-6634996672/overlap-map.json` `6b54c1f3f7c8e963972728c030fdbfd8f170193f`
- `state/records/reviews/checkpoint-6634996672/report.md` `7bda60cf9bf710d250b79d39b2e0f7304bb240cd`
- `state/records/reviews/checkpoint-6637121066/report.md` `3aef58d80d748723cc5cd60692ca51f357da6ef1`
- `state/records/reviews/checkpoint-6640153383/bridge-audit.json` `55c42b76b05512361415b19e7575c87d8abc8cd7`
- `state/records/reviews/checkpoint-6640153383/report.md` `3bd84e7daea147d5fe61a6c43411d49247205855`
- `state/records/reviews/inbox-6645643541/report.md` `1065dbd96a3e96824ca5180959e3bfb21ef43a47`
- `state/records/reviews/inbox-6646251725/arithmetic.json` `ae9b974cd0a12fc4b51e03bad2d08edfdafdf5f4`
- `state/records/reviews/inbox-6646251725/replay.py` `0d4932d1265c96faf3dda540e0d0694756d05d56`
- `state/records/reviews/inbox-6646251725/report.md` `51e925ee9201cf4dadb01da186a8460f2cb440bd`
- `state/records/reviews/inbox-6651023083/report.md` `ae1952d7a08a9bc96c97500cbbbfce7cb9f5edd1`
- `state/records/reviews/inbox-6651023083/resource-model.json` `e4a534c84bf013be2a4f020b50af8c3047ec2b7c`
- `state/records/reviews/inbox-6653416078/checkpoint.md` `7410ba013c5e506645d566f87c9b45e35381ab91`
- `state/records/reviews/inbox-6653416078/evidence-manifest.json` `fbdc3d103f11dd8a541fd5fb37d79e0b000a626e`
- `state/records/reviews/inbox-6653416078/report.md` `46660075045b1ae48ace3c7b482919daeea0ccc2`
- `state/records/reviews/inbox-6653416078/resource-model.json` `59705556ec988632c0669752a42d03e4668674ac`
- `state/records/reviews/inbox-6657883434/evidence.json` `a9953631719c3558e47b3517eb91a86129a12c2f`
- `state/records/reviews/inbox-6657883434/model.json` `e0e0ce0fd60a4bf52c29551c4580cb8ef1dd9606`
- `state/records/reviews/inbox-6657883434/progress.md` `acace4cc4ae883e3e7dd099796f02533405a368b`
- `state/records/reviews/inbox-6657883434/report.md` `4d5ff11e6e8929946e849949ee939356e72f934d`
- `state/records/reviews/inbox-6659992544/evidence.json` `b7b38199c4495ad3aaf980850258c3ab72ff70eb`
- `state/records/reviews/inbox-6659992544/model.json` `ea65dd8d254bff8007c30ff1256e7052f1103c55`
- `state/records/reviews/inbox-6659992544/report.md` `78e6adc6cef6fda2c3ad6b9d8de5690d12ec08a5`
- `state/records/reviews/inbox-6683037641/arithmetic.json` `9f44022a6440e72937bc867bc4dac13bc0efff5a`
- `state/records/reviews/inbox-6683037641/evidence-manifest.json` `70a345bd631179d9df7a7fcefa533b6bf65b3228`
- `state/records/reviews/inbox-6683037641/model.json` `9a9ef9bb50140467bf1279223cfc51b40e486422`
- `state/records/reviews/inbox-6683037641/report.md` `906a03dd6147e698efafd293f73a8c81685a9582`
- `state/records/reviews/scanner-146/README.md` `b68153798689b3377e39de06262a8a85882342c7`
- `state/records/reviews/scanner-146/asm-comparison.json` `439a8f80927ded594f04f43bf904f427c7e9ae06`
- `state/records/reviews/scanner-146/compare_asm.py` `ded489e7f026ea5aa2d48b1ffdecab6c2eec5562`
- `state/records/reviews/scanner-146/complete-evidence-manifest.json` `f854538a2341059fd34e7f3f066f4e15a250cae2`
- `state/records/reviews/scanner-146/ctor2-weighted.tsv` `ed4464fca2bd34ad7efeed2973bc6b48ed032fb4`
- `state/records/reviews/scanner-146/ctor2s-weighted.tsv` `2ca827b0f639807ff91815cff5492ed822af2727`
- `state/records/reviews/scanner-146/replay-result.json` `cb9086381d539dc19c349a0244dc1bb5aa577850`
- `state/records/reviews/scanner-146/replay.py` `15293e2e2c73e11d30df43c3918c36917b398b73`
- `state/records/reviews/scanner-146/report.md` `9575787e53f512a1498199731f852323a4d2a222`
- `state/records/reviews/scanner-146/reproduce.py` `7b2c5f9d47b2341b7dd9510fb7b8267da341c2db`
- `state/records/reviews/scanner-146/review-cursor.json` `20f24d3768fc403d496d357f53e7237dceb5957e`
- `state/records/reviews/scanner-146/safety/SAFETY-REVIEW.md` `fc6d4662e4ce1bda151d61ed5abc37e24f874a47`
- `state/records/reviews/scanner-146/safety/fetched-pins.json` `784c0ff96641a5ef03f73fcf036ac5517a2276ed`
- `state/records/reviews/scanner-146/safety/replay-results.json` `8af5e76031cc996d1f4161ff9494e30518f62eb5`
- `state/records/reviews/scanner-146/safety/replay_safety.py` `89e2a04ab05488ac66e37575dd720c3d2557f6a5`
- `state/records/reviews/scanner-146/validation.json` `ab52a51c098c2839fe93af3e2a10b2bb6970f996`
- `state/records/reviews/v3/evidence-manifest.json` `a6a7c47aa287c817e75d306e981a8055f5170144`
- `state/records/reviews/v3/finalizer-costs.txt` `7601e65b0ae5a0b75a82a0367c177465808ccae3`
- `state/records/reviews/v3/replay-results.json` `86920bf7ba062f64c6878b5381890dc331c2a56b`
- `state/records/reviews/v3/replay.py` `5194cec190e58d864bb9b59f85bc4b0e4eb5e4e3`
- `state/records/reviews/v3/report.md` `d881c076fcbe6c33851e0c67b8b53f822822d390`
- `state/records/structural-findings.md` `6739635b3158680347a444c6441aaa40461842f2`
- `state/records/structural-replay.json` `33ebe2432fd0264b6470b970a9f449a142a2bfe4`

## Queue state carried over (from the cursor, verbatim)

```json
{
 "inbox_briefs": [
  {
   "comment_id": 6645643541,
   "status": "completed",
   "completed": true
  },
  {
   "comment_id": 6646251725,
   "status": "completed",
   "completed": true
  },
  {
   "comment_id": 6651023083,
   "status": "finished",
   "completed": true
  },
  {
   "comment_id": 6653416078,
   "status": "finished",
   "completed": true
  },
  {
   "comment_id": 6657883434,
   "status": "finished",
   "completed": true
  },
  {
   "comment_id": 6659992544,
   "status": "finished",
   "completed": true
  },
  {
   "comment_id": 6683037641,
   "status": "finished",
   "completed": true
  }
 ]
}
```
