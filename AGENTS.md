# e-inky: the Xteink X4 Library Manager (Tauri) — Repository Guidelines

This repo exists to make the Xteink X4 usable for normal humans.

The device firmware/UI is minimal, EPUB handling is unreliable, and the “just put books on an SD card” experience falls apart fast. Our job is to build a desktop app that:

1. takes ordinary EPUBs,
2. renders them with a real, proven renderer (frontend),
3. captures deterministic page images,
4. converts those images into the X4’s performant native container format (XTC with XTG/XTH pages),
5. keeps a clean on-disk library,
6. syncs that library to the X4’s mounted microSD (and optionally the device’s hotspot uploader).

This file is written for code-generation agents. If you ignore it, you will ship garbage.

---

## Product Scope

### MVP: “Local EPUB → Render → Capture → XTC → SD Sync”

- Import one or more local `.epub` files.
- Render EPUB into a paginated e-reader view using **foliate-js** (frontend).
- Capture each page deterministically using **SnapDOM** (frontend).
- Convert captured pages to:
  - **XTH** (preferred, 4-level grayscale, better readability), or
  - **XTG** (fallback, 1-bit monochrome).
- Package pages into **XTC** container with metadata; chapters are part of the spec and we should respect it even if firmware support lags.
- Maintain a local library that always contains:
  - the original EPUB
  - its matching XTC
  - a metadata file and conversion manifest (settings + versioning)
- Sync to a user-selected mounted volume (the microSD card mounted by macOS/Windows).
- Never write OS junk files into the device library (`.DS_Store`, `._*`, `Thumbs.db`, etc.).

---

## Xteink X4/X3 Constraints You Must Respect

### Supported formats and transfer paths

- Device supports books: TXT (UTF-8) and EPUB, but EPUB can be “garbled” and images may not display reliably.
- Primary transfer: microSD card mounted by OS.
- Alternate transfer (TBD): device hotspot + web uploader at `192.168.3.3` (password commonly `12345678`).

### Screen target / device presets

We must support these device presets:

- **Xteink X4**: portrait **480×800**
- **Xteink X3**: portrait **528×792**

These dimensions define the **capture canvas** and the **XTC page payload dimensions**.

### Typography and margins

- Keep margins conservative; avoid tiny fonts.
- Rendering must be stable and deterministic across runs (same input + same settings => same pages).

---

## Architecture (Frontend Renderer + Backend Encoder)

### Core idea

- **Frontend owns layout fidelity** (HTML/CSS rendering + pagination).
- **Frontend produces page frames** (captured images / pixel buffers).
- **Rust backend owns format correctness** (XTH/XTG/XTC encoding byte-for-byte, validation, library + sync).

This eliminates the “Rust EPUB renderer” problem and focuses Rust where it matters: correctness + speed.

### Tech choices (locked for now)

- Desktop: **Tauri**
  - Rust backend
  - Frontend: **Next.js (Pages Router)** + TypeScript
- Renderer: **foliate-js**
- Capture: **SnapDOM**
- Avoid “install Calibre and call ebook-convert” as a hard dependency. Optional integration is fine, not required.

---

## Renderer + Converter Feature Parity Target (Match https://x4converter.rho.sh/)

We aim to match the reference converter’s controls and output semantics.

### File handling

- Drag/drop or file picker for one or more EPUB files.
- Remove individual files and “Clear All”.

### Device

- Device preset: X4 (480×800), X3 (528×792).

### Text settings (must implement)

Fonts:

- Font face:
  - Default
  - Literata
  - Noto Sans
  - Noto Serif
  - Open Sans
  - Source Serif 4
  - Lora
  - Merriweather
- Custom font: user-supplied font file(s) selectable and applied to rendering

Text layout:

- Font size (px)
- Font weight (numeric slider/input; map to CSS font-weight deterministically)
- Line height (%)
- Margins (px)
  - Implement as a single value that applies to all sides (matches typical converter UX)
  - (Optional later) per-side margins if we want more control, but keep parity first
- Ignore document margins (toggle)

Text shaping:

- Text align: Default / Left / Right / Center / Justify
- Word spacing presets:
  - Small (50%), Condensed (75%), Normal (100%), Expanded (125%), Wide (150%), Extra Wide (200%)
- Hyphenation: Off / Algorithmic / Dictionary
- Hyphenation language: Auto + selectable language list

### Image settings (must implement)

- Quality mode:
  - Fast (1-bit → XTG)
  - High Quality (grayscale → XTH)
- Dithering amount (percent)
- Dark Mode (Negative) toggle (invert page)

### Progress bar (must implement)

The reference converter can render progress UI into the page; we must bake this into the captured output.

- Enabled toggle
- Position: Top / Bottom
- Full width toggle
- Progress line toggle
- “Book” toggle
- “Chapter” toggle
- Chapter marks toggle
- Content fields/toggles:
  - Page (X/Y)
  - Book %
  - Chapter (X/Y)
  - Chapter %
- Progress bar font size (px)
- Edge margin (top/bottom) (px)
- Side margin (left/right) (px)

### Chapters / navigation (must implement)

- Display EPUB TOC (chapters list)
- Jump to chapter
- Previous / Next page
- “Page N / M” indicator
- Refresh / re-render

### Export controls (must implement)

- Export to XTC (current file)
- Export All Files (batch)
- Export Current Page (XTG) (debug/inspection)

---

## Rendering + Capture Pipeline (Frontend)

### Determinism rules (non-negotiable)

- Fix viewport size exactly to the selected device resolution (X4/X3).
- Fix device pixel ratio for capture (dpr=1 unless explicitly changed).
- Ensure fonts are loaded before pagination + capture.
- Ensure images are loaded before capture.
- Disable animations and transitions in the capture subtree.
- Never allow reflow between page captures.

### foliate-js integration approach

- Use `<foliate-view>` (web component) as the core renderer.
- Open EPUB from a `Blob` created from bytes supplied by Tauri (frontend does not read the filesystem directly).
- Drive pagination + navigation programmatically:
  - open book
  - prev/next page
  - go-to TOC entries
- Track progress using relocate events and/or the view’s range/progress APIs.
- Apply our settings using **scoped CSS overrides** and explicit px-based layout.

### Progress bar rendering approach

- Render progress UI inside the paginated view (top/bottom) so it is included in the capture.
- Keep it deterministic: no timers, no “live” animation.

### SnapDOM capture approach

- Capture the exact page subtree (including progress overlay) to canvas/blob.
- Enforce output dimensions exactly equal to the device preset.
- Capture per page, streaming results to backend incrementally.

### Data sent to backend (standardize)

For MVP: send **grayscale8** frames.

- Frontend captures to canvas → reads pixels → converts to grayscale8 `Uint8Array`
- Backend handles dither/quantize + packing to XTG/XTH

Do not ship PNGs unless you have a measured reason.

---

## Backend Conversion Pipeline (Rust)

### Responsibilities

- Validate incoming frames match expected dimensions.
- Apply conversion settings deterministically:
  - grayscale quantization policy
  - dithering (percent)
  - dark mode (if not already applied at capture)
  - XTH LUT quirks per spec
- Encode to XTG/XTH bit-exact.
- Build XTC container with correct offsets, sizes, and metadata tables.
- Stream-write XTC (do not hold entire book in RAM).
- Produce `conversion.json` capturing:
  - renderer settings
  - converter settings
  - versions of foliate-js + snapdom + encoder crate
  - page count + optional per-page hashes

---

## File Formats We Produce (XTC/XTG/XTH)

### Authoritative spec

- **XTC/XTG/XTH/XTCH Format Technical Specification**:
  - https://gist.github.com/CrazyCoder/b125f26d6987c0620058249f59f1327d

You will implement exactly what the spec says. Do not invent fields, sizes, byte orders, or “probably fine” shortcuts.

### Key implementation requirements (high signal)

- All multi-byte values are **Little-Endian**.
- XTG and XTH have **22-byte headers** and raw bitmap payloads.
- XTH storage is **column-major / vertical scan order** (weird but required).
- XTH grayscale mapping is **non-linear** on device (middle values swapped); follow the spec’s LUT notes.
- XTC header is **56 bytes**, includes offsets (metadata/index/data/thumb/chapter).

### Validation strategy

- Build test suite that:
  - reads the container
  - validates offsets are in-bounds and aligned with payload sizes
  - validates each page’s embedded XTG/XTH header and dataSize
  - optionally renders a page back to PNG for sanity checks

---

## Library Model (On Disk)

### Library root

User chooses a single directory as library root (default under app data).

Recommended layout:

- `library/`
  - `books/`
    - `<book_id>/`
      - `source.epub`
      - `book.xtc`
      - `meta.json`
      - `conversion.json`
      - `cover.png` (optional)
  - `index.sqlite`

### Identifiers

- `book_id` should be stable and filesystem-safe:
  - prefer `sha256(epub_bytes)` prefix + short slug (title)
- Filename on device:
  - `"<Title>.xtc"` sanitized, length-limited.

---

## Project Structure & Module Organization (Target Layout)

- `src-tauri/` — Tauri Rust app
  - `src/commands/` — IPC command handlers
  - `src/state/` — app state, config, caches
- `web/` — Next.js (Pages Router)
  - `pages/` — UI routes
  - `components/` — reusable UI (including `EpubReader`, `SettingsPanel`, `CaptureRunner`)
  - `lib/` — client-side helpers (no secrets)
- `crates/` — Rust workspace crates
  - `xtc` — read/write XTC, XTG, XTH; pure and tested
  - `encoder` — grayscale buffers -> XTG/XTH (dither/quantize/LUT)
  - `library` — library index, metadata, hashing, filenames
  - `sync` — mount detection + sync plan execution
- `docs/` — protocol notes, screenshots, device quirks
- `assets/` — icons, sample fixtures (small)

---

## IPC Rules (Updated)

Frontend still does not write to the filesystem. It may hold data in memory and send to backend.

All IO goes through Tauri commands. Expected commands:

Library + metadata:

- `import_epub(paths)`
- `list_library()`, `get_book(book_id)`, `delete_book(book_id)`
- `fetch_metadata(book_hint)`
- `get_epub_bytes(book_id)` -> bytes for frontend blob

Conversion streaming:

- `start_conversion(book_id, options)` -> returns `{ conversion_id, width, height }`
- `push_frame(conversion_id, page_index, grayscale_u8)` -> streaming frames
- `finish_conversion(conversion_id)` -> finalize XTC + manifests

Sync:

- `sync(book_id|all, target_volume)`

---

## Build, Test, and Development Commands

### Prereqs

- Rust stable (edition 2021)
- Node LTS
- Prefer `pnpm` (monorepo friendly). `npm` acceptable if you keep it consistent.

### Commands (expected)

- `pnpm install`
- `pnpm dev` — run Next.js UI
- `pnpm tauri dev` — run desktop app
- `pnpm lint` — frontend lint
- `pnpm test` — frontend tests (if any)

Rust:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -D warnings`
- `cargo test --workspace`

---

## Coding Style & Naming Conventions

### Rust

- `rustfmt` mandatory.
- `clippy` clean (no warning debt).
- Error handling:
  - `thiserror` for public errors
  - `anyhow` only in binaries/glue
- Logging:
  - `tracing` + `tracing-subscriber`
  - never log secrets or full file contents

### TypeScript / Next.js

- Prettier + ESLint enforced.
- No giant React components.
- Keep state management simple; prefer `zustand` if needed.

### Naming

- Rust crates/modules: `snake_case`
- TS files: `kebab-case.tsx` or `snake_case.ts` — pick one.
- Device/library filenames: sanitized, predictable, stable.

---

## Testing Guidelines

### What must be tested (backend)

- XTG/XTH encoding/decoding (bit-exact).
- XTC container read/write round-trip (header fields + offsets).
- Golden fixtures:
  - generate a small known XTC with 2–3 pages
  - verify it opens on device (manual verification) AND decodes correctly in tests

### What must be tested (frontend)

- Settings application: changing font size/weight/margins/line-height produces deterministic layout given the same EPUB + preset.
- Capture pipeline: capture output dimensions are exact; repeat-capture of page N matches (hash equality) when inputs are unchanged.

### Integration tests

- “Sync plan” testable against a temp directory representing a mounted SD.
- End-to-end smoke test (dev-only):
  - load a known EPUB fixture
  - render + capture first 3 pages
  - build XTC
  - decode first page back to PNG

---

## Security & Privacy

- Treat all EPUBs as untrusted input:
  - validate zip structure
  - limit file sizes
  - avoid path traversal (“zip slip”)
- Renderer security:
  - do not execute EPUB embedded scripts
  - sandbox iframes as much as platform allows
- API keys (if/when added) must be stored using OS keychain/credential vault:
  - do not store keys in plaintext config
- No telemetry by default.

---

## Commit & Pull Request Guidelines

### Commits

Use conventional commits:

- `feat: ...`
- `fix: ...`
- `chore: ...`
- `docs: ...`
- `refactor: ...`
- `test: ...`

### PR checklist (minimum)

- What changed + why
- Screenshots for UI
- Test evidence (commands run)
- Notes on device validation if relevant

---

## Project Plan (Feature-by-Feature)

Order matters. Build confidence in rendering/capture first; format correctness second.

### Phase 0 — Foundations (Repo + plumbing)

Feature: Workspace + app skeleton

- Create Rust workspace crates: `xtc`, `encoder`, `library`, `sync`.
- Tauri commands skeleton + typed IPC payloads.
- Next.js pages + minimal UI shell.

Acceptance:

- `pnpm tauri dev` runs.
- `cargo test --workspace` runs.
- A dummy command roundtrip works.

---

### Phase 1 — EPUB Renderer (Frontend) BEFORE format work

Feature: foliate-js integration (preview-quality first)

- Add foliate-js; build `EpubReader` component using `<foliate-view>`.
- Load EPUB bytes from backend (`get_epub_bytes`) into `Blob`.
- Implement page navigation + TOC extraction + TOC jumps.
- Implement device preset sizing with fixed dimensions (X4/X3).
- Implement the text settings UI + application:
  - font face (including custom font)
  - font size, weight
  - line height
  - margins + ignore doc margins
  - text align
  - word spacing
  - hyphenation + language
- Implement progress bar overlay (top/bottom) baked into the view.

Acceptance:

- Load an EPUB and flip pages reliably.
- Changing font size/weight/margins/line-height visibly affects layout and stays stable.
- Progress overlay is visible and deterministic.
- Chapters list renders and can jump.

---

### Phase 2 — Deterministic Capture (Frontend) + streaming plumbing

Feature: SnapDOM capture runner

- Implement `CaptureRunner` that:
  - iterates pages and waits for stable layout + loaded assets
  - captures exact page subtree at exact device dimensions
  - converts capture to grayscale8 `Uint8Array`
- Implement conversion streaming IPC:
  - `start_conversion` (returns conversion_id + expected dims)
  - `push_frame` per page
  - `finish_conversion`

Acceptance:

- Capture first N pages without drift (repeat-capture hash equality).
- Streaming sends frames to backend without batching the whole book in memory.

---

### Phase 3 — XTG/XTH/XTC Core (Backend correctness)

Feature: `crates/xtc` format I/O

- Implement XTG/XTH structs with exact header sizes.
- Implement XTC writer with correct offsets.
- Implement XTC reader used for validation tests.

Feature: `crates/encoder` grayscale8 -> XTG/XTH

- Deterministic dither (percent) implementation.
- XTH column-major packing + LUT mapping per spec.
- XTG 1-bit packing.

Acceptance:

- Bit-exact unit tests for known fixtures.
- Round-trip decode tests for random buffers (property tests).
- “Decode page back to PNG” debug helper works.

---

### Phase 4 — Export + Batch

Feature: Export to XTC (single)

- Wire “Export to XTC” to run capture+encode pipeline for current book.

Feature: Export all files (batch)

- Queue multiple EPUBs; run sequentially; show progress.

Feature: Export current page (XTG)

- Debug export for currently visible page.

Acceptance:

- Exports produce files that open on device.
- Batch export can run unattended.

---

### Phase 5 — Local Library + Index

Feature: Library layout + metadata

- Book import: compute book_id, store `source.epub`, metadata.
- Store `conversion.json` with all settings + versions.
- Maintain `index.sqlite` entries.

Acceptance:

- Re-importing same EPUB is idempotent.
- Library list persists across restarts.

---

### Phase 6 — Sync to microSD

Feature: Sync plan engine

- Detect mount volumes.
- Write XTC to device library path using sanitized filename.
- Prevent OS junk files (and optionally clean them if present).
- Optional: verify by reading back file size/hash.

Acceptance:

- Sync to a temp dir works in integration tests.
- Sync to a real microSD works manually.

---

### Phase 7 — Regression and device quirks

Feature: Golden corpus

- Maintain EPUB fixtures:
  - text-heavy
  - image-heavy
  - complex CSS
  - TOC-heavy multi-level

Feature: Regression tests

- Snapshot hashes for first 3 pages per fixture at default settings.
- Detect layout drift across dependency upgrades.

Acceptance:

- One-command validation catches renderer or encoder regressions.

---

## Agent-Specific Instructions (Read This Twice)

- Do not hallucinate device behavior. If it isn’t in linked specs/manual, label it as a hypothesis and isolate behind a feature flag.
- When implementing file formats, match the spec byte-for-byte. No creative liberties.
- Keep the MVP ruthless:
  - Local EPUB import → preview → convert → sync
  - No “account systems,” no cloud, no overengineering
- Any “download books from the internet” feature must use legal sources (OPDS catalogs for public domain, etc.). No exceptions.
- Keep this file up to date as the repository evolves (structure, commands, style, testing tools, and renderer/capture constraints).

---

## References (Authoritative)

- XTC/XTG/XTH spec:
  - https://gist.github.com/CrazyCoder/b125f26d6987c0620058249f59f1327d
- XTH generator (sanity checking):
  - https://gist.github.com/CrazyCoder/31f02014a1d569986c7b9940e775bb5d
- X4 User Manual (transfer + device behavior):
  - https://cdn.shopify.com/s/files/1/0759/9344/8689/files/X4_User_Manual_ENG.pdf
- Reference converter (parity target):
  - https://x4converter.rho.sh/
- foliate-js (renderer):
  - https://github.com/johnfactotum/foliate-js
- SnapDOM (capture):
  - https://github.com/zumerlab/snapdom
