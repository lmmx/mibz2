# Native sparse first-return constructor

The compiled experimental constructor takes the maximal original L runs, their original marked row and the depth-k context endpoints and modal bytes. Its loader deliberately discards the supplied model's precomputed default successor and exception ranges. Within a timed call it rebuilds LF image positions from byte counts, extracts exception/synthetic ranges, composes unanimous contexts, transports affine labels and closes all virtual anchors with the weighted interval FIFO. It assigns retained ordinals to exceptional labels during the physical source pass, so the final publisher only needs a source ordering and adjacency coalescing. The resulting triples are (retained source ordinal, retained target ordinal, elapsed LF steps). [Core](ranges.hpp) · [Native sequence](rope.hpp) · [FIFO closure](sweep.hpp) · [Input and output ABI](bridge.cpp)

Three builds independently passed 9,789 tiny marked models and all 498,084 canonical triples from thirty original models, including the optimized build used by the price harness. The undefined-behavior sanitizer reported no issue in these checks. These gates compare every physical return for tiny columns and the complete raw output of the independent original-run membership constructor on real columns. [All build flags and local ELF hashes](verification.json) · [Verifier](verify.py) · [Independent real hashes](../real.csv)

The registered hardware screen gives the candidate its maximal runs and ready depth-eight model boundaries and times production capped scalar-cap to full inverse output. The candidate's measured deliverable ends at the canonical return descriptors. Full discovery, their marked temporal walk and final output still carry costs. Mutable vectors and standard-library allocator behavior follow the normal native invocation, with one reusable solver per arm. [Gate](PLAN.md) · [Measurement protocol](price/PLAN.md)

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
d="$e/acquire/ranges/native"
g++ -O3 -DNDEBUG -std=c++20 -shared -fPIC "$d/bridge.cpp" -o "$tmp/libranges.so"
python3 -B "$d/verify.py" "$tmp/libranges.so" tiny | diff - "$d/checked-small.json"
python3 -B "$d/verify.py" "$tmp/libranges.so" real "$original" "$models" "$e/contexts/inputs.csv" "$d/../real.csv" | diff - "$d/checked-real.json"
```
