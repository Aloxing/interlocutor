use std::sync::Mutex;

use tauri::{Emitter, Listener, Manager, WindowEvent};

#[derive(Default)]
struct FileOpenState {
    ready: bool,
    pending_path: Option<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.len() < 2 {
                return;
            }
            let path = argv[1].clone();
            let state = app.state::<Mutex<FileOpenState>>();
            let mut file_state = state.lock().unwrap();
            if file_state.ready {
                let _ = app.emit("file-open", path);
            } else {
                file_state.pending_path = Some(path);
            }
            drop(file_state);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            read_text_file,
            save_text_file,
            save_binary_file,
            markdown_to_html,
            markdown_to_plain_text,
            open_default_apps_settings
        ])
        .manage(Mutex::new(FileOpenState::default()))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            setup_window_events(app)?;
            setup_file_open_events(app.handle().clone())?;
            ensure_window_shown(app);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn open_default_apps_settings() -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", "ms-settings:defaultapps"])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err("该功能仅在 Windows 上可用".into())
    }
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_text_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_binary_file(path: String, contents: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn markdown_to_html(source: String) -> String {
    markdown_core::Markdown::to_html(&source)
}

#[tauri::command]
fn markdown_to_plain_text(source: String) -> String {
    markdown_core::Markdown::to_plain_text(&source)
}

fn setup_window_events(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let focus_window = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::Focused(focused) = event {
                set_webview_memory_level(
                    &focus_window,
                    if *focused {
                        MemoryUsageLevel::Normal
                    } else {
                        MemoryUsageLevel::Low
                    },
                );
            }
        });
    }
    Ok(())
}

fn setup_file_open_events(
    handle: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let ready_handle = handle.clone();
    handle.listen("file-open-ready", move |_event| {
        let state = ready_handle.state::<Mutex<FileOpenState>>();
        let mut file_state = state.lock().unwrap();
        file_state.ready = true;
        if let Some(path) = file_state.pending_path.take() {
            let _ = ready_handle.emit("file-open", path);
        }
    });
    Ok(())
}

fn ensure_window_shown(app: &tauri::App) {
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        if let Some(window) = handle.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                return;
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
    });
}

#[derive(Clone, Copy)]
enum MemoryUsageLevel {
    Normal,
    Low,
}

#[cfg(windows)]
fn set_webview_memory_level(window: &tauri::WebviewWindow, level: MemoryUsageLevel) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL,
    };
    use windows_core::Interface;

    let _ = window.with_webview(move |webview| unsafe {
        if let Ok(core) = webview.controller().CoreWebView2() {
            if let Ok(webview19) = core.cast::<ICoreWebView2_19>() {
                let value = match level {
                    MemoryUsageLevel::Normal => 0,
                    MemoryUsageLevel::Low => 1,
                };
                let _ = webview19
                    .SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL(value));
            }
        }
    });
}

#[cfg(not(windows))]
fn set_webview_memory_level(_window: &tauri::WebviewWindow, _level: MemoryUsageLevel) {}
