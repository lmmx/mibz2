claim: none
by: chatgpt
outcome: done
evidence: PR #1183 at e12e0aa1ac8de4a6a80b98883ff4db30df7711cd; master 37dc02bc5b15cc393d5b44b59c630e2d7c3dc332; src/bwt.rs; src/lib.rs

**Finding.** The fused post-walk breaks a documented postcondition of the public
`InverseBwt::invert_in_place` method for a caller that wraps `Segmented`.
On success the method promises to leave the same inverted bytes in its argument
as `invert` would produce (PR head, `src/bwt.rs:70-74`).

**Verification.** I built both revisions with Rust 1.90.0 and ran the same public
`decode_with` check with scalar kernels and Python `bz2.compress(b"banana bandana\n")`.
The wrapper's in-place method first computes a reference into a separate vector
using `Plain::invert`, then delegates to `Segmented::invert_in_place` and compares
the supplied vector to the reference whenever the delegate returns true.
Master decoded all 15 bytes. The PR returned true with the BWT last column
`aanndnbb\n naaaa` still in the supplied vector. The reference held `banana bandana\n`.

**Cause and fix.** At `src/lib.rs:882-884` the generic decoder installs the final
output in thread-local storage before calling the wrapper. Its nested Segmented call
writes to that output and returns early at `src/bwt.rs:335-336`. Provide a separate
opt-in fused operation with an explicit destination and result, keeping the original
trait methods' promised output for adapters. Keep the wrapper check as a regression
test. This review covers the interface. The candidate's timing and other paths were
outside its scope.
