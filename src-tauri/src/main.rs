// Hermes Wrapper — a lightweight Tauri v2 (WebView2) wrapper around the
// official Hermes Agent web dashboard (English UI).
//
// Responsibilities:
//   1. Spawn `hermes dashboard --port 9119 --skip-build --no-open` at startup
//      (unless the port is already taken by a running instance).
//   2. Load http://127.0.0.1:9119 in the main WebView2 window.
//   3. Minimize to system tray on close; keep the backend alive while hidden.
//   4. Kill the spawned backend child process on real exit so port 9119 frees.
//   5. Gracefully handle a busy port / not-yet-ready backend.

use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

const HERMES_PORT: u16 = 9119;
const HERMES_URL: &str = "http://127.0.0.1:9119";

/// Handle to the spawned Hermes backend child process. Guarded by a Mutex so
/// we can kill it exactly once at exit.
struct Backend {
    child: Mutex<Option<Child>>,
}

/// Spawn the Hermes dashboard backend as a child process. Returns the handle
/// (kept alive by the caller) or None on failure.
fn spawn_backend() -> Option<Child> {
    // Resolve the hermes binary from the known install location; fall back to
    // a bare `hermes` on PATH if the install path is missing.
    let home = std::env::var("HERMES_HOME").unwrap_or_else(|_| {
        format!(
            "{}\\AppData\\Local\\hermes",
            std::env::var("USERPROFILE").unwrap_or_default()
        )
    });
    let exe = format!("{home}\\bin\\hermes.exe");

    let program: String = if std::path::Path::new(&exe).exists() {
        exe
    } else {
        "hermes".into()
    };

    let mut cmd = Command::new(program);
    cmd.arg("dashboard")
        .arg("--port")
        .arg(HERMES_PORT.to_string())
        .arg("--skip-build")
        .arg("--no-open")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            println!("Hermes backend spawned (pid {}).", child.id());
            Some(child)
        }
        Err(e) => {
            eprintln!("Failed to spawn Hermes backend: {e}");
            None
        }
    }
}

/// Returns true if the dashboard port is already occupied by something else.
fn port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}

/// Returns true once something is listening on the dashboard port.
fn backend_ready() -> bool {
    TcpStream::connect(("127.0.0.1", HERMES_PORT)).is_ok()
}

/// Poll the backend until it answers, then navigate the window to it.
fn wait_for_backend(app: &tauri::AppHandle, max_attempts: u32) {
    for attempt in 0..max_attempts {
        if backend_ready() {
            println!("Hermes backend is ready after {attempt} attempts.");
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.navigate(HERMES_URL.parse().unwrap());
            }
            return;
        }
        thread::sleep(Duration::from_millis(700));
    }
    eprintln!("Hermes backend did not become ready in time.");
    let _ = app.emit(
        "backend-timeout",
        "Hermes backend did not start within the expected time. Port 9119 may already be in use by another process.",
    );
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| Image::from_bytes(include_bytes!("../icons/32x32.png")).unwrap());

    TrayIconBuilder::with_id("hermes-tray")
        .icon(icon)
        .tooltip("Hermes Wrapper")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();

            // Decide whether we own the backend.
            let owns_backend = if port_in_use(HERMES_PORT) {
                eprintln!("Port {HERMES_PORT} already in use — attaching to existing backend.");
                let _ = handle.emit(
                    "backend-busy",
                    "Port 9119 is already in use; attaching to an existing Hermes dashboard.",
                );
                false
            } else {
                match spawn_backend() {
                    Some(child) => {
                        app.manage(Backend {
                            child: Mutex::new(Some(child)),
                        });
                        true
                    }
                    None => {
                        let _ = handle.emit(
                            "backend-error",
                            "Failed to start the Hermes backend. Is the Hermes CLI installed?",
                        );
                        false
                    }
                }
            };

            build_tray(app)?;

            if owns_backend {
                // Poll for readiness in a background thread; navigate when up.
                let h = handle.clone();
                thread::spawn(move || wait_for_backend(&h, 40));
            } else if backend_ready() {
                // An instance is already up — navigate immediately.
                if let Some(win) = handle.get_webview_window("main") {
                    let _ = win.navigate(HERMES_URL.parse().unwrap());
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize to tray instead of closing.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building Hermes Wrapper")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { code, .. } = event {
                if code.is_some() {
                    // Real exit — kill the backend child so the port is freed.
                    if let Some(state) = app.try_state::<Backend>() {
                        if let Ok(mut guard) = state.child.lock() {
                            if let Some(mut child) = guard.take() {
                                // ponytail: taskkill /T kills the whole tree —
                                // child.kill() alone leaves grandchildren alive on port 9119
                                let pid = child.id().to_string();
                                let _ = Command::new("taskkill")
                                    .args(["/PID", &pid, "/T", "/F"])
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .status();
                                let _ = child.wait();
                                println!("Hermes backend killed on exit.");
                            }
                        }
                    }
                }
            }
        });
}
