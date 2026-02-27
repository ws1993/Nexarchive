mod app_state;
mod constants;
mod errors;
mod models;
mod services;
mod utils;

use std::sync::Arc;

use app_state::AppState;
use models::{AppConfig, LogFilters, PagedResult};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};

#[tauri::command]
async fn init_system(
    state: State<'_, Arc<AppState>>,
    inbox_path: String,
    archive_root_path: String,
) -> Result<bool, String> {
    state
        .init_system(inbox_path, archive_root_path)
        .await
        .map_err(err_to_string)
}

#[tauri::command]
async fn get_init_preview(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<models::InitPreviewItem>, String> {
    Ok(state.get_init_preview().await)
}

#[tauri::command]
async fn save_settings(state: State<'_, Arc<AppState>>, config: AppConfig) -> Result<bool, String> {
    state.save_settings(config).await.map_err(err_to_string)
}

#[tauri::command]
async fn load_settings(state: State<'_, Arc<AppState>>) -> Result<AppConfig, String> {
    Ok(state.load_settings().await)
}

#[tauri::command]
async fn test_llm_connection(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    state.test_llm_connection().await.map_err(err_to_string)
}

#[tauri::command]
async fn test_mineru_connection(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    state.test_mineru_connection().await.map_err(err_to_string)
}

#[tauri::command]
async fn run_job_once(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    state.run_job_once().await.map_err(err_to_string)
}

#[tauri::command]
fn get_jobs(
    state: State<'_, Arc<AppState>>,
    page: usize,
    page_size: usize,
    status: Option<String>,
    date_range: Option<Vec<String>>,
) -> Result<PagedResult<models::JobRecord>, String> {
    state
        .get_jobs(page, page_size, status, date_range)
        .map_err(err_to_string)
}

#[tauri::command]
fn get_file_tasks(
    state: State<'_, Arc<AppState>>,
    job_id: String,
    status: Option<String>,
) -> Result<Vec<models::FileTaskRecord>, String> {
    state.get_file_tasks(job_id, status).map_err(err_to_string)
}

#[tauri::command]
fn get_logs(
    state: State<'_, Arc<AppState>>,
    filters: LogFilters,
) -> Result<PagedResult<models::LogEvent>, String> {
    state.get_logs(filters).map_err(err_to_string)
}

#[tauri::command]
fn restore_from_recycle_bin(
    state: State<'_, Arc<AppState>>,
    task_id: String,
) -> Result<bool, String> {
    state
        .restore_from_recycle_bin(task_id)
        .map_err(err_to_string)
}

#[tauri::command]
fn undo_archive_task(state: State<'_, Arc<AppState>>, task_id: String) -> Result<bool, String> {
    state.undo_archive_task(task_id).map_err(err_to_string)
}

fn err_to_string(err: impl std::fmt::Display) -> String {
    err.to_string()
}

pub fn run() {
    let state = AppState::new().expect("init app state failed");
    let boot_state = state.clone();

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            let show_item =
                MenuItem::with_id(app, "show", "Show NexArchive", true, Option::<&str>::None)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, Option::<&str>::None)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = {
                let mut builder = TrayIconBuilder::new()
                    .menu(&tray_menu)
                    .show_menu_on_left_click(false);
                if let Some(icon) = app.default_window_icon() {
                    builder = builder.icon(icon.clone());
                }
                builder
                .on_menu_event(|app_handle, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        std::process::exit(0);
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
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?
            };

            tauri::async_runtime::spawn(async move {
                boot_state.bootstrap_scheduler().await;
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Arc<AppState>>();
                if state.run_in_background_enabled() {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            init_system,
            get_init_preview,
            save_settings,
            load_settings,
            test_llm_connection,
            test_mineru_connection,
            run_job_once,
            get_jobs,
            get_file_tasks,
            get_logs,
            restore_from_recycle_bin,
            undo_archive_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
