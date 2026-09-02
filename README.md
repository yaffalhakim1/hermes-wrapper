# Hermes Wrapper

A **lightweight desktop wrapper** for the official [Hermes Agent](https://github.com/NousResearch/hermes-agent) web dashboard.

Built with **Tauri v2 + WebView2 (Rust)** instead of Electron — the bundle is ~3 MB, not 200 MB+. The UI is the **official English dashboard** served locally by `hermes dashboard`; this app only embeds it in a native window.

## Why

- The official Hermes Desktop (Electron) is heavy.
- Community CN Desktop (Tauri) is light but ships a Mandarin-only UI with no English option.
- This wrapper reuses the official dashboard you already have installed (`hermes dashboard --port 9119`), so it stays small and stays English.

## How it works

On launch the wrapper:

1. Spawns `hermes dashboard --port 9119 --skip-build --no-open` (skips the one-time web UI build if `web_dist` already exists).
2. Loads `http://127.0.0.1:9119` in the main WebView2 window.
3. Minimizes to the **system tray** on close (backend keeps running while hidden).
4. Kills the spawned backend child process on real exit so port 9119 is freed.

> Requires a local Hermes Agent install (`hermes` on PATH or at the default Windows location). The dashboard web UI must have been built at least once (`hermes dashboard` builds it on first run).

## Build

Prerequisites: Rust (MSVC target), Node 22+, WebView2 runtime (preinstalled on Windows 10/11).

```bash
npm install
npm run tauri build
```

Output: `src-tauri/target/release/hermes-wrapper.exe`.

## Run

```bash
./src-tauri/target/release/hermes-wrapper.exe
```

The dashboard opens inside the window. Closing the window hides it to the tray; right-click the tray icon to quit (this also stops the backend).

## Platform

Windows-first (Tauri v2, WebView2). The approach generalizes to macOS/Linux where WebView2 is replaced by the platform webview.

## License

See the Hermes Agent license for the underlying agent. This wrapper code is provided as-is.
