# e-inky: the Xteink X4 Library Manager (Tauri) — Repository Guidelines

This repo exists to make the Xteink X4 usable for normal humans.

The device firmware/UI is minimal, EPUB handling is unreliable, and the “just put books on an SD card” experience falls apart fast. Our job is to build a desktop app that:

1. takes ordinary EPUBs,
2. converts them into the X4’s performant native container format (XTC with XTG/XTH pages),
3. keeps a clean on-disk library,
4. syncs that library to the X4’s mounted microSD (and optionally the device’s hotspot uploader).

This file is written for code-generation agents. If you ignore it, you will ship garbage.

---

## Product Scope

### MVP: “Local EPUB → XTC → SD Sync”

- Import one or more local `.epub` files.
- Convert EPUB → paginated images at X4’s effective resolution (portrait **480×800**).
- Encode each page as:
  - **XTH** (preferred, 4-level grayscale, better readability), or
  - **XTG** (fallback, 1-bit monochrome).
- Package pages into **XTC** container with metadata; chapters are part of the specification and we should respect it but firmware support may lag.
- Maintain a local library that always contains:
  - the original EPUB
  - its matching XTC
  - a metadata file and conversion manifest
- Sync to a user-selected mounted volume (the microSD card mounted by macOS/Windows).
- Never write OS junk files into the device library (`.DS_Store`, `._*`, `Thumbs.db`, etc.).

## Xteink X4 Constraints You Must Respect

### Supported formats and transfer paths

- Device supports books: TXT (UTF-8) and EPUB, but EPUB can be “garbled” and images may not display reliably.
- Primary transfer: microSD card mounted by OS.
- Alternate transfer (TBD): device hotspot + web uploader at `192.168.3.3` (password commonly `12345678`).

### Screen target

- Use **480×800** portrait as the target canvas for rendering and page images.
- Keep margins conservative; avoid tiny fonts.

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

## Architecture

### Tech choices (locked for now)

- Desktop: **Tauri**
  - Rust backend
  - Frontend: **Next.js (Pages Router)** + TypeScript
- Conversion pipeline runs in Rust.
- EPUB rendering to pages may use:
  - a headless browser renderer (Playwright) invoked as a bundled helper **OR**
  - a pure-Rust renderer if you can match layout fidelity (unlikely early).
- Avoid “install Calibre and call ebook-convert” as a hard dependency. Optional integration is fine, not required.

### Rust crates (recommended)

- `crates/xtc` — read/write XTC, XTG, XTH; pure and tested.
- `crates/render` — EPUB → page images (PNG/gray buffers).
- `crates/library` — library index, metadata, hashing, filenames.
- `crates/sync` — mount detection + sync plan execution.
- `src-tauri/` — Tauri commands glue + app state.

### IPC rules

- Frontend never touches filesystem directly.
- All IO goes through Tauri commands:
  - `import_epub(paths)`
  - `fetch_metadata(book_hint)`
  - `convert(book_id, options)`
  - `sync(book_id|all, target_volume)`
  - `list_library()`, `get_book(book_id)`, `delete_book(book_id)`

---

## Project Structure & Module Organization (Target Layout)

- `src-tauri/` — Tauri Rust app
  - `src/commands/` — IPC command handlers
  - `src/state/` — app state, config, caches
- `web/` — Next.js (Pages Router)
  - `pages/` — UI routes
  - `components/` — reusable UI
  - `lib/` — client-side helpers (no secrets)
- `crates/` — Rust workspace crates (`xtc`, `render`, `library`, `sync`)
- `docs/` — protocol notes, screenshots, device quirks
- `assets/` — icons, sample fixtures (small)

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

### What must be tested

- XTG/XTH encoding/decoding (bit-exact).
- XTC container read/write round-trip (header fields + offsets).
- Golden fixtures:
  - generate a small known XTC with 2–3 pages
  - verify it opens on device (manual verification) AND decodes correctly in tests

### Property tests (high leverage)

- Random images → encode → decode → compare (within expected quantization for XTH).

### Integration tests

- “Sync plan” should be testable against a temp directory representing a mounted SD.

---

## Security & Privacy

- Treat all EPUBs as untrusted input:
  - validate zip structure
  - limit file sizes
  - avoid path traversal (“zip slip”)
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

## Agent-Specific Instructions (Read This Twice)

- Do not hallucinate device behavior. If it isn’t in the linked specs/manual, label it as a hypothesis and isolate it behind a feature flag.
- When implementing file formats, match the spec byte-for-byte. No creative liberties.
- Keep the MVP ruthless:
  - Local EPUB import → convert → sync
  - No “account systems,” no cloud, no overengineering
- Any “download books from the internet” feature must use **legal** sources (OPDS catalogs for public domain, etc.). No exceptions.
- Keep this file up to date as the repository evolves (structure, commands, style, and testing tools).

---

## References (Authoritative)

- XTC/XTG/XTH spec:
  - https://gist.github.com/CrazyCoder/b125f26d6987c0620058249f59f1327d
- XTH generator (useful for sanity checking thresholds/UI ideas):
  - https://gist.github.com/CrazyCoder/31f02014a1d569986c7b9940e775bb5d
- X4 User Manual (transfer + device behavior):
  - https://cdn.shopify.com/s/files/1/0759/9344/8689/files/X4_User_Manual_ENG.pdf
- Community ePub to XTC converter based on wasm CRReader (From koreader)
  - https://x4converter.rho.sh/
  - There is no relevant or helpful documentation but it shows a potential path for rendering ePub (i.e. rust bindings to CREngine)
