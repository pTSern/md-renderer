#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use wry::{WebView, WebViewBuilder};

const APP_HTML: &str = include_str!("../assets/app.html");
const SAMPLE_MD: &str = include_str!("../assets/sample.md");
const DEFAULT_KEYBINDINGS: &str = include_str!("../keybindings.json");
const LOGO_RGBA: &[u8] = include_bytes!("../assets/logo.rgba");

#[derive(Debug, Deserialize)]
struct IpcRequest {
    cmd: String,
    path: Option<String>,
    content: Option<String>,
    #[serde(rename = "defaultName")]
    default_name: Option<String>,
    title: Option<String>,
    json: Option<String>,
}

#[derive(Debug, Serialize)]
struct IpcResponse<'a> {
    #[serde(rename = "type")]
    response_type: &'a str,
    path: Option<String>,
    name: Option<String>,
    content: Option<String>,
    message: Option<String>,
    json: Option<String>,
}

fn send_to_webview(webview: &WebView, response: &IpcResponse) {
    if let Ok(json_str) = serde_json::to_string(response) {
        let script = format!("window.receiveFromRust({});", json_str);
        let _ = webview.evaluate_script(&script);
    }
}

fn validate_or_fallback_keybindings(read_res: Result<String, std::io::Error>) -> (String, Option<String>) {
    match read_res {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => (content, None),
                Err(e) => {
                    let msg = format!("'keybindings.json' is corrupted ({}). Reverted to default settings.", e);
                    eprintln!("[WARN] {}", msg);
                    (DEFAULT_KEYBINDINGS.to_string(), Some(msg))
                }
            }
        }
        Err(e) => {
            let msg = format!("Failed to read 'keybindings.json' ({}). Reverted to default settings.", e);
            eprintln!("[WARN] {}", msg);
            (DEFAULT_KEYBINDINGS.to_string(), Some(msg))
        }
    }
}

fn get_keybindings_content() -> (String, Option<String>) {
    let keybindings_path = Path::new("keybindings.json");
    if keybindings_path.exists() {
        validate_or_fallback_keybindings(fs::read_to_string(keybindings_path))
    } else {
        (DEFAULT_KEYBINDINGS.to_string(), None)
    }
}

const PIPE_NAME: &str = r"\\.\pipe\mdviewer_single_instance_ipc";

const PIPE_ACCESS_DUPLEX: u32 = 0x00000003;
const PIPE_TYPE_BYTE: u32 = 0x00000000;
const PIPE_READMODE_BYTE: u32 = 0x00000000;
const PIPE_WAIT: u32 = 0x00000000;
const INVALID_HANDLE_VALUE: isize = -1;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateNamedPipeW(
        lpName: *const u16,
        dwOpenMode: u32,
        dwPipeMode: u32,
        nMaxInstances: u32,
        nOutBufferSize: u32,
        nInBufferSize: u32,
        nDefaultTimeOut: u32,
        lpSecurityAttributes: *mut std::ffi::c_void,
    ) -> isize;
    fn ConnectNamedPipe(hNamedPipe: isize, lpOverlapped: *mut std::ffi::c_void) -> i32;
    fn DisconnectNamedPipe(hNamedPipe: isize) -> i32;
    fn CloseHandle(hObject: isize) -> i32;
    fn GetLastError() -> u32;
}

#[derive(Debug)]
enum AppEvent {
    OpenFile(PathBuf),
    FocusWindow,
}

fn resolve_target_path(p: &Path) -> String {
    if let Ok(canon) = fs::canonicalize(p) {
        let s = canon.to_string_lossy().to_string();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            stripped.to_string()
        } else {
            s
        }
    } else if let Ok(abs) = std::env::current_dir().map(|cwd| cwd.join(p)) {
        abs.to_string_lossy().to_string()
    } else {
        p.to_string_lossy().to_string()
    }
}

fn try_send_to_existing_instance(path: Option<&Path>) -> bool {
    use std::io::{Read, Write};

    let mut client_file = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(PIPE_NAME)
    {
        Ok(f) => f,
        Err(_) => return false,
    };

    let payload = match path {
        Some(p) => resolve_target_path(p),
        None => String::new(),
    };

    if client_file.write_all(format!("{}\n", payload).as_bytes()).is_ok() {
        let _ = client_file.flush();
        let mut ack = [0u8; 3];
        let _ = client_file.read_exact(&mut ack);
        return true;
    }

    false
}

fn run_ipc_server(proxy: EventLoopProxy<AppEvent>) {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{FromRawHandle, IntoRawHandle};
    use std::io::{BufRead, BufReader, Write};

    let pipe_wide: Vec<u16> = std::ffi::OsStr::new(PIPE_NAME)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    loop {
        let pipe_handle = unsafe {
            CreateNamedPipeW(
                pipe_wide.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                255,
                4096,
                4096,
                5000,
                std::ptr::null_mut(),
            )
        };

        if pipe_handle == INVALID_HANDLE_VALUE {
            std::thread::sleep(std::time::Duration::from_millis(200));
            continue;
        }

        let connected = unsafe { ConnectNamedPipe(pipe_handle, std::ptr::null_mut()) };
        if connected != 0 || unsafe { GetLastError() } == 535 {
            let mut file = unsafe { std::fs::File::from_raw_handle(pipe_handle as _) };
            let mut line = String::new();
            {
                let mut reader = BufReader::new(&mut file);
                let _ = reader.read_line(&mut line);
            }
            let _ = file.write_all(b"OK\n");
            let _ = file.flush();

            let trimmed = line.trim();
            if trimmed.is_empty() {
                let _ = proxy.send_event(AppEvent::FocusWindow);
            } else {
                let path = PathBuf::from(trimmed);
                let _ = proxy.send_event(AppEvent::OpenFile(path));
            }

            let _ = file.into_raw_handle();
            unsafe {
                DisconnectNamedPipe(pipe_handle);
                CloseHandle(pipe_handle);
            }
        } else {
            unsafe {
                CloseHandle(pipe_handle);
            }
        }
    }
}

fn main() {
    let initial_path_arg = std::env::args().nth(1).map(PathBuf::from);

    // If an instance is already running, pass arguments to it and exit immediately
    if try_send_to_existing_instance(initial_path_arg.as_deref()) {
        return;
    }

    let event_loop: EventLoop<AppEvent> = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    std::thread::spawn(move || {
        run_ipc_server(proxy);
    });

    let icon = tao::window::Icon::from_rgba(LOGO_RGBA.to_vec(), 128, 128).ok();

    let mut window_builder = WindowBuilder::new()
        .with_title("MDViewer")
        .with_decorations(false) // Borderless window (custom title bar controls)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(960.0, 840.0)) // Half-screen target size
        .with_min_inner_size(LogicalSize::new(460.0, 360.0));

    if let Some(ic) = icon {
        window_builder = window_builder.with_window_icon(Some(ic));
    }

    let window = window_builder
        .build(&event_loop)
        .expect("Failed to create application window");

    let window_arc = Arc::new(window);
    let window_clone_ipc = window_arc.clone();

    let webview_holder: Arc<Mutex<Option<WebView>>> = Arc::new(Mutex::new(None));
    let webview_for_ipc = webview_holder.clone();

    let initial_path_for_ipc = initial_path_arg.clone();

    let webview = WebViewBuilder::new()
        .with_html(APP_HTML)
        .with_ipc_handler(move |req| {
            let body = req.body();
            if let Ok(request) = serde_json::from_str::<IpcRequest>(body) {
                let holder = webview_for_ipc.lock().unwrap();
                if let Some(wv) = holder.as_ref() {
                    match request.cmd.as_str() {
                        "drag_window" => {
                            let _ = window_clone_ipc.drag_window();
                        }

                        "minimize_window" => {
                            window_clone_ipc.set_minimized(true);
                        }

                        "maximize_window" => {
                            window_clone_ipc.set_maximized(!window_clone_ipc.is_maximized());
                        }

                        "close_window" => {
                            std::process::exit(0);
                        }

                        "get_init_data" => {
                            let (kb_json, kb_err) = get_keybindings_content();
                            if let Some(ref path_buf) = initial_path_for_ipc {
                                if path_buf.exists() {
                                    if let Ok(content) = fs::read_to_string(path_buf) {
                                        let name = path_buf
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("Untitled.md")
                                            .to_string();
                                        send_to_webview(
                                            wv,
                                            &IpcResponse {
                                                response_type: "init",
                                                path: Some(path_buf.to_string_lossy().to_string()),
                                                name: Some(name),
                                                content: Some(content),
                                                message: kb_err,
                                                json: Some(kb_json),
                                            },
                                        );
                                        return;
                                    }
                                }
                            }

                            // If no file provided or file read failed, load sample
                            send_to_webview(
                                wv,
                                &IpcResponse {
                                    response_type: "init",
                                    path: None,
                                    name: Some("Welcome.md".to_string()),
                                    content: Some(SAMPLE_MD.to_string()),
                                    message: kb_err,
                                    json: Some(kb_json),
                                },
                            );
                        }

                        "read_file" => {
                            if let Some(path_str) = request.path {
                                let path = PathBuf::from(&path_str);
                                if path.exists() {
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        let name = path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("Untitled.md")
                                            .to_string();
                                        send_to_webview(
                                            wv,
                                            &IpcResponse {
                                                response_type: "file_opened",
                                                path: Some(path.to_string_lossy().to_string()),
                                                name: Some(name),
                                                content: Some(content),
                                                message: None,
                                                json: None,
                                            },
                                        );
                                        return;
                                    }
                                }
                            }
                        }

                        "open_file" => {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Markdown Files (*.md, *.markdown)", &["md", "markdown"])
                                .add_filter("Text Files (*.txt)", &["txt"])
                                .add_filter("All Files (*.*)", &["*"])
                                .pick_file()
                            {
                                match fs::read_to_string(&path) {
                                    Ok(content) => {
                                        let name = path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("Untitled.md")
                                            .to_string();
                                        send_to_webview(
                                            wv,
                                            &IpcResponse {
                                                response_type: "file_opened",
                                                path: Some(path.to_string_lossy().to_string()),
                                                name: Some(name),
                                                content: Some(content),
                                                message: None,
                                                json: None,
                                            },
                                        );
                                    }
                                    Err(e) => {
                                        send_to_webview(
                                            wv,
                                            &IpcResponse {
                                                response_type: "error",
                                                path: None,
                                                name: None,
                                                content: None,
                                                message: Some(format!("Failed to open file: {}", e)),
                                                json: None,
                                            },
                                        );
                                    }
                                }
                            }
                        }

                        "save_file" => {
                            if let (Some(path_str), Some(content)) = (request.path, request.content) {
                                let path = Path::new(&path_str);
                                match fs::write(path, content) {
                                    Ok(_) => {
                                        let name = path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("Untitled.md")
                                            .to_string();
                                        send_to_webview(
                                            wv,
                                            &IpcResponse {
                                                response_type: "file_saved",
                                                path: Some(path_str),
                                                name: Some(name),
                                                content: None,
                                                message: None,
                                                json: None,
                                            },
                                        );
                                    }
                                    Err(e) => {
                                        send_to_webview(
                                            wv,
                                            &IpcResponse {
                                                response_type: "error",
                                                path: None,
                                                name: None,
                                                content: None,
                                                message: Some(format!("Failed to save file: {}", e)),
                                                json: None,
                                            },
                                        );
                                    }
                                }
                            }
                        }

                        "save_file_as" => {
                            if let Some(content) = request.content {
                                let default_name = request.default_name.unwrap_or_else(|| "document.md".to_string());
                                let dialog = rfd::FileDialog::new()
                                    .add_filter("Markdown (*.md)", &["md"])
                                    .add_filter("All Files (*.*)", &["*"])
                                    .set_file_name(&default_name);

                                if let Some(path) = dialog.save_file() {
                                    match fs::write(&path, content) {
                                        Ok(_) => {
                                            let name = path
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Untitled.md")
                                                .to_string();
                                            send_to_webview(
                                                wv,
                                                &IpcResponse {
                                                    response_type: "file_saved",
                                                    path: Some(path.to_string_lossy().to_string()),
                                                    name: Some(name),
                                                    content: None,
                                                    message: None,
                                                    json: None,
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            send_to_webview(
                                                wv,
                                                &IpcResponse {
                                                    response_type: "error",
                                                    path: None,
                                                    name: None,
                                                    content: None,
                                                    message: Some(format!("Failed to save file: {}", e)),
                                                    json: None,
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        "save_keybindings" => {
                            if let Some(json_content) = request.json {
                                match serde_json::from_str::<serde_json::Value>(&json_content) {
                                    Ok(_) => {
                                        if let Err(e) = fs::write("keybindings.json", &json_content) {
                                            eprintln!("[ERROR] Failed to save keybindings.json: {}", e);
                                            send_to_webview(
                                                wv,
                                                &IpcResponse {
                                                    response_type: "error",
                                                    path: None,
                                                    name: None,
                                                    content: None,
                                                    message: Some(format!("Failed to save keybindings: {}", e)),
                                                    json: None,
                                                },
                                            );
                                        } else {
                                            send_to_webview(
                                                wv,
                                                &IpcResponse {
                                                    response_type: "keybindings_saved",
                                                    path: None,
                                                    name: None,
                                                    content: None,
                                                    message: Some("Keybindings saved successfully.".to_string()),
                                                    json: Some(json_content),
                                                },
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        send_to_webview(
                                            wv,
                                            &IpcResponse {
                                                response_type: "error",
                                                path: None,
                                                name: None,
                                                content: None,
                                                message: Some(format!("Invalid keybindings JSON: {}", e)),
                                                json: None,
                                            },
                                        );
                                    }
                                }
                            }
                        }

                        "reset_default_keybindings" => {
                            if let Err(e) = fs::write("keybindings.json", DEFAULT_KEYBINDINGS) {
                                eprintln!("[ERROR] Failed to write default keybindings: {}", e);
                            }
                            send_to_webview(
                                wv,
                                &IpcResponse {
                                    response_type: "keybindings_reset",
                                    path: None,
                                    name: None,
                                    content: None,
                                    message: Some("Keybindings reset to default settings.".to_string()),
                                    json: Some(DEFAULT_KEYBINDINGS.to_string()),
                                },
                            );
                        }

                        "set_title" => {
                            if let Some(title) = request.title {
                                window_clone_ipc.set_title(&title);
                            }
                        }

                        _ => {}
                    }
                }
            }
        })
        .build(&window_arc)
        .expect("Failed to initialize WebView");

    *webview_holder.lock().unwrap() = Some(webview);

    let webview_for_drop = webview_holder.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(AppEvent::FocusWindow) => {
                window_arc.set_minimized(false);
                window_arc.set_focus();
            }

            Event::UserEvent(AppEvent::OpenFile(path)) => {
                window_arc.set_minimized(false);
                window_arc.set_focus();

                if path.exists() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Untitled.md")
                            .to_string();
                        let holder = webview_for_drop.lock().unwrap();
                        if let Some(wv) = holder.as_ref() {
                            send_to_webview(
                                wv,
                                &IpcResponse {
                                    response_type: "file_opened",
                                    path: Some(path.to_string_lossy().to_string()),
                                    name: Some(name),
                                    content: Some(content),
                                    message: None,
                                    json: None,
                                },
                            );
                        }
                    }
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::DroppedFile(path),
                ..
            } => {
                if path.exists() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Untitled.md")
                            .to_string();
                        let holder = webview_for_drop.lock().unwrap();
                        if let Some(wv) = holder.as_ref() {
                            send_to_webview(
                                wv,
                                &IpcResponse {
                                    response_type: "file_opened",
                                    path: Some(path.to_string_lossy().to_string()),
                                    name: Some(name),
                                    content: Some(content),
                                    message: None,
                                    json: None,
                                },
                            );
                        }
                    }
                }
            }

            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_keybindings() {
        let valid_json = r#"{"preview":{"scroll_down":"j"},"settings":{"scroll_step_y":60}}"#.to_string();
        let (content, err) = validate_or_fallback_keybindings(Ok(valid_json.clone()));
        assert_eq!(content, valid_json);
        assert!(err.is_none());
    }

    #[test]
    fn test_corrupted_keybindings_fallback() {
        let corrupted_json = r#"{"preview": { invalid json "#.to_string();
        let (content, err) = validate_or_fallback_keybindings(Ok(corrupted_json));
        assert_eq!(content, DEFAULT_KEYBINDINGS);
        assert!(err.is_some());
        assert!(err.unwrap().contains("corrupted"));
    }

    #[test]
    fn test_io_error_keybindings_fallback() {
        let io_err = Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"));
        let (content, err) = validate_or_fallback_keybindings(io_err);
        assert_eq!(content, DEFAULT_KEYBINDINGS);
        assert!(err.is_some());
        assert!(err.unwrap().contains("Failed to read"));
    }

    #[test]
    fn test_resolve_target_path() {
        let p = Path::new("Cargo.toml");
        let resolved = resolve_target_path(p);
        assert!(!resolved.is_empty());
        assert!(!resolved.starts_with(r"\\?\"));
        assert!(resolved.ends_with("Cargo.toml"));
    }

    #[test]
    fn test_named_pipe_roundtrip() {
        use std::io::{Read, Write};
        use std::os::windows::ffi::OsStrExt;

        let test_pipe_name = format!(r"\\.\pipe\mdviewer_test_pipe_{}", std::process::id());
        let pipe_wide: Vec<u16> = std::ffi::OsStr::new(&test_pipe_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let pipe_handle = unsafe {
            CreateNamedPipeW(
                pipe_wide.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                255,
                4096,
                4096,
                5000,
                std::ptr::null_mut(),
            )
        };
        assert_ne!(pipe_handle, INVALID_HANDLE_VALUE, "Failed to create named pipe");

        let handle_copy = pipe_handle;
        let server_thread = std::thread::spawn(move || {
            let connected = unsafe { ConnectNamedPipe(handle_copy, std::ptr::null_mut()) };
            assert_ne!(connected, 0, "ConnectNamedPipe failed");

            // Wrap handle into std::fs::File for reading & writing
            use std::os::windows::io::FromRawHandle;
            let mut file = unsafe { std::fs::File::from_raw_handle(handle_copy as _) };
            let mut reader = std::io::BufReader::new(&mut file);
            let mut line = String::new();
            use std::io::BufRead;
            reader.read_line(&mut line).unwrap();
            assert_eq!(line.trim(), "E:\\test\\doc.md");

            let _ = file.write_all(b"OK\n");
            let _ = file.flush();

            // Prevent FromRawHandle from double-closing since CloseHandle will be called
            use std::os::windows::io::IntoRawHandle;
            let _ = file.into_raw_handle();
            unsafe {
                DisconnectNamedPipe(handle_copy);
                CloseHandle(handle_copy);
            }
        });

        // Client side in main test thread
        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut client_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&test_pipe_name)
            .expect("Client failed to open pipe");

        client_file.write_all(b"E:\\test\\doc.md\n").unwrap();
        client_file.flush().unwrap();

        let mut ack = [0u8; 3];
        client_file.read_exact(&mut ack).unwrap();
        assert_eq!(&ack, b"OK\n");

        server_thread.join().unwrap();
    }
}
