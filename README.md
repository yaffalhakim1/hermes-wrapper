# Hermes Wrapper

A thin desktop shell for the official Hermes Agent web dashboard. It starts the dashboard's local web server and shows it in a native WebView2 window — no agent runtime bundled, no Electron.

**Why it exists.** The official Hermes Desktop is an Electron app and heavy. The community CN Desktop is light but ships a Mandarin-only UI with no English setting. This wrapper reuses the English dashboard you already get from `hermes dashboard`, so the binary stays around 3 MB and the UI stays English.

## What it does

At launch it:

1. Runs `hermes dashboard --port 9119 --skip-build --no-open` (the `--skip-build` flag skips the one-time web UI build when `web_dist` is already present).
2. Opens `http://127.0.0.1:9119` in the main WebView2 window.
3. Hides to the system tray when the window is closed; the backend keeps running while hidden.
4. Tries to stop the spawned backend when the app exits, to release port 9119.

> Needs a local Hermes Agent install (`hermes` on PATH or the default Windows location). The dashboard web UI must have been built at least once — `hermes dashboard` builds it on first run.

## Build

Requirements: Rust (MSVC target), Node 22+, and the WebView2 runtime (built into Windows 10/11).

```bash
npm install
npm run tauri build
```

Result: `src-tauri/target/release/hermes-wrapper.exe`.

## Run

```bash
./src-tauri/target/release/hermes-wrapper.exe
```

The dashboard appears in the window. Closing the window moves the app to the tray; right-click the tray icon to quit.

## Known issues

- None currently. (Backend cleanup on exit now kills the whole `hermes.exe` process tree via `taskkill /T`.)

## Scope

This is a shell, not a fork. It does not modify the dashboard, add features to Hermes, or bundle a model runtime. Support is Windows-first (Tauri v2 + WebView2); the same approach ports to macOS/Linux by swapping WebView2 for the platform webview.

## License

MIT — see [LICENSE](LICENSE). The underlying Hermes Agent is governed by its own license.
