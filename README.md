# e-inky

Desktop companion for preparing and syncing content to the Xteink X4 e-reader. The app uses a Tauri shell with a Next.js (Pages Router) frontend, keeping the stack lightweight while we build the EPUB → XTC conversion pipeline and SD card sync workflow.

## Project layout

- `src/` — Next.js (Pages Router) frontend in TypeScript.
  - `src/pages/` — entry pages (default home view only for now).
  - `src/styles/` — global styling.
- `public/` — static assets.
- `src-tauri/` — Tauri Rust project, config, and build scripts.
- `next.config.js` — Next.js config with static export enabled for bundling into Tauri.

## Scripts

- `pnpm dev` — run the Next.js dev server on port 3000.
- `pnpm build` — build the static frontend (`out/`) for bundling with Tauri.
- `pnpm start` — serve the built frontend (useful for validating exports).
- `pnpm lint` — run ESLint using Next.js defaults.
- `pnpm tauri:dev` — start Tauri with the Next dev server (hot reload in the desktop shell).
- `pnpm tauri:build` — build a packaged desktop app (runs `pnpm build` first via Tauri config).

## Tauri build configuration

- Development: `devPath` is `http://localhost:3000`, and Tauri runs `pnpm dev` before launching.
- Production: `distDir` is `../out` (static export from Next.js). Tauri runs `pnpm build` before bundling.
- Window defaults: 420×720 with sensible minimum sizes for an e-ink-friendly layout.

## Getting started

1. Install dependencies: `pnpm install`.
2. Run the web UI: `pnpm dev`.
3. Launch desktop shell: `pnpm tauri:dev` (starts Next.js then wraps it in Tauri).
4. Package the app: `pnpm tauri:build` (exports the static site and bundles with Tauri).

## Next steps

- Implement the EPUB import + conversion pipeline in Rust crates (see `AGENTS.md` for expectations).
- Define UI flows for library management and sync planning.
- Add tests for XTC/XTG/XTH handling and sync logic as the pipeline solidifies.
