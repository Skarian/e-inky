# CREngine vendoring

The CREngine-NG C/C++ sources are vendored locally to support EPUB ingestion and rendering for the Xteink X4 pipeline.

## Upstream source

- Repository: <https://gitlab.com/coolreader-ng/crengine-ng.git>
- Pinned commit: `054875c021539c21e93665fcfc969d61d5a3e9e8`
- License: GPL-2.0 (see `LICENSE.third-party.md` in the repository root)
- Local vendor directory: `crates/crengine/vendor/` (ignored in Git to keep large upstream drops out of diffs)

## Updating or re-vendoring

Use `scripts/update-crengine.sh` to refresh the vendored sources to a new upstream commit or to recreate a clean copy of the pinned revision.

The script:

1. Clones the upstream repository.
2. Checks out the pinned commit.
3. Copies the upstream contents into `crates/crengine/vendor/`.
4. Applies every patch found in `crates/crengine/patches/` (alphabetical order).

### Patch set

Keep upstream modifications minimal and tracked as standalone `.patch` files in `crates/crengine/patches/`. Prefer small, well-scoped changes with descriptive filenames and comments inside the patch if the intent is non-obvious.

After updating upstream, review the patch set to confirm each patch still applies cleanly and remains necessary.
