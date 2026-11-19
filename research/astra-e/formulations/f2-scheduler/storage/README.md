# Retained vector capacity under four discovery schedules

One worker processed the 233 canonical large Wikidata blocks in original stream order, using a separate reusable kernel per arm. All four accepted 228 blocks and were compared against the independent Segmented output on each admission. After each call the observer adds actual vector capacities times native element size, including the eight private path buffers at each scheduled level. It counts the first and second inverse structures and their retained data. It excludes output, input/MTF symbols, fallback scratch, stack state, vector headers and allocator metadata. The process resident peak and worker concurrency are measured by the independent benchmarks.

On frozen the dynamic first level holds 2,952,280 bytes, of which 2,048 bytes are its private paths. The scalar version holds 2,950,232. The stream high waters over the 233-block traversal are 7,197,968 bytes for both eight-path variants and 7,188,752 for scalar. Each first-level scheduled arm's private buffers reached 9,216 bytes. Reusing eight lanes on both levels reached 7,166,224 bytes in total, including 1,024 bytes of second-level integer-token paths. Its other vectors grow differently as publication order and word identities change. These differences are observed allocations for this corpus and vector history. They do not certify a process memory ratio at 7–10 workers. [Every block and arm](observed.csv) · [High waters](summary.txt)

```sh
d=research/astra-e/formulations/f2-scheduler/storage
bash "$d/run.sh" /tmp/f2-storage "$PWD/bench/moonshot/target/release/deps"
```
