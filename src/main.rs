#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{WebView, WebViewBuilder};

const APP_HTML: &str = include_str!("../assets/app.html");
const SAMPLE_MD: &str = include_str!("../assets/sample.md");

#[derive(Debug, Deserialize)]
struct IpcRequest {
    cmd: String,
    path: Option<String>,
    content: Option<String>,
    #[serde(rename = "defaultName")]
    default_name: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct IpcResponse<'a> {
    #[serde(rename = "type")]
    response_type: &'a str,
    path: Option<String>,
    name: Option<String>,
    content: Option<String>,
    message: Option<String>,
}

fn send_to_webview(webview: &WebView, response: &IpcResponse) {
    if let Ok(json) = serde_json::to_string(response) {
        let script = format!("window.receiveFromRust({});", json);
        let _ = webview.evaluate_script(&script);
    }
}

fn main() {
    // Get command line argument (e.g. mdviewer.exe "path\to\file.md")
    let initial_path_arg = std::env::args().nth(1).map(PathBuf::from);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("MDViewer - Markdown & UDP Diagram Editor")
        .with_inner_size(LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(LogicalSize::new(600.0, 400.0))
        .build(&event_loop)
        .expect("Failed to create application window");

    let window_arc = Arc::new(window);
    let window_clone_ipc = window_arc.clone();

    let webview_holder: Arc<Mutex<Option<WebView>>> = Arc::new(Mutex::new(None));
    let webview_for_ipc = webview_holder.clone();

    // Initial file state
    let initial_path_for_ipc = initial_path_arg.clone();

    let webview = WebViewBuilder::new()
        .with_html(APP_HTML)
        .with_ipc_handler(move |req| {
            let body = req.body();
            if let Ok(request) = serde_json::from_str::<IpcRequest>(body) {
                let holder = webview_for_ipc.lock().unwrap();
                if let Some(wv) = holder.as_ref() {
                    match request.cmd.as_str() {
                        "get_init_data" => {
                            if let Some(ref path_buf) = initial_path_for_ipc {
                                if path_buf.exists() {
                                    match fs::read_to_string(path_buf) {
                                        Ok(content) => {
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
                                                    message: None,
                                                },
                                            );
                                            return;
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to read file: {:?}", e);
                                        }
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
                                    message: None,
                                },
                            );
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
                                                },
                                            );
                                        }
                                    }
                                }
                            }
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
