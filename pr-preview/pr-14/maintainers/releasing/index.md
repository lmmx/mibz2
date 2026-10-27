# Releasing

The crate and the Python package are released at one version, from one commit.

## Deploy Release

Run the **Deploy Release** workflow (`deploy.yml`) with a bump (patch, minor or major). It runs `just release`, which:

1. bumps the version in `pyproject.toml` (`uv version --bump`) and sets the crate's in `Cargo.toml` to match;
2. writes the crate's changelog with release-plz;
3. commits `chore(release): bump 🐍 -> vX.Y.Z` and tags `py-X.Y.Z`;
4. pushes the commit and the tag, then starts **CI** on the tag (a tag pushed with the workflow's token starts no workflow by itself).

CI on a `py-*` tag builds the wheels and the sdist and publishes them to PyPI, and publishes the crate to crates.io with release-plz (which tags `mibz2-vX.Y.Z` and makes the GitHub release). Both use Trusted Publishing, in the `pypi` and `crates-io` environments.

## Out-of-band cases

- **Tag Release** (`tag.yml`): tags an existing commit `py-<version>` at the version its `pyproject.toml` and `Cargo.toml` already hold (no bump), and starts CI on the tag. Each registry is skipped where it already has the version.
- **Release Notes** (`release-notes.yml`): makes a version's GitHub release (`v<version>`) at a given commit if it does not exist, or updates its notes, from that version's section of `CHANGELOG.md`.
