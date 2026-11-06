# Migration manifest: oracle

- **Source:** `review/oracle-intention` at `382518724f59c1c4e3755a57649c318fb443923d` (its exact final head, every inbox comment finished).
- **Destination:** `team-state/oracle`, a continuation of that commit.
- **Moves:** `docs/journal/oracle/cursor.json` → `state/cursor.json`; `docs/journal/oracle/intention.md` → `state/seed-intention.md`
- **Removed:** every other tracked file; they remain on `review/oracle-intention` at `382518724f59c1c4e3755a57649c318fb443923d`.
- **Changed in content:** only `state/cursor.json`, which gains `migrated_from`.

## Files (path, blob at migration; unchanged from the source except the cursor)

- `state/cursor.json` `e2a0a1cb3e283ec84d624d90d43247add2f0db5f`
- `state/seed-intention.md` `26d975cf92f34e0724bc9d03ca223f10277b7604`

## Queue state carried over (from the cursor, verbatim)

```json
{
 "processed_comment_ids": []
}
```
