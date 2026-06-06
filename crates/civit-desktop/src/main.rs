#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use tauri::Manager;

mod sync_benchmark;
mod tray;

/// CLI args for auto-login: --username X --email X --display-name X
/// --server-url URL  Connect to a remote backend (skip local spawn)
struct CliArgs {
    username: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    password: Option<String>,
    server_url: Option<String>,
}

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

/// Capture the current page HTML and save it to /tmp/civit-capture.html.
/// Triggered by Ctrl+Shift+H from the injected JS keydown listener.
#[tauri::command]
fn save_page_html(html: String) -> Result<String, String> {
    let path = "/tmp/civit-capture.html";
    let mut file =
        fs::File::create(path).map_err(|e| format!("Failed to create {path}: {e}"))?;
    file.write_all(html.as_bytes())
        .map_err(|e| format!("Failed to write {path}: {e}"))?;
    eprintln!("[civit-desktop] Page HTML captured to {path} ({} bytes)", html.len());
    Ok(format!("Saved to {path} ({} bytes)", html.len()))
}

/// Read the navigation trigger file and return the URL to navigate to.
/// The test script writes /tmp/civit-navigate.txt with a URL path.
#[tauri::command]
fn read_navigate_trigger() -> Result<String, String> {
    let path = "/tmp/civit-navigate.txt";
    let url = fs::read_to_string(path).unwrap_or_default();
    // Clear the file after reading (consume the trigger)
    let _ = fs::write(path, "");
    Ok(url)
}

/// JS snippet injected into every page:
/// - Ctrl+Shift+H: capture page HTML to /tmp/civit-capture.html
fn inject_debug_js() -> &'static str {
    r#"
    (function() {
        // Ctrl+Shift+H: capture page HTML to /tmp/civit-capture.html
        document.addEventListener('keydown', function(e) {
            if (e.ctrlKey && e.shiftKey && e.key === 'H') {
                e.preventDefault();
                var html = '<!-- Captured at ' + new Date().toISOString() + ' -->\n' + document.documentElement.outerHTML;
                // Use XMLHttpRequest to POST to a tiny local endpoint — but we don't have one.
                // Instead, encode as base64 and put in document.title so Rust can read it.
                // Actually, the simplest approach: use Tauri event system.
                if (window.__TAURI_INTERNALS__) {
                    window.__TAURI_INTERNALS__.invoke('save_page_html', { html: html });
                }
            }
        });
    })();
    "#
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli = parse_args();

    // If all three auto-login fields provided, store for injection
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

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_server_url,
            open_external_url,
            save_page_html,
            read_navigate_trigger,
            sync_benchmark::benchmark_file_sync,
            sync_benchmark::benchmark_dir_scan,
            sync_benchmark::benchmark_git_status
        ])
        .setup(move |app| {
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
                                    
                                    let _ = std::fs::write("/tmp/civit-capture.html", &html);
                                    eprintln!("[capture] Saved {} bytes to /tmp/civit-capture.html", html.len());
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
                                 let (body, content_type) = match &path[..] {
                                    "/__navigate__" => {
                                        let url =
                                            std::fs::read_to_string("/tmp/civit-navigate.txt")
                                                .unwrap_or_default();
                                        let _ = std::fs::write("/tmp/civit-navigate.txt", "");
                                        (url.into_bytes(), "text/plain")
                                    }
                                    "/__capture__" => {
                                        let html = std::fs::read_to_string(
                                            "/tmp/civit-capture.html",
                                        )
                                        .unwrap_or_default();
                                        (html.into_bytes(), "text/html")
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
                                         (body, content_type)
                                     }
                                 };
                                 let resp = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\n\
                                     Content-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\
                                     Access-Control-Allow-Headers: *\r\n\r\n",
                                    body.len()
                                );
                                 let _ = stream.write_all(resp.as_bytes());
                                 let _ = stream.write_all(&body);
                                 let _ = stream.flush();
                                 } // end else (GET processing)
                             });
                        }
                    });

                    // Inject API URL override (do this BEFORE spawning threads that move window)
                let _ = window.eval(&format!(
                    "window.__CIVIT_API_URL = \"http://{}/api/v1\";",
                    server_url
                ));
                eprintln!("[civit-desktop] Injected API URL: http://{server_url}/api/v1");

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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Spawn civit-core as a child process, wait for it to be ready,
/// then redirect the main window to the server URL (same-origin).
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
                format!(
                    "{}/.local/share/civitforge/data",
                    dirs::home_dir().unwrap_or_default().display()
                )
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

fn main() {
    run();
}

fn base64_encode(data: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
}
