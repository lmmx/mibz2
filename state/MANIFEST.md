# Migration manifest: guardian

- **Source:** `review/compact30-guardian-shadow` at `f9dab3d2b40739d89d613f86e8c2d67bec3159e4` (its exact final head, every inbox comment finished).
- **Destination:** `team-state/guardian`, a continuation of that commit.
- **Moves:** `docs/journal/compact30-guardian-shadow` → `state/records`; `state/records/cursor.json` → `state/cursor.json`; `state/records/decisions.md` → `state/decisions.md`
- **Removed:** every other tracked file; they remain on `review/compact30-guardian-shadow` at `f9dab3d2b40739d89d613f86e8c2d67bec3159e4`.
- **Changed in content:** only `state/cursor.json`, which gains `migrated_from`.

## Files (path, blob at migration; unchanged from the source except the cursor)

- `state/cursor.json` `6965d424903146cffbd223e712a6fbbc73129d25`
- `state/decisions.md` `1382d4ee7c51681e45e2b2ff79549f10e7b56ce0`
- `state/records/README.md` `0555eceb329cea213586e5bf27804879e80384d0`
- `state/records/astra-v3-replayed.json` `86920bf7ba062f64c6878b5381890dc331c2a56b`
- `state/records/d008/d008-results.json` `7185628523a95811b5499b3e3004cd37ef84850c`
- `state/records/d008/event.json` `72dc1939bdee6f75efd1d2cc38098aa9ecf8dcab`
- `state/records/d008/manifest.json` `ca6b3bf8a4f72e477419fd808273367eceee9ab2`
- `state/records/d008/parser.py` `e39d688d7bc117cac6e2c96864dce44ba5da850c`
- `state/records/d008/replay_d008.py` `f8c8e713cce6cf9248239376611916ac159592f4`
- `state/records/d009-receipt.json` `31b9af963c23ba217129346bef01f4da2a8e0b2a`
- `state/records/d010-receipt.json` `78f831a3c016748e2846252925b9e8c35c4bbfd3`
- `state/records/d011-receipt.json` `cd7fa35333f0c075e1a229ea51bacd3f86ae16b1`
- `state/records/d012-receipt.json` `ebc925869ba7806c7c7254465964e154dd5a097b`
- `state/records/d013-receipt.json` `8e239692812a2bc2f8fb08192c01fef52c654710`
- `state/records/d014-receipt.json` `44e5e0c34b63fac6de9123944848e21b768cb664`
- `state/records/d015-receipt.json` `a243bdf9614d980eeea538abbe279a5ef4e3ed00`
- `state/records/d016-receipt.json` `1372192d0759055c622f1fe13a4074b3256217fd`
- `state/records/d017-receipt.json` `ec9eae698ad3baebbc88c82606cb044cd0451846`
- `state/records/d018-receipt.json` `ded9c03da743d9719f946668f2ebd2462b57c3b3`
- `state/records/d019-receipt.json` `045f03155188faa806b49b63f10a79884296e14c`
- `state/records/d020-receipt.json` `34abae4b2ca29f69645519d0e79cb158530459e7`
- `state/records/d021-receipt.json` `0b7380050c92873ab600e013752442380199d91b`
- `state/records/d022-receipt.json` `d6d8f6a6bb5dcff98c03dfbde799895dcab4217c`
- `state/records/d023-receipt.json` `5189218f6f68e67ed21e1aba1e952c2828975f0b`
- `state/records/d024-receipt.json` `b12c1791e07c4f6ec34e205999e99586cc0f8902`
- `state/records/d025-receipt.json` `c894144582a7c3d96e49178627c20b63a6109587`
- `state/records/d027-reproduce.py` `9604169d7ac7e988eeff73a0b90f53a63223ed99`
- `state/records/d027-reproduction.json` `0faddcc850767f92a497d73cbc0916d578d911ca`
- `state/records/d028-reproduce.py` `c2d60541e56f0f082b816b6e9043ba2cec09870a`
- `state/records/d028-reproduction.json` `8963e08722211e0b8db150d1f4c6f0c7a2840fc9`
- `state/records/d029-reproduce.py` `1800922620c83fce1d1833b293b1cb5144a74003`
- `state/records/d029-reproduction.json` `f786479a6a7fa9421b2d1dc83ef3a63c6af34331`
- `state/records/d029-test-results.json` `f293e0834ba2b408e81d98f9ebb3c4d36d5289ad`
- `state/records/d030-reproduce.py` `6ccdebe8bb79f090484eb721937209825ddbfb65`
- `state/records/d030-results.json` `c0bd0fe5a517a10c1b25626b23bcd94123861e53`
- `state/records/d031-probes.py` `9266b920c2664ff35a8a81a99d871d2e38e3c123`
- `state/records/d031-results.json` `a6bd91ecb138b0cf642f6fa6ec7027008451e30d`
- `state/records/d032-probes.py` `cce64b4a461921c0b278d448845d1546ac3c8fb2`
- `state/records/d032-results.json` `51a6915a4d4482356022c9a7e6d44ce1b908acba`
- `state/records/d033-probes.py` `c74e9c7b9485ae90c6692ce50aaf88e972595fb3`
- `state/records/d033-results.json` `4ea11a07f3d0395110f40e5ee1c7d134ddd01d9a`
- `state/records/p1-manifest.json` `5eacdc34b127b11b8f3b39daf844832db2aed20c`
- `state/records/p1-results.json` `a992e7c468ebed57b5520975f9fecb63868c215b`
- `state/records/p2-manifest.json` `be79f6823aec10dc03b3676510713c9e43f52169`
- `state/records/p2-results.json` `c856cab21132bebf678e0a899668f1ea41a61e5f`
- `state/records/p3-incoming.json` `0c55f118f4514f5e5996661e74fe54c3b07a90d9`
- `state/records/p4-channel-log.md` `ab18784fa4f2688efc7d27c0848213dc292c5bf6`
- `state/records/p4-closure-manifest.json` `e9f726d3b8e843ba32cb60eae695582543de2077`
- `state/records/p4-closure-results.json` `7043345c1ac2a9e8a69a6e8746760a38e8d5de68`
- `state/records/p4-outgoing.json` `ee1c131117a72a2d04e12bc9ef658286b5a730e6`
- `state/records/p4-receipt.json` `b8d5e214f64433d5124a1db959a58b4f4b6fee4e`
- `state/records/peer-scanner/asm-comparison.json` `439a8f80927ded594f04f43bf904f427c7e9ae06`
- `state/records/peer-scanner/replay-result.json` `cb9086381d539dc19c349a0244dc1bb5aa577850`
- `state/records/peer-scanner/safety-replay-results.json` `8af5e76031cc996d1f4161ff9494e30518f62eb5`
- `state/records/replay_p1.py` `3f6b8cdb5d41cb8951024686075971ac5d973139`
- `state/records/replay_p2.py` `ee50ff20519b20775aa190a572dacabc9889460b`
- `state/records/replay_p4.py` `0715bcb00066f05417384124b92ccb1332aad6ee`
- `state/records/webhook-test-6632161918.json` `b810a4899de204ac47573fab42573e78e6f91a1a`

## Queue state carried over (from the cursor, verbatim)

```json
{
 "consumed_incoming_comment_ids": [
  6630529621,
  6630530217,
  6630638365,
  6630672389,
  6632161918,
  6632963394,
  6633116288,
  6633793271,
  6635380133,
  6635419092,
  6637300905,
  6639998280,
  6640153383,
  6641512290,
  6641672441,
  6643481606,
  6643744616,
  6643809123,
  6643992749,
  6644364456,
  6644912625,
  6645304470,
  6645920904,
  6646249200,
  6681847165,
  6682032075,
  6682106949,
  6682260206,
  6682936439,
  6683783830,
  6683967083,
  6684935718,
  6685165401
 ],
 "own_outgoing_comment_ids": [
  6630519853,
  6630589596,
  6630627391,
  6630629015,
  6630660444,
  6631014855,
  6632636427,
  6633274082,
  6633918984,
  6635476486,
  6635620697,
  6637353454,
  6640069360,
  6641768409,
  6643850845,
  6643927959,
  6644051935,
  6644422056,
  6644979187,
  6645337230,
  6645964170,
  6646304926,
  6681996710,
  6682139215,
  6682239005,
  6682363998,
  6683039047,
  6683861491,
  6684025721,
  6685099382,
  6685253567
 ],
 "inbox_comments": [
  {
   "comment_id": 6681847165,
   "status": "finished",
   "decision": "D026",
   "review_comment_id": 6681996710
  },
  {
   "comment_id": 6682032075,
   "status": "finished",
   "decision": "D027",
   "review_comment_id": 6682139215
  },
  {
   "comment_id": 6682106949,
   "status": "finished",
   "decision": "D028",
   "review_comment_id": 6682239005
  },
  {
   "comment_id": 6682260206,
   "status": "finished",
   "decision": "D029",
   "review_comment_id": 6682363998
  },
  {
   "comment_id": 6682936439,
   "status": "finished",
   "decision": "D030",
   "review_comment_id": 6683039047
  },
  {
   "comment_id": 6683783830,
   "status": "finished",
   "decision": "D031",
   "review_comment_id": 6683861491
  },
  {
   "comment_id": 6683967083,
   "status": "finished",
   "decision": "D032",
   "review_comment_id": 6684025721
  },
  {
   "comment_id": 6684935718,
   "status": "finished",
   "decision": "D033",
   "review_comment_id": 6685099382
  },
  {
   "comment_id": 6685165401,
   "status": "finished",
   "decision": "D034",
   "review_comment_id": 6685253567
  }
 ]
}
```
