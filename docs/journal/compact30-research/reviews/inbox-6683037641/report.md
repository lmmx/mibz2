# Inbox 6683037641 — live bytes and three versus four tables

Status: in progress; proposals only. No candidates built or timed.

Source pin: f4f99ab82652411feb4a5a6e2ccb96d04902fb58. Evidence archive: cbc6eb14b065fdb7092911147ce3374f63541467.
The source pin's journal says “Not yet run”; the later archive's commit.txt and occupancy metadata name that source pin. These are different roles for the two commits, not missing benchmark evidence.

The 320 archived rows have one hash, one output size, no errors and 20 unique rounds per cell. Paired geometric ratios reproduce the published central estimates: 4×3 1.016955; 4×4 1.003463; exploratory 3×4 1.096728. Neither primary passes. No counter evidence exists.

Static findings:
- T has at most 4n accessed bytes, not its 5.4 MB capacity.
- Cooperative walk does not read ll or rle. ll is dead after build; rle is filled by assembly afterwards.
- Segment Vecs retain historical growth. The reserve formula is not the measured 6.1 MB: at n=900000, 1024 segments request 1,865,728 bytes in total before growth.
- Segment writes touch prefixes, not reserved tails. A write-stream footprint and retired allocation are different quantities.
- The cooperative path explicitly disables existing in-place ll reuse; a separate proposal can extend that lifetime reuse without changing the constructor.
