#![deny(unsafe_code)]

use std::fs;
use std::io::Write;
use tauri::Manager;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod sync_benchmark;
mod tray;

/// Return a platform-appropriate temporary directory for CivitForge captures.
fn civit_tmp_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::temp_dir().join("civitforge")
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::path::PathBuf::from("/tmp")
    }
}

/// Ensure the CivitForge temp directory exists.
fn ensure_civit_tmp() -> std::path::PathBuf {
    let dir = civit_tmp_dir();
    let _ = fs::create_dir_all(&dir);
    dir
}

/// CLI args for auto-login: --username X --email X --display-name X
/// --server-url URL  Connect to a remote backend (skip local spawn)
#[cfg(not(any(target_os = "android", target_os = "ios")))]
struct CliArgs {
    username: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    password: Option<String>,
    server_url: Option<String>,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut cli = CliArgs {
        username: None,
        email: None,
        display_name: None,
        password: None,
        server_url: None,
    };
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" if i + 1 < args.len() => {
                cli.username = Some(args[i + 1].clone());
                i += 2;
            }
            "--email" | "-e" if i + 1 < args.len() => {
                cli.email = Some(args[i + 1].clone());
                i += 2;
            }
            "--display-name" | "-d" | "--display_name" if i + 1 < args.len() => {
                cli.display_name = Some(args[i + 1].clone());
                i += 2;
            }
            "--password" | "-p" if i + 1 < args.len() => {
                cli.password = Some(args[i + 1].clone());
                i += 2;
            }
            "--server-url" | "-s" if i + 1 < args.len() => {
                cli.server_url = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    cli
}

#[tauri::command]
fn get_server_url(_window: tauri::Window) -> String {
    "http://localhost:9091".to_string()
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

/// Capture the current page HTML and save it to <tmpdir>/civit-capture.html.
/// Triggered by Ctrl+Shift+H from the injected JS keydown listener.
#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn save_page_html(html: String) -> Result<String, String> {
    let dir = ensure_civit_tmp();
    let path = dir.join("civit-capture.html");
    let path_str = path.to_string_lossy().to_string();
    let mut file =
        fs::File::create(&path).map_err(|e| format!("Failed to create {path_str}: {e}"))?;
    file.write_all(html.as_bytes())
        .map_err(|e| format!("Failed to write {path_str}: {e}"))?;
    eprintln!("[civit-desktop] Page HTML captured to {path_str} ({} bytes)", html.len());
    Ok(format!("Saved to {path_str} ({} bytes)", html.len()))
}

/// Take a screenshot of the entire display and save to <tmpdir>/civit-screenshot-<timestamp>.png.
/// Triggered by Ctrl+Shift+S from the injected JS keydown listener.
/// Also saves a copy to <tmpdir>/civit-screenshot-latest.png for easy reference.
/// Falls back gracefully on each platform when no screenshot tool is available.
#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn take_screenshot() -> Result<String, String> {
    let dir = ensure_civit_tmp();
    let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let filename = dir.join(format!("civit-screenshot-{ts}.png"));
    let latest = dir.join("civit-screenshot-latest.png");
    let filename_str = filename.to_string_lossy().to_string();

    // Try platform-specific screenshot tools in order of preference
    // Wayland: grim → X11: maim/scrot/import → Windows: nircmd/Powershell → macOS: screencapture
    #[cfg(not(target_os = "windows"))]
    let commands: &[(&str, &[&str])] = &[
        // Wayland native
        ("grim", &[filename_str.as_str()]),
        // X11 tools
        ("maim", &[filename_str.as_str()]),
        ("scrot", &[filename_str.as_str()]),
        ("import", &["-window", "root", filename_str.as_str()]),
    ];

    #[cfg(target_os = "windows")]
    let commands: &[(&str, &[&str])] = &[
        // nircmd: lightweight Windows CLI utility for screenshots
        ("nircmd", &["screenshot", "full", filename_str.as_str()]),
    ];

    for (cmd, args) in commands {
        if let Ok(output) = std::process::Command::new(cmd).args(*args).output() {
            if output.status.success() {
                let _ = std::fs::copy(&filename, &latest);
                let size = std::fs::metadata(&filename)
                    .map(|m| m.len())
                    .unwrap_or(0);
                eprintln!(
                    "[civit-desktop] Screenshot captured to {filename_str} ({size} bytes) via {cmd}"
                );
                return Ok(format!(
                    "Screenshot saved to {filename_str} ({size} bytes) via {cmd}"
                ));
            }
        }
    }

    // Windows fallback: use PowerShell to capture via .NET
    #[cfg(target_os = "windows")]
    {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bmp)
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$bmp.Save('{filename_str}')
$graphics.Dispose()
$bmp.Dispose()
Write-Output 'ok'
"#,
            filename_str = filename_str.replace('\'', "''")
        );
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
            .output()
        {
            if output.status.success() {
                let _ = std::fs::copy(&filename, &latest);
                let size = std::fs::metadata(&filename)
                    .map(|m| m.len())
                    .unwrap_or(0);
                eprintln!(
                    "[civit-desktop] Screenshot captured to {filename_str} ({size} bytes) via PowerShell"
                );
                return Ok(format!(
                    "Screenshot saved to {filename_str} ({size} bytes) via PowerShell"
                ));
            }
        }
    }

    // No tool found — provide helpful install instructions
    #[cfg(target_os = "windows")]
    eprintln!(
        "[civit-desktop] No screenshot tool found. Install nircmd or ensure PowerShell is available."
    );
    #[cfg(not(target_os = "windows"))]
    eprintln!(
        "[civit-desktop] No screenshot tool found. Install grim (Wayland) or maim/scrot (X11)."
    );

    #[cfg(target_os = "windows")]
    {
        Err(format!(
            "No screenshot tool found. Install nircmd, or ensure PowerShell has System.Windows.Forms available. \
             Screenshot file would be: {filename_str}"
        ))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(format!(
            "No screenshot tool found. Install one of: grim (Wayland), maim, scrot (X11), import (ImageMagick). \
             Screenshot file would be: {filename_str}"
        ))
    }
}

/// Check if a screenshot tool is available and return its name.
#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn screenshot_tool_available() -> String {
    #[cfg(target_os = "windows")]
    let tools = ["nircmd"];
    #[cfg(not(target_os = "windows"))]
    let tools = ["grim", "maim", "scrot", "import"];

    for cmd in &tools {
        if which_tool(cmd) {
            return cmd.to_string();
        }
    }

    // On Windows, PowerShell is always available as a fallback
    #[cfg(target_os = "windows")]
    {
        return "powershell".to_string();
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::new()
    }
}

/// Check if a command-line tool exists on PATH.
fn which_tool(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Read the navigation trigger file and return the URL to navigate to.
/// The test script writes <tmpdir>/civit-navigate.txt with a URL path.
#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_navigate_trigger() -> Result<String, String> {
    let dir = civit_tmp_dir();
    let path = dir.join("civit-navigate.txt");
    let url = fs::read_to_string(&path).unwrap_or_default();
    // Clear the file after reading (consume the trigger)
    let _ = fs::write(&path, "");
    Ok(url)
}

/// JS snippet injected into every page:
/// - Ctrl+Shift+H: capture page HTML to /tmp/civit-capture.html
/// - Ctrl+Shift+S: take screenshot to /tmp/civit-screenshot-<timestamp>.png
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn inject_debug_js() -> &'static str {
    r#"
    (function() {
        // Ctrl+Shift+H: capture page HTML to /tmp/civit-capture.html
        // Ctrl+Shift+S: take screenshot to /tmp/civit-screenshot-*.png
        document.addEventListener('keydown', function(e) {
            if (e.ctrlKey && e.shiftKey && (e.key === 'H' || e.key === 'S')) {
                e.preventDefault();
                if (e.key === 'H') {
                    var html = '<!-- Captured at ' + new Date().toISOString() + ' -->\n' + document.documentElement.outerHTML;
                    if (window.__TAURI_INTERNALS__) {
                        window.__TAURI_INTERNALS__.invoke('save_page_html', { html: html });
                    }
                } else if (e.key === 'S') {
                    if (window.__TAURI_INTERNALS__) {
                        window.__TAURI_INTERNALS__.invoke('take_screenshot');
                    }
                }
            }
        });
    })();
    "#
}

#[cfg(any(target_os = "android", target_os = "ios"))]
struct MobileApiState {
    api_url: String,
}

/// Mobile initialization: connect to remote API
#[cfg(any(target_os = "android", target_os = "ios"))]
fn mobile_init(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let api_url = std::env::var("CIVIT_API_URL")
        .unwrap_or_else(|_| "https://api.civitforge.com".into());
    app.manage(MobileApiState { api_url });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let cli = parse_args();

    // If all three auto-login fields provided, store for injection
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let auto_login_json = if cli.username.is_some() && cli.password.is_some() {
        serde_json::json!({
            "username": cli.username,
            "email": cli.email,
            "display_name": cli.display_name,
            "password": cli.password
        })
        .to_string()
    } else {
        String::new()
    };

    let mut builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_fs::init());
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        builder = builder
            .plugin(tauri_plugin_deep_link::init())
            .plugin(tauri_plugin_notification::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            get_server_url,
            open_external_url,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            save_page_html,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            take_screenshot,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            screenshot_tool_available,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            read_navigate_trigger,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            sync_benchmark::benchmark_file_sync,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            sync_benchmark::benchmark_dir_scan,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            sync_benchmark::benchmark_git_status
        ])
        .setup(move |app| {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let _ = tray::setup_tray(&app.handle());

                // Inject debug JS (Ctrl+Shift+H capture) after short delay
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if let Some(w) = handle.get_webview_window("main") {
                        let _ = w.eval(inject_debug_js());
                        eprintln!("[civit-desktop] Debug JS injected (Ctrl+Shift+H to capture HTML)");
                    }
                });

                let remote_url = cli.server_url.clone();
                let auto_login = auto_login_json.clone();

                if let Some(server_url) = remote_url {
                    // Remote backend mode: serve WASM locally via HTTP, proxy API to remote
                    eprintln!("[civit-desktop] Using remote server: {server_url}");
                    let window = app
                        .get_webview_window("main")
                        .expect("main window not found");

                    // Start a local static file server so WASM runs on http://127.0.0.1
                    // (avoid tauri:// cross-origin issues with reqwest)
                    let dist_dir = std::env::current_exe()
                        .ok()
                        .and_then(|p| p.parent().map(|d| d.join("dist")))
                        .or_else(|| {
                            // Dev fallback: look for dist next to workspace Cargo.toml
                            std::env::var("CARGO_MANIFEST_DIR")
                                .ok()
                                .map(|d| std::path::PathBuf::from(d).join("../civit-ui/dist"))
                        })
                        .filter(|d| d.exists());

                    if let Some(dist) = dist_dir {
                        let serve_port: u16 = 9092;
                        let dist_clone = dist.clone();
                        std::thread::spawn(move || {
                            // Wait for window to be ready
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            let addr = std::net::SocketAddr::from((
                                [127, 0, 0, 1], serve_port,
                            ));
                            let listener = std::net::TcpListener::bind(addr)
                                .expect("Failed to bind local file server");
                            eprintln!(
                                "[civit-desktop] Serving WASM dist on http://127.0.0.1:{serve_port} from {}",
                                dist.display()
                            );
                            for stream in listener.incoming().flatten() {
                                let dist_inner = dist_clone.clone();
                                std::thread::spawn(move || {
                                    use std::io::{Read, Write};
                                    let mut stream = stream;
                                    let mut buf = [0u8; 65536];
                                    let n = stream.read(&mut buf).unwrap_or(0);
                                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let method = if req.starts_with("POST ") { "POST" } else { "GET" };
                                    let path = req
                                        .lines()
                                        .find(|l| l.starts_with(method))
                                        .and_then(|l| {
                                            l.strip_prefix(method)
                                                .map(|s| s.split_whitespace().next().unwrap_or("/"))
                                        })
                                        .unwrap_or("/")
                                        .to_string();

                                    // Handle POST /__capture__ (body = page HTML)
                                    let handled = if method == "POST" && path == "/__capture__" {
                                        // Parse Content-Length header
                                        let content_length: usize = req
                                            .lines()
                                            .find(|l| l.to_lowercase().starts_with("content-length:"))
                                            .and_then(|l| l.split(':').nth(1))
                                            .and_then(|v| v.trim().parse().ok())
                                            .unwrap_or(0);
                                        let header_end = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
                                        let body_in_buf = if header_end <= n { req[header_end..].to_string() } else { String::new() };

                                        // Read more data if body is incomplete
                                        let mut html = body_in_buf;
                                        if html.len() < content_length {
                                            let mut remaining = vec![0u8; content_length - html.len()];
                                            let mut total_read = 0;
                                            while total_read < remaining.len() {
                                                let r = stream.read(&mut remaining[total_read..]).unwrap_or(0);
                                                if r == 0 { break; }
                                                total_read += r;
                                            }
                                            html.push_str(&String::from_utf8_lossy(&remaining[..total_read]));
                                        }

                                        let capture_path = civit_tmp_dir().join("civit-capture.html");
                                        let _ = std::fs::write(&capture_path, &html);
                                        eprintln!("[capture] Saved {} bytes to {}", html.len(), capture_path.display());
                                        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nAccess-Control-Allow-Origin: *\r\n\r\nOK";
                                        let _ = stream.write_all(resp.as_bytes());
                                        let _ = stream.flush();
                                        true
                                    } else {
                                        false
                                    };

                                    if handled {
                                        // POST handled, skip GET processing
                                    } else {
                                     // GET endpoints
                                     let (body, content_type, status, redirect) = match &path[..] {
                                        "/__navigate__" => {
                                            let nav_path = civit_tmp_dir().join("civit-navigate.txt");
                                            let url =
                                                std::fs::read_to_string(&nav_path)
                                                    .unwrap_or_default();
                                            let _ = std::fs::write(&nav_path, "");
                                            (url.into_bytes(), "text/plain", 200, None)
                                        }
                                        "/__capture__" => {
                                            let cap_path = civit_tmp_dir().join("civit-capture.html");
                                            let html = std::fs::read_to_string(&cap_path)
                                                .unwrap_or_default();
                                            (html.into_bytes(), "text/html", 200, None)
                                        }
                                        "/__logout__" => {
                                            // Redirect to login page after logout
                                            (Vec::new(), "text/html", 302, Some("/login".to_string()))
                                        }
                                        _ => {
                                            // SPA fallback: serve index.html for all non-file paths
                                            let file_path = if path == "/" {
                                                dist_inner.join("index.html")
                                            } else {
                                                let candidate = dist_inner.join(path.trim_start_matches('/'));
                                                if candidate.is_file() {
                                                    candidate
                                                } else {
                                                    dist_inner.join("index.html")
                                                }
                                            };
                                            let body =
                                                std::fs::read(&file_path).unwrap_or_default();
                                            let ext = file_path
                                                .extension()
                                                .and_then(|e| e.to_str())
                                                .unwrap_or("");
                                            let content_type = match ext {
                                                "html" => "text/html",
                                                "js" => "application/javascript",
                                                "wasm" => "application/wasm",
                                                "css" => "text/css",
                                                "json" => "application/json",
                                                "svg" => "image/svg+xml",
                                                "ico" => "image/x-icon",
                                                _ => "application/octet-stream",
                                            };
                                              (body, content_type, 200, None)
                                          }
                                      };
                                      let status_line = if status == 302 {
                                          format!("HTTP/1.1 302 Found\r\nLocation: {}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n", redirect.as_deref().unwrap_or("/"))
                                      } else {
                                          format!(
                                             "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\n\
                                              Content-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\
                                              Access-Control-Allow-Headers: *\r\n\r\n",
                                             body.len()
                                         )
                                      };
                                      let _ = stream.write_all(status_line.as_bytes());
                                      if status != 302 {
                                          let _ = stream.write_all(&body);
                                      }
                                      let _ = stream.flush();
                                     } // end else (GET processing)
                                 });
                            }
                        });

                        // Inject API URL override (do this BEFORE spawning threads that move window)
                        // server_url may already include scheme (http://) from CLI arg
                        let api_url = if server_url.starts_with("http://") || server_url.starts_with("https://") {
                            format!("{server_url}/api/v1")
                        } else {
                            format!("http://{server_url}/api/v1")
                        };
                    let _ = window.eval(&format!(
                        "window.__CIVIT_API_URL = \"{api_url}\";",
                    ));
                    eprintln!("[civit-desktop] Injected API URL: {api_url}");

                    // Navigate to local WASM server in background
                    let nav_window = window.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        let _ = nav_window.eval(
                            "window.location.href = 'http://127.0.0.1:9092/';"
                        );
                        eprintln!("[civit-desktop] Navigated to local WASM server");
                    });
                }
                    if !auto_login.is_empty() {
                        let encoded = base64_encode(&auto_login);
                        let nav = format!("tauri://localhost/login#auto={encoded}");
                        let _ = window.eval(&format!("window.location.href = '{nav}';"));
                    }
                } else {
                    // Local embedded server mode
                    let handle = app.handle().clone();
                    let auto_login = auto_login_json.clone();
                    std::thread::spawn(move || {
                        spawn_embedded_server(&handle, &auto_login);
                    });
                }
            }

            #[cfg(any(target_os = "android", target_os = "ios"))]
            {
                mobile_init(app)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Spawn civit-core as a child process, wait for it to be ready,
/// then redirect the main window to the server URL (same-origin).
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn spawn_embedded_server(app: &tauri::AppHandle, auto_login_json: &str) {
    let port = std::env::var("CIVIT_PORT").unwrap_or_else(|_| "9091".to_string());

    // Find the civit-core binary next to the Tauri binary
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    let server_bin = exe_dir.as_ref().and_then(|dir| {
        ["civit-core", "civit-core.exe"]
            .into_iter()
            .map(|name| dir.join(name))
            .find(|p| p.exists())
    });

    let mut server_cmd = match server_bin {
        Some(bin) => std::process::Command::new(bin),
        None => {
            // Fall back to cargo run for development
            let mut cmd = std::process::Command::new("cargo");
            cmd.args(["run", "--bin", "civit-core"]);
            cmd
        }
    };

    server_cmd
        .env("CIVIT_PORT", &port)
        .env(
            "DATABASE_URL",
            std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://civit:civit@127.0.0.1:15432/civit?sslmode=disable".to_string()
            }),
        )
        .env(
            "REDIS_URL",
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6380".to_string()),
        )
        .env(
            "JWT_SECRET",
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-test-secret-key-32bytes-minimum".to_string()),
        )
        .env(
            "CIVIT_STORAGE_PATH",
            std::env::var("CIVIT_STORAGE_PATH").unwrap_or_else(|_| {
                let home = dirs::home_dir().unwrap_or_default();
                home.join(".local/share/civitforge/data")
                    .to_string_lossy()
                    .to_string()
            }),
        )
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn civit-core server");

    // Wait for server to be ready
    let server_url = format!("http://127.0.0.1:{port}");
    for _ in 0..60 {
        if reqwest::blocking::get(&format!("{server_url}/healthz"))
            .is_ok_and(|r| r.status().is_success())
        {
            eprintln!("[civit-desktop] Server ready at {server_url}");
            // Redirect main window to server URL (same-origin, no CORS issues)
            if let Some(window) = app.get_webview_window("main") {
                // Navigate with credentials in hash fragment (survives cross-origin)
                let nav_url = if !auto_login_json.is_empty() {
                    let encoded = base64_encode(&auto_login_json);
                    format!("{server_url}/login#auto={encoded}")
                } else {
                    server_url.clone()
                };
                let _ = window.eval(&format!("window.location.href = '{nav_url}';"));
                if !auto_login_json.is_empty() {
                    eprintln!("[civit-desktop] Auto-login via hash fragment");
                }
            }
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    eprintln!("[civit-desktop] Warning: server may not be ready");
}

#[allow(unsafe_code)]
fn main() {
    // Tachyon-style Wayland/X11 fix for WebKitGTK compatibility.
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    if is_wayland {
        // SAFETY: called at process start before any threads are spawned.
        unsafe { std::env::set_var("GDK_BACKEND", "x11"); }
    }

    let is_nvidia = std::fs::read_to_string("/proc/modules")
        .map(|m| m.contains("nvidia_drm") || m.contains("nvidia"))
        .unwrap_or(false);

    if is_nvidia {
        // SAFETY: called at process start before any threads are spawned.
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1"); }
        // Force software GL rendering to avoid GBM/dri crashes on Wayland+NVIDIA
        if is_wayland {
            unsafe { std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1"); }
        }
    }

    run();
}

fn base64_encode(data: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
}
