#![forbid(unsafe_code)]

mod sync_benchmark;
mod tray;

#[tauri::command]
fn get_server_url(_window: tauri::Window) -> String {
    "http://localhost:9091".to_string()
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_server_url,
            open_external_url,
            sync_benchmark::benchmark_file_sync,
            sync_benchmark::benchmark_dir_scan,
            sync_benchmark::benchmark_git_status
        ])
        .setup(|app| {
            let _ = tray::setup_tray(&app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
