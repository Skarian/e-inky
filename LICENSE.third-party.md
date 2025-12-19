# Third-Party Notices

## CREngine-NG

- **Source:** <https://gitlab.com/coolreader-ng/crengine-ng>
- **Pinned commit:** `054875c021539c21e93665fcfc969d61d5a3e9e8`
- **License:** GPL-2.0 (per upstream `LICENSE`)
- **Usage:** Vendored C/C++ sources (via `scripts/update-crengine.sh`) for EPUB rendering and related functionality. Any local modifications are tracked in `crates/crengine/patches/`. The vendor directory `crates/crengine/vendor/` is populated locally and ignored in Git to avoid inflating diffs, so re-run the update script when refreshing dependencies.
