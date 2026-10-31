# Inputs of the unrolled room walk's full-decoder trial

These are copied unchanged from the P74-C decoder trial at `research/p74c-decoder`
`3870c12`, so both trials decode the same bytes:

- `heldout-spec.json`: the same-dump held-out windows;
- `different-dump/windows.json`: the primary corpus's windows (wikidata-20261026), with
  their frozen hashes;
- `reference-p74c-decoder-frozen.json`: that trial's `frozen.json`. This trial's freeze
  asserts that every corpus they share has the same compressed and decoded SHA-256.

The P74-C branch is not modified.
