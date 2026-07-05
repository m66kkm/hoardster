mod db;
mod error;
mod scanner;
mod epic;
mod steam_api;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use rusqlite::params;
use rusqlite::Connection;
use regex::Regex;

/// 扫描状态，用于支持取消扫描操作
pub struct ScanState {
    pub is_cancelled: Arc<AtomicBool>,
}

/// 抓取状态，用于支持取消 1337x 抓取操作
pub struct ScrapeState {
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
fn get_torrents_1337x_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<db::Torrent1337x>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_torrents_1337x(&conn).map_err(|e| e.to_string())
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

#[tauri::command]
fn open_url_command(url: String) -> Result<(), String> {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct ScrapeProgress {
    current_page: u32,
    total_pages: u32,
    message: String,
    status: String,
}

#[derive(Debug)]
struct ScrapedTorrent {
    torrent_id: String,
    name: String,
    url: String,
    seeds: i64,
    leeches: i64,
    date: String,
    size: String,
    uploader: String,
    uploader_url: String,
}

fn decode_simple_entities(s: &str) -> String {
    s.replace("&amp;", "&")
     .replace("&quot;", "\"")
     .replace("&#39;", "'")
     .replace("&lt;", "<")
     .replace("&gt;", ">")
}

fn parse_page_html(html: &str) -> Vec<ScrapedTorrent> {
    let mut torrents = Vec::new();
    let tr_regex = Regex::new(r"(?s)<tr>(.*?)</tr>").unwrap();
    let url_name_regex = Regex::new(r#"href="(?P<url>/torrent/(?P<id>\d+)/[^"]*)"[^>]*>(?P<name>[^<]+)</a>"#).unwrap();
    let seeds_regex = Regex::new(r#"class="[^"]*seeds"[^>]*>(?P<seeds>\d+)</td>"#).unwrap();
    let leeches_regex = Regex::new(r#"class="[^"]*leeches"[^>]*>(?P<leeches>\d+)</td>"#).unwrap();
    let date_regex = Regex::new(r#"class="coll-date"[^>]*>(?P<date>[^<]+)</td>"#).unwrap();
    let size_regex = Regex::new(r#"class="[^"]*size[^"]*"[^>]*>(?P<size>[^<]+)</td>"#).unwrap();
    let uploader_regex = Regex::new(r#"href="(?P<url>/user/[^"]*)"[^>]*>(?P<name>[^<]+)</a>"#).unwrap();

    for cap in tr_regex.captures_iter(html) {
        let row_html = &cap[1];
        if let Some(url_cap) = url_name_regex.captures(row_html) {
            let id = url_cap["id"].to_string();
            let name = decode_simple_entities(&url_cap["name"]).trim().to_string();
            let url = format!("https://www.1337xx.to{}", &url_cap["url"]);
            
            let seeds = seeds_regex.captures(row_html)
                .map(|c| c["seeds"].parse::<i64>().unwrap_or(0))
                .unwrap_or(0);
            let leeches = leeches_regex.captures(row_html)
                .map(|c| c["leeches"].parse::<i64>().unwrap_or(0))
                .unwrap_or(0);
            let date = date_regex.captures(row_html)
                .map(|c| c["date"].trim().to_string())
                .unwrap_or_default();
            let size = size_regex.captures(row_html)
                .map(|c| c["size"].trim().to_string())
                .unwrap_or_default();
            
            let uploader = uploader_regex.captures(row_html)
                .map(|c| decode_simple_entities(&c["name"]).trim().to_string())
                .unwrap_or_else(|| "Anonymous".to_string());
            let uploader_url = uploader_regex.captures(row_html)
                .map(|c| format!("https://www.1337xx.to{}", &c["url"]))
                .unwrap_or_default();

            torrents.push(ScrapedTorrent {
                torrent_id: id,
                name,
                url,
                seeds,
                leeches,
                date,
                size,
                uploader,
                uploader_url,
            });
        }
    }
    torrents
}

#[tauri::command]
async fn scrape_1337x_command(
    mode: String,
    app_handle: tauri::AppHandle,
    scrape_state: tauri::State<'_, ScrapeState>,
) -> Result<(), String> {
    use tauri::Emitter;
    use tokio::time::{sleep, Duration};
    use std::process::Command;

    let cancel_flag = scrape_state.is_cancelled.clone();
    cancel_flag.store(false, Ordering::Relaxed);

    let total_pages = 150;
    let mut page = 1;
    let session_time = chrono::Utc::now().to_rfc3339();

    while page <= total_pages {
        let event_name = format!("scrape-progress-{}", mode);
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app_handle.emit(&event_name, ScrapeProgress {
                current_page: page - 1,
                total_pages,
                message: "抓取任务已被用户取消".to_string(),
                status: "error".to_string(),
            });
            return Ok(());
        }

        let _ = app_handle.emit(&event_name, ScrapeProgress {
            current_page: page,
            total_pages,
            message: format!("正在获取第 {}/{} 页种子列表...", page, total_pages),
            status: "fetching".to_string(),
        });

        let url = match mode.as_str() {
            "leechers" => format!("https://www.1337xx.to/sort-cat/Games/leechers/desc/{}/", page),
            "seeders" => format!("https://www.1337xx.to/sort-cat/Games/seeders/desc/{}/", page),
            _ => format!("https://www.1337xx.to/sort-cat/Games/time/desc/{}/", page),
        };
        
        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

        let mut cmd = Command::new("curl");
        cmd.args([
            "-s",
            "-A", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            &url
        ]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let output = cmd.output();

        if let Ok(out) = output {
            if out.status.success() {
                let html = String::from_utf8_lossy(&out.stdout).to_string();
                let parsed = parse_page_html(&html);
                if !parsed.is_empty() {
                    let db_path = db::get_db_path();
                    let session_time_clone = session_time.clone();
                    let has_existing = tokio::task::spawn_blocking(move || -> Result<bool, String> {
                        let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;
                        let mut found_existing = false;
                        if let Ok(tx) = conn.transaction() {
                            for t in parsed {
                                let exists: bool = tx.query_row(
                                    "SELECT EXISTS(SELECT 1 FROM torrents_1337x WHERE torrent_id = ?)",
                                    params![t.torrent_id],
                                    |row| row.get(0),
                                ).unwrap_or(false);
                                
                                if exists {
                                    found_existing = true;
                                }
                                
                                let _ = tx.execute(
                                    "INSERT OR IGNORE INTO torrents_1337x (
                                        torrent_id, name, url, seeds, leeches, date, size, uploader, uploader_url, fetched_at
                                     ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                    params![
                                        t.torrent_id,
                                        t.name,
                                        t.url,
                                        t.seeds,
                                        t.leeches,
                                        t.date,
                                        t.size,
                                        t.uploader,
                                        t.uploader_url,
                                        &session_time_clone,
                                    ],
                                );
                            }
                            let _ = tx.commit();
                        }
                        Ok(found_existing)
                    }).await.map_err(|e| e.to_string())??;
                    
                    if has_existing {
                        break;
                    }
                }
            }
        }

        page += 1;
        sleep(Duration::from_millis(200)).await;
    }

    let event_name = format!("scrape-progress-{}", mode);
    let _ = app_handle.emit(&event_name, ScrapeProgress {
        current_page: total_pages,
        total_pages,
        message: "更新完成！1337x 数据已成功同步至本地。".to_string(),
        status: "complete".to_string(),
    });

    Ok(())
}

#[tauri::command]
fn cancel_scrape_command(scrape_state: tauri::State<'_, ScrapeState>) -> Result<(), String> {
    scrape_state.is_cancelled.store(true, Ordering::Relaxed);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            // 注册抓取取消状态到 Tauri 托管状态
            app.manage(ScrapeState {
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
            get_torrents_1337x_command,
            open_url_command,
            scrape_1337x_command,
            cancel_scrape_command,
            epic::fetch_epic_free_games_command,
            epic::get_epic_free_games_command,
            steam_api::fetch_steam_free_games_command,
            steam_api::get_steam_free_games_command
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
