mod db;
mod error;
mod scanner;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// 扫描状态，用于支持取消扫描操作
pub struct ScanState {
    pub is_cancelled: Arc<AtomicBool>,
}

#[tauri::command]
fn get_scan_paths_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_scan_paths(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_scan_path_command(state: tauri::State<'_, db::DbState>, path: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_scan_path(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_scan_path_command(state: tauri::State<'_, db::DbState>, path: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::remove_scan_path(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_games_stats_command(state: tauri::State<'_, db::DbState>) -> Result<db::StatsSummary, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_games_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_games_list_command(
    search: String,
    drive: String,
    r#type: String,
    rating: String,
    sort: String,
    only_representatives: bool,
    only_installed: bool,
    state: tauri::State<'_, db::DbState>,
) -> Result<Vec<db::Game>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_games_list(
        &conn,
        &search,
        &drive,
        &r#type,
        &rating,
        &sort,
        only_representatives,
        only_installed,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_duplicates_command(
    state: tauri::State<'_, db::DbState>,
    dup_type: String,
) -> Result<Vec<db::DuplicateGroup>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_duplicates(&conn, &dup_type).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_franchises_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<db::FranchiseGroup>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_franchises(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_scan_command(
    app_handle: tauri::AppHandle,
    scan_state: tauri::State<'_, ScanState>,
) -> Result<(), String> {
    // 在进入 spawn_blocking 之前 clone 出 cancel_flag（不能将 State 传入闭包）
    let cancel_flag = scan_state.is_cancelled.clone();
    // 重置取消标志
    cancel_flag.store(false, Ordering::Relaxed);

    tokio::task::spawn_blocking(move || {
        scanner::run_scan(app_handle, cancel_flag)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 取消正在进行的扫描操作
#[tauri::command]
fn cancel_scan_command(scan_state: tauri::State<'_, ScanState>) -> Result<(), String> {
    scan_state.is_cancelled.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
fn get_config_command(state: tauri::State<'_, db::DbState>, key: String) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_config(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_config_command(state: tauri::State<'_, db::DbState>, key: String, value: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::set_config(&conn, &key, &value).map_err(|e| e.to_string())
}

/// 获取所有配置项
#[tauri::command]
fn get_all_config_command(state: tauri::State<'_, db::DbState>) -> Result<HashMap<String, String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_all_config(&conn).map_err(|e| e.to_string())
}

/// 获取扫描历史记录
#[tauri::command]
fn get_scan_history_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<db::ScanHistoryRecord>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_scan_history(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_steam_cache_command(state: tauri::State<'_, db::DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::clear_steam_cache(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_genres_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_all_genres(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_genre_stats_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<db::GenreStat>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_genre_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_rating_stats_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<db::RatingStat>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_rating_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_game_folder_command(path: String) -> Result<(), String> {
    use std::process::Command;
    Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化数据库连接
            let db_path = db::get_db_path();
            let conn = rusqlite::Connection::open(db_path)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            db::init_db(&conn)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            // 注册数据库状态到 Tauri 托管状态
            app.manage(db::DbState(Mutex::new(conn)));
            // 注册扫描状态到 Tauri 托管状态
            app.manage(ScanState {
                is_cancelled: Arc::new(AtomicBool::new(false)),
            });

            Ok(())
        })
        .register_uri_scheme_protocol("cover", |_app_handle, request| {
            let uri = request.uri();
            let filename = uri.path().trim_start_matches('/');
            
            // 从 exe 所在目录解析 covers 路径
            let mut path = std::env::current_exe().unwrap_or_default();
            path.pop(); // 移除 exe 文件名，保留目录
            path.push("covers");
            path.push(filename);
            
            let body = if path.exists() {
                std::fs::read(&path).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let mime = if filename.ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };
            
            tauri::http::Response::builder()
                .header("content-type", mime)
                .header("access-control-allow-origin", "*")
                .body(body)
                .unwrap()
        })
        .invoke_handler(tauri::generate_handler![
            get_scan_paths_command,
            add_scan_path_command,
            remove_scan_path_command,
            get_games_stats_command,
            get_games_list_command,
            get_duplicates_command,
            get_franchises_command,
            start_scan_command,
            cancel_scan_command,
            open_game_folder_command,
            get_config_command,
            set_config_command,
            get_all_config_command,
            get_scan_history_command,
            clear_steam_cache_command,
            get_all_genres_command,
            get_genre_stats_command,
            get_rating_stats_command,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
