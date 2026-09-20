# Dependency upgrade — 2026-09-20

This is the historical record for commit `0597adb`. The subsequent
[code-quality migration](code-quality.md) replaced ESLint with Oxlint/Oxfmt and
removed the TypeScript 6 compatibility package. Use that guide for current lint
and formatting commands.

## Versions

Direct dependencies were checked against npm's `latest` tag and crates.io's
`max_stable_version`. Prereleases were excluded. Both lockfiles were regenerated;
transitive dependencies remain subject to their upstream version constraints.

| Dependency | Before | After |
| --- | --- | --- |
| Excalidraw | 0.17.6 | 0.18.1 |
| React / React DOM | 18.3.1 | 19.3.0 |
| Tauri JS API | 2.1.1 | 2.11.1 |
| Tauri CLI | 2.1.0 | 2.11.5 |
| Tauri Rust | 2.1.1 in lockfile | 2.11.6 |
| Vite | 6.0.2 in lockfile | 8.3.0 |
| Vite React plugin | 4.3.4 | 6.1.1 |
| TypeScript compiler | 5.7.2 | 7.0.2 |
| ESLint | 9.16.0 | 10.11.0 |
| Antfu ESLint config | 3.11.2 | 9.5.1 |
| xcap | 0.0.14 | 0.9.8 |
| image | 0.25.5 | 0.25.10 |
| cocoa | 0.25.0 | 0.27.0 |
| strum / strum_macros | 0.26.x | 0.28.0 |
| base64 | 0.22.1 | 0.23.1 |

See `package.json` and `src-tauri/Cargo.toml` for all direct dependency versions.
`scrap` remains at 0.5.0, its latest stable release at the time of the upgrade.

## Compatibility decisions

- TypeScript 7 runs as `tsc` through the `@typescript/native` npm alias.
  The `typescript` dependency aliases the official `@typescript/typescript6`
  compatibility package because typescript-eslint still needs the TypeScript 6
  JavaScript API. This follows Microsoft's
  [side-by-side installation instructions](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/#running-side-by-side-with-typescript-6.0).
- Excalidraw pins Radix Tabs 1.0.2, which does not declare React 19 support.
  A scoped pnpm override selects 1.1.21, which supports React 19. Remove the
  override when Excalidraw updates that dependency.
- Excalidraw's stylesheet is explicitly imported. Vite copies the bundled fonts
  into the generated `public/excalidraw/fonts` directory for development and
  production. The app uses that local asset path; CSP allows same-origin fonts
  and font fetching. See the
  [Excalidraw installation guide](https://docs.excalidraw.com/docs/@excalidraw/excalidraw/installation).
- TypeScript path aliases use relative paths without the removed `baseUrl`
  option. The Vite config uses `import.meta.dirname`.
- xcap metadata accessors now return `Result`; required values propagate errors
  through the existing command result. The tray uses Tauri's renamed
  `show_menu_on_left_click` builder method.

## Development and validation

Use Node.js 24 or newer, pnpm 9.15.0 (the package manager used for this lockfile),
and a current stable Rust toolchain. This upgrade was checked with Rust 1.90.0
on Apple Silicon macOS.

```sh
pnpm install --frozen-lockfile --strict-peer-dependencies
pnpm exec eslint .
pnpm build
cargo check --locked --manifest-path src-tauri/Cargo.toml
pnpm tauri build --debug --bundles app
```

`pnpm lint` retains its existing auto-fix behavior. Use `pnpm exec eslint .` for
a read-only check.

Verified: frozen installation with strict peer dependencies, ESLint, frontend
production build, the Vite config's TypeScript check, and `cargo check --locked`.
Browser smoke checks covered rectangle drawing, undo/redo, Chinese text, and
image export preview, with no console warnings or errors observed.
Chinese text and export preview were also checked with the app's CSP applied
as an HTTP response header; local fonts loaded without CSP errors.

Full macOS app packaging is **not verified**: both the normal debug build and a
retry with debug symbols and incremental compilation disabled ran out of disk
space. The generated Cargo target directory was cleaned afterwards. Native
tray, shortcut, capture-permission, and clipboard interactions still need a
manual regression pass after a successful package build. No native app was
launched during this validation.

The existing Excalidraw board, menus, tray actions, shortcuts, screenshot
implementations, and window behavior are retained. Cocoa deprecation warnings
and existing unused Rust code remain; replacing the native window layer belongs
to a separate change. Windows/Linux adapters were already incomplete and are
not made production-ready by this dependency upgrade.
