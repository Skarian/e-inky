# CREngine patch set

Store local modifications to CREngine-NG as `.patch` files in this directory. The `scripts/update-crengine.sh` script applies patches in alphabetical order after copying upstream sources into `crates/crengine/vendor/`.

Guidance:

- Keep the patch set minimal and focused on integration needs.
- Include context in commit messages or patch headers where possible.
- Re-run the update script after adding or editing a patch to verify it applies cleanly.
