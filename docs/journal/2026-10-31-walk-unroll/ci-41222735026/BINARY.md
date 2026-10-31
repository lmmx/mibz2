# The exact timed binary of CI run 41222735026

- **Location:** `walk_arm.gz` is the `walk_arm` all three jobs built and timed, gzip -9.
  Unzipped, it is 2,687,088 bytes.
- **SHA-256:** `48ddfd16aea51eff9e44951aa48c9600cfa537fc0789f1857893ef1cc20b1201`, identical in
  r1, r2 and r3 (each job's `binary.sha256`).
- **Build:** at `b145e14`, rustc 1.97.0, `cargo build --locked --release --example walk_arm`,
  no `RUSTFLAGS`, on GitHub `ubuntu-24.04`.
- **Kept here** because the run's artifacts expire after 30 days.
- **Check:** `gunzip -c walk_arm.gz | sha256sum`.
