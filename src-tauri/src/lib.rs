mod db;
mod error;
mod scanner;
mod epic;
mod steam_api;
mod steam_service;
mod data_correction;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use rusqlite::params;
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
async fn fetch_steam_game_info_command(
    base_name: String,
    _app_handle: tauri::AppHandle,
    state: tauri::State<'_, db::DbState>,
) -> Result<Option<crate::steam_service::SteamCacheEntry>, String> {
    // 获取语言设置
    let lang = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        match db::get_config(&conn, "language") {
            Ok(Some(l)) => l,
            _ => "schinese".to_string(),
        }
    };

    let base_name_clone = base_name.clone();
    let mut covers_dir = std::env::current_exe().unwrap_or_default();
    covers_dir.pop();
    covers_dir.push("covers");
    let _ = std::fs::create_dir_all(&covers_dir);

    // 由于 reqwest::blocking 是同步的，我们需要在阻塞线程中创建和运行 Client
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<Option<crate::steam_service::SteamCacheEntry>, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let mut entry = match crate::steam_service::fetch_steam_game_info(&client, &base_name_clone, &lang) {
            Some(e) => e,
            None => return Ok(None),
        };
        
        // 下载封面图
        if let Some(cover_url) = entry.local_cover.clone() {
            if let Some(app_id) = entry.appid {
                let cover_filename = format!("{}.jpg", app_id);
                let local_path = covers_dir.join(&cover_filename);

                let mut download_success = false;
                if let Ok(head_res) = client.head(&cover_url).send() {
                    if head_res.status().is_success() {
                        if let Ok(img_res) = client.get(&cover_url).send() {
                            if img_res.status().is_success() {
                                if let Ok(img_bytes) = img_res.bytes() {
                                    let _ = std::fs::write(&local_path, &img_bytes);
                                    download_success = true;
                                }
                            }
                        }
                    }
                }

                if !download_success {
                    let fallback_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/capsule_616x353.jpg", app_id);
                    if let Ok(img_res) = client.get(&fallback_url).send() {
                        if img_res.status().is_success() {
                            if let Ok(img_bytes) = img_res.bytes() {
                                let _ = std::fs::write(&local_path, &img_bytes);
                                download_success = true;
                            }
                        }
                    }
                }

                if download_success {
                    entry.local_cover = Some(format!("covers/{}", cover_filename));
                } else {
                    entry.local_cover = None;
                }
            }
        }
        Ok(Some(entry))
    })
    .await
    .map_err(|e| e.to_string())??;

    if let Some(entry) = result {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        db::insert_steam_cache_entry(&conn, &entry).map_err(|e| e.to_string())?;
        Ok(Some(entry))
    } else {
        Ok(None)
    }
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

use chrono::Datelike;

fn parse_1337x_date(date_str: &str) -> i64 {
    let now = chrono::Utc::now().timestamp();
    let lower = date_str.to_lowercase();
    
    if let Ok(re) = regex::Regex::new(r"^(\d+)\s+(min|hr|hour|day|week|month|year)") {
        if let Some(caps) = re.captures(&lower) {
            let num: i64 = caps[1].parse().unwrap_or(0);
            let unit = &caps[2];
            if unit.starts_with("min") { return now - num * 60; }
            if unit.starts_with("hr") || unit.starts_with("hour") { return now - num * 3600; }
            if unit.starts_with("day") { return now - num * 86400; }
            if unit.starts_with("week") { return now - num * 7 * 86400; }
            if unit.starts_with("month") { return now - num * 30 * 86400; }
            if unit.starts_with("year") { return now - num * 365 * 86400; }
        }
    }

    if lower.contains("today") || lower.contains("am") || lower.contains("pm") {
        return now;
    }
    if lower.contains("yesterday") {
        return now - 86400;
    }
    
    let current_year = chrono::Utc::now().year();
    let mut year = current_year;
    
    if let Ok(re) = regex::Regex::new(r"'(\d{2})") {
        if let Some(caps) = re.captures(&lower) {
            let y: i32 = caps[1].parse().unwrap_or(0);
            if y > 0 { year = 2000 + y; }
        }
    }
    
    let mut month = 1;
    let months = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    for (i, m) in months.iter().enumerate() {
        if lower.contains(m) {
            month = i as u32 + 1;
            break;
        }
    }
    
    let mut day = 1;
    if let Ok(re) = regex::Regex::new(r"(\d+)(st|nd|rd|th)") {
        if let Some(caps) = re.captures(&lower) {
            day = caps[1].parse().unwrap_or(1);
        }
    }
    
    use chrono::TimeZone;
    if let chrono::LocalResult::Single(dt) = chrono::Utc.with_ymd_and_hms(year, month, day, 0, 0, 0) {
        return dt.timestamp();
    }
    
    now
}

fn parse_page_html(html: &str) -> Vec<db::Torrent1337x> {
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
            
            let published_ts = parse_1337x_date(&date);
            let base_name = Some(crate::scanner::base_game_name(&name));

            torrents.push(db::Torrent1337x {
                id: None,
                torrent_id: id,
                name,
                url,
                seeds,
                leeches,
                date,
                size,
                uploader,
                uploader_url,
                published_ts,
                base_name,
                appid: None,
                local_cover: None,
                review_score_desc: None,
                positive_percent: None,
                total_reviews: None,
            });
        }
    }
    torrents
}

fn sync_steam_reviews_blocking(
    app_handle: tauri::AppHandle,
    event_name: String,
    missing_games: Vec<String>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<bool, String> {
    use tauri::Emitter;
    use std::collections::VecDeque;
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

    let total_missing = missing_games.len();
    if total_missing == 0 {
        return Ok(false);
    }

    // 1. 读取配置文件中设置的线程数与延迟（不强制写死限流值）
    let (threads, delay_ms, language) = {
        if let Ok(conn) = crate::db::get_connection() {
            let t_str: String = conn.query_row("SELECT value FROM config WHERE key = 'steam_api_threads'", [], |r| r.get(0)).unwrap_or_else(|_| "10".to_string());
            let d_str: String = conn.query_row("SELECT value FROM config WHERE key = 'steam_api_delay_ms'", [], |r| r.get(0)).unwrap_or_else(|_| "300".to_string());
            let l_str: String = conn.query_row("SELECT value FROM config WHERE key = 'language'", [], |r| r.get(0)).unwrap_or_else(|_| "schinese".to_string());
            (
                t_str.parse::<usize>().unwrap_or(10).max(1).min(20),
                d_str.parse::<u64>().unwrap_or(300),
                l_str,
            )
        } else {
            (10, 300, "schinese".to_string())
        }
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    // 任务队列：(base_name, retry_count)
    let queue: Arc<Mutex<VecDeque<(String, usize)>>> = Arc::new(Mutex::new(
        missing_games.into_iter().map(|name| (name, 0)).collect()
    ));

    // 自适应频控控制器：当出现 403 时降为单线程保护期 1 分钟；保护期恢复时每 1 分钟 +1 并发，若 +1 后遇 403 则回滚 -1
    use crate::steam_service::ThrottleController;

    let throttle_ctrl = Arc::new(Mutex::new(ThrottleController::new(threads)));
    let active_tasks = Arc::new(AtomicUsize::new(0));

    let covers_dir = {
        let mut p = std::env::current_exe().unwrap_or_default();
        p.pop();
        p.push("covers");
        let _ = std::fs::create_dir_all(&p);
        Arc::new(p)
    };

    enum WorkerMessage {
        StatusNotification {
            message: String,
        },
        ItemProcessed {
            base_name: String,
            entry: Option<crate::steam_service::SteamCacheEntry>,
            current_concurrency: usize,
            is_ramping: bool,
        },
    }

    let (tx, rx) = std::sync::mpsc::channel::<WorkerMessage>();

    let mut thread_handles = Vec::new();
    for thread_idx in 0..threads {
        let tx_clone = tx.clone();
        let queue_clone = Arc::clone(&queue);
        let cancel_clone = Arc::clone(&cancel_flag);
        let throttle_ctrl_clone = Arc::clone(&throttle_ctrl);
        let active_tasks_clone = Arc::clone(&active_tasks);
        let client_clone = client.clone();
        let lang = language.clone();
        let covers_dir_clone = Arc::clone(&covers_dir);

        let handle = std::thread::spawn(move || {
            loop {
                if cancel_clone.load(Ordering::Relaxed) {
                    break;
                }

                // 检查自适应爬坡（满1分钟 +1 并发）
                let (current_threads, is_ramping) = {
                    let mut ctrl = throttle_ctrl_clone.lock().unwrap();
                    let ramp_msg = ctrl.check_ramp();
                    let cur = ctrl.get_current_threads();
                    let ramping = ctrl.is_ramping();
                    drop(ctrl);
                    if let Some((_c, msg)) = ramp_msg {
                        let _ = tx_clone.send(WorkerMessage::StatusNotification { message: msg });
                    }
                    (cur, ramping)
                };

                // 若当前线程号超出当前允许的并发限制，休眠等待
                if thread_idx >= current_threads {
                    // 若所有任务均已结束且队列为空，无须继续等待，直接退出
                    let is_done = {
                        let q = queue_clone.lock().unwrap();
                        q.is_empty() && active_tasks_clone.load(Ordering::SeqCst) == 0
                    };
                    if is_done {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }

                // 尝试提取下一个待抓取游戏
                let task = {
                    let mut q = queue_clone.lock().unwrap();
                    q.pop_front()
                };

                let (base_name, retry_count) = match task {
                    Some(item) => {
                        active_tasks_clone.fetch_add(1, Ordering::SeqCst);
                        item
                    }
                    None => {
                        // 队列暂空：若仍有线程在处理任务（可能因 403 重新入队），则等待片刻
                        if active_tasks_clone.load(Ordering::SeqCst) > 0 {
                            std::thread::sleep(Duration::from_millis(100));
                            continue;
                        } else {
                            break;
                        }
                    }
                };

                // 根据当前并发保护状态调整请求延迟
                let sleep_ms = if is_ramping && current_threads == 1 {
                    delay_ms.max(1000)
                } else if is_ramping {
                    delay_ms.max(500)
                } else {
                    delay_ms
                };
                if sleep_ms > 0 {
                    std::thread::sleep(Duration::from_millis(sleep_ms));
                }

                if cancel_clone.load(Ordering::Relaxed) {
                    active_tasks_clone.fetch_sub(1, Ordering::SeqCst);
                    break;
                }

                let (maybe_entry, got_403) = crate::steam_service::fetch_steam_game_info_ext(&client_clone, &base_name, &lang);

                if got_403 {
                    // 遭遇 403：触发降级或回滚（+1后遇403则回滚到 -1）
                    let notify_msg = {
                        let mut ctrl = throttle_ctrl_clone.lock().unwrap();
                        let (_c, msg) = ctrl.on_403(&base_name);
                        msg
                    };
                    let _ = tx_clone.send(WorkerMessage::StatusNotification { message: notify_msg });

                    // 若未达到最大重试次数 (2次)，重新放入队列尾部等待重试
                    if retry_count < 2 {
                        {
                            let mut q = queue_clone.lock().unwrap();
                            q.push_back((base_name, retry_count + 1));
                        }
                        active_tasks_clone.fetch_sub(1, Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(2000));
                        continue;
                    }
                }

                // 处理封面与结果装配
                let final_entry = if let Some(mut entry) = maybe_entry {
                    if let Some(app_id) = entry.appid {
                        let cover_filename = format!("{}.jpg", app_id);
                        let local_path = covers_dir_clone.join(&cover_filename);
                        if local_path.exists() {
                            entry.local_cover = Some(format!("covers/{}", cover_filename));
                        } else {
                            let mut urls_to_try = Vec::new();
                            urls_to_try.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/capsule_616x353.jpg", app_id));
                            if let Some(ref u) = entry.local_cover {
                                if u.starts_with("http") && !urls_to_try.contains(u) {
                                    urls_to_try.push(u.clone());
                                }
                            }
                            urls_to_try.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg", app_id));

                            for u in urls_to_try {
                                if let Ok(img_res) = client_clone.get(&u).send() {
                                    if img_res.status().is_success() {
                                        if let Ok(img_bytes) = img_res.bytes() {
                                            let _ = std::fs::write(&local_path, &img_bytes);
                                            entry.local_cover = Some(format!("covers/{}", cover_filename));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(entry)
                } else {
                    None
                };

                let _ = tx_clone.send(WorkerMessage::ItemProcessed {
                    base_name,
                    entry: final_entry,
                    current_concurrency: current_threads,
                    is_ramping,
                });

                active_tasks_clone.fetch_sub(1, Ordering::SeqCst);
            }
        });
        thread_handles.push(handle);
    }

    drop(tx);
    drop(client); // 释放 client 资源

    let mut processed = 0;
    let conn = crate::db::get_connection().map_err(|e| e.to_string())?;

    for msg in rx {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app_handle.emit(&event_name, ScrapeProgress {
                current_page: processed,
                total_pages: total_missing as u32,
                message: "Steam 信息获取已被用户取消".to_string(),
                status: "error".to_string(),
            });
            for handle in thread_handles {
                let _ = handle.join();
            }
            return Ok(true);
        }

        match msg {
            WorkerMessage::StatusNotification { message } => {
                let _ = app_handle.emit(&event_name, ScrapeProgress {
                    current_page: processed,
                    total_pages: total_missing as u32,
                    message,
                    status: "fetching".to_string(),
                });
            }
            WorkerMessage::ItemProcessed { base_name, entry, current_concurrency, is_ramping } => {
                processed += 1;
                let ramp_tag = if is_ramping {
                    format!(" [保护恢复中: {}/{}线程]", current_concurrency, threads)
                } else {
                    String::new()
                };
                let _ = app_handle.emit(&event_name, ScrapeProgress {
                    current_page: processed,
                    total_pages: total_missing as u32,
                    message: format!("正在向 Steam 获取游戏评价 ({} / {}){}: {}", processed, total_missing, ramp_tag, base_name),
                    status: "fetching".to_string(),
                });

                if let Some(entry) = entry {
                    if entry.appid.is_some() {
                        for attempt in 0..3 {
                            match crate::db::insert_steam_cache_entry(&conn, &entry) {
                                Ok(_) => break,
                                Err(e) => {
                                    let err_str = e.to_string();
                                    if err_str.contains("locked") && attempt < 2 {
                                        std::thread::sleep(Duration::from_millis(300 * (attempt as u64 + 1)));
                                    } else {
                                        eprintln!("Error inserting steam cache for {}: {}", base_name, e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for handle in thread_handles {
        let _ = handle.join();
    }

    Ok(false)
}

#[tauri::command]
async fn scrape_1337x_command(
    mode: String,
    app_handle: tauri::AppHandle,
    scrape_state: tauri::State<'_, ScrapeState>,
) -> Result<String, String> {
    use tauri::Emitter;
    use tokio::time::{sleep, Duration};
    use std::process::Command;

    let cancel_flag = scrape_state.is_cancelled.clone();
    cancel_flag.store(false, Ordering::Relaxed);

    let total_pages = 150;
    let mut page = 1;
    let mut total_new_added = 0;
    let session_time = chrono::Utc::now().to_rfc3339();

    let concurrency = {
        if let Ok(conn) = crate::db::get_connection() {
            let threads_str: String = conn.query_row("SELECT value FROM config WHERE key = 'steam_api_threads'", [], |r| r.get(0)).unwrap_or_else(|_| "5".to_string());
            threads_str.parse::<u32>().unwrap_or(5).max(1).min(20)
        } else {
            5
        }
    };

    let mut consecutive_existing = 0;

    while page <= total_pages {
        let event_name = format!("scrape-progress-{}", mode);
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app_handle.emit(&event_name, ScrapeProgress {
                current_page: page - 1,
                total_pages,
                message: "抓取任务已被用户取消".to_string(),
                status: "error".to_string(),
            });
            return Ok("抓取任务已被用户取消".to_string());
        }

        let end_page = (page + concurrency - 1).min(total_pages);
        let _ = app_handle.emit(&event_name, ScrapeProgress {
            current_page: end_page,
            total_pages,
            message: format!("并发获取第 {} - {} 页种子...", page, end_page),
            status: "fetching".to_string(),
        });

        let mut handles = Vec::new();
        for p in page..=end_page {
            let url = match mode.as_str() {
                "leechers" => format!("https://www.1337xx.to/sort-cat/Games/leechers/desc/{}/", p),
                "seeders" => format!("https://www.1337xx.to/sort-cat/Games/seeders/desc/{}/", p),
                _ => format!("https://www.1337xx.to/sort-cat/Games/time/desc/{}/", p),
            };
            
            let handle = tokio::task::spawn_blocking(move || {
                #[cfg(target_os = "windows")]
                use std::os::windows::process::CommandExt;
                let mut cmd = Command::new("curl");
                cmd.args([
                    "-s",
                    "-A", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                    &url
                ]);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(0x08000000);

                if let Ok(out) = cmd.output() {
                    if out.status.success() {
                        let html = String::from_utf8_lossy(&out.stdout).to_string();
                        return parse_page_html(&html);
                    }
                }
                Vec::new()
            });
            handles.push(handle);
        }

        let mut all_parsed = Vec::new();
        for h in handles {
            if let Ok(parsed) = h.await {
                all_parsed.extend(parsed);
            }
        }

        if !all_parsed.is_empty() {
            let mut seen = std::collections::HashSet::new();
            all_parsed.retain(|t| seen.insert(t.torrent_id.clone()));
            let session_time_clone = session_time.clone();
            let (new_consecutive, new_added) = tokio::task::spawn_blocking(move || -> Result<(usize, usize), String> {
                let mut conn = crate::db::get_connection().map_err(|e| e.to_string())?;
                let mut local_consecutive = consecutive_existing;
                let mut newly_added = 0;
                
                if let Ok(tx) = conn.transaction() {
                    for t in all_parsed {
                        let exists: bool = tx.query_row(
                            "SELECT EXISTS(SELECT 1 FROM torrents_1337x WHERE torrent_id = ?)",
                            params![t.torrent_id],
                            |row| row.get(0),
                        ).unwrap_or(false);
                        
                        if exists {
                            local_consecutive += 1;
                        } else {
                            local_consecutive = 0;
                        }
                        
                        let base_name = t.base_name.as_deref().unwrap_or("");
                        let rows_affected = tx.execute(
                            "INSERT OR IGNORE INTO torrents_1337x (
                                torrent_id, name, url, seeds, leeches, date, size, uploader, uploader_url, fetched_at, published_ts, base_name
                             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                                t.published_ts,
                                base_name,
                            ],
                        ).unwrap_or(0);
                        
                        newly_added += rows_affected as usize;
                    }
                    let _ = tx.commit();
                }
                Ok((local_consecutive, newly_added))
            }).await.map_err(|e| e.to_string())??;
            
            consecutive_existing = new_consecutive;
            total_new_added += new_added;
            
            if consecutive_existing >= 5 && (mode == "time" || mode == "latest") {
                break;
            }
        }

        page = end_page + 1;
        sleep(Duration::from_millis(200)).await;
    }

    // 阶段二：当游戏清单获取完成后，和游戏索引模块一致，去 Steam 获取评价信息
    let event_name = format!("scrape-progress-{}", mode);
    if cancel_flag.load(Ordering::Relaxed) {
        let _ = app_handle.emit(&event_name, ScrapeProgress {
            current_page: page - 1,
            total_pages,
            message: "抓取任务已被用户取消".to_string(),
            status: "error".to_string(),
        });
        return Ok("抓取任务已被用户取消".to_string());
    }

    // 自动用最新清洗规则校准 1337x 记录的 base_name
    if let Ok(conn) = crate::db::get_connection() {
        let all_rows: Vec<(i64, String, Option<String>)> = {
            if let Ok(mut stmt) = conn.prepare("SELECT id, name, base_name FROM torrents_1337x") {
                if let Ok(rows) = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))) {
                    rows.filter_map(|r| r.ok()).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };
        if let Ok(mut update_stmt) = conn.prepare("UPDATE torrents_1337x SET base_name = ? WHERE torrent_id = ?") {
            let _ = conn.execute_batch("BEGIN TRANSACTION;");
            for (id, name, existing_base) in all_rows {
                let base = crate::scanner::base_game_name(&name);
                if existing_base.as_deref() != Some(&base) {
                    let _ = update_stmt.execute(params![base, id]);
                }
            }
            let _ = conn.execute_batch("COMMIT;");
        }
    }

    // 找出尚未在 steam_cache 中缓存的 1337x 游戏 base_name
    let missing_games: Vec<String> = {
        if let Ok(conn) = crate::db::get_connection() {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT t.base_name 
                 FROM torrents_1337x t
                 LEFT JOIN steam_db.steam_cache s ON t.base_name = s.base_name
                 WHERE t.base_name IS NOT NULL 
                   AND TRIM(t.base_name) != '' 
                   AND (s.base_name IS NULL OR s.appid IS NULL)
                 GROUP BY t.base_name
                 ORDER BY MAX(t.published_ts) DESC"
            ) {
                if let Ok(rows) = stmt.query_map([], |r| r.get(0)) {
                    rows.filter_map(|r| r.ok()).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    };

    let total_missing = missing_games.len();
    if total_missing > 0 {
        let app_handle_clone = app_handle.clone();
        let event_name_clone = event_name.clone();
        let cancel_flag_clone = cancel_flag.clone();

        let was_cancelled = tokio::task::spawn_blocking(move || {
            sync_steam_reviews_blocking(app_handle_clone, event_name_clone, missing_games, cancel_flag_clone)
        })
        .await
        .map_err(|e| e.to_string())??;

        if was_cancelled {
            return Ok("Steam 信息获取已被用户取消".to_string());
        }
    }

    if let Ok(conn) = crate::db::get_connection() {
        let _ = crate::db::sync_game_metadata_covers(&conn);
    }

    let _ = app_handle.emit(&event_name, ScrapeProgress {
        current_page: 100,
        total_pages: 100,
        message: "1337x 数据与 Steam 评价同步完成！".to_string(),
        status: "complete".to_string(),
    });

    let msg = if total_new_added > 0 {
        format!("更新完成！本次同步新增了 {} 个游戏种子，并已同步 Steam 评价。", total_new_added)
    } else {
        "更新完成！1337x 游戏列表已是最新，并已同步 Steam 评价。".to_string()
    };
    Ok(msg)
}

#[tauri::command]
fn cancel_scrape_command(scrape_state: tauri::State<'_, ScrapeState>) -> Result<(), String> {
    scrape_state.is_cancelled.store(true, Ordering::Relaxed);
    Ok(())
}

fn parse_sr_date(date_str: &str) -> i64 {
    let now = chrono::Utc::now().timestamp();
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        if let Some(datetime) = dt.and_hms_opt(0, 0, 0) {
            return datetime.and_utc().timestamp();
        }
    }
    now
}

fn parse_sr_html(html: &str) -> Vec<db::TorrentSR> {
    let mut torrents = Vec::new();
    let block_regex = Regex::new(r"(?s)<div class=.post (.*?)</div><!--End post-->").unwrap();
    let title_url_regex = Regex::new(r#"<h2><a href="([^"]+)">([^<]+)</a></h2>"#).unwrap();
    let meta_regex = Regex::new(r"(?s)Posted\s+(.*?)\s+in\s+(.*?)\s*</div>").unwrap();
    let img_regex = Regex::new(r#"<img[^>]*src="([^"]+)""#).unwrap();
    let cat_regex = Regex::new(r#"<a[^>]*>([^<]+)</a>"#).unwrap();
    let comments_regex = Regex::new(r#"class="comments-link"[^>]*>(\d+)\s+Comment"#).unwrap();
    
    let now_str = chrono::Utc::now().to_rfc3339();

    for cap in block_regex.captures_iter(html) {
        let block = &cap[1];
        
        let mut url = String::new();
        let mut title = String::new();
        if let Some(tu) = title_url_regex.captures(block) {
            url = tu[1].to_string();
            title = decode_simple_entities(&tu[2]);
        }
        
        if url.is_empty() || title.is_empty() {
            continue;
        }
        
        let id = url.clone();
        
        let mut date_str = String::new();
        let mut cats = Vec::new();
        if let Some(m) = meta_regex.captures(block) {
            date_str = m[1].trim().to_string();
            let cat_html = &m[2];
            for c in cat_regex.captures_iter(cat_html) {
                let cat_name = decode_simple_entities(&c[1]);
                let lower_cat = cat_name.to_lowercase();
                if !lower_cat.contains("request accepted") && !lower_cat.contains("pc games") {
                    cats.push(cat_name);
                }
            }
        }
        
        let mut image_url = String::new();
        if let Some(i) = img_regex.captures(block) {
            image_url = i[1].to_string();
        }
        
        let published_ts = parse_sr_date(&date_str);
        
        let mut comments = 0;
        if let Some(c) = comments_regex.captures(block) {
            comments = c[1].parse::<i32>().unwrap_or(0);
        }

        let base_name = Some(crate::scanner::base_game_name(&title));

        torrents.push(db::TorrentSR {
            id,
            title,
            url,
            image_url,
            category: cats.join(", "),
            date: date_str,
            fetched_at: now_str.clone(),
            published_ts,
            comments,
            base_name,
            appid: None,
            review_score_desc: None,
            positive_percent: None,
            total_reviews: None,
        });
    }
    torrents
}

#[tauri::command]
fn get_torrents_sr_command(state: tauri::State<'_, db::DbState>) -> Result<Vec<db::TorrentSR>, String> {
    let conn = state.0.lock().unwrap();
    db::get_torrents_sr(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scrape_sr_command(
    app_handle: tauri::AppHandle,
    scrape_state: tauri::State<'_, ScrapeState>,
) -> Result<String, String> {
    use tauri::Emitter;
    use tokio::time::{sleep, Duration};
    use std::process::Command;

    let cancel_flag = scrape_state.is_cancelled.clone();
    cancel_flag.store(false, Ordering::Relaxed);

    let total_pages = 150;
    let mut current_page = 1;
    let mut total_new_added = 0;

    let concurrency = {
        if let Ok(conn) = crate::db::get_connection() {
            let threads_str: String = conn.query_row("SELECT value FROM config WHERE key = 'steam_api_threads'", [], |r| r.get(0)).unwrap_or_else(|_| "5".to_string());
            threads_str.parse::<u32>().unwrap_or(5).max(1).min(20)
        } else {
            5
        }
    };

    let mut consecutive_existing = 0;

    while current_page <= total_pages {
        let event_name = "scrape-progress-sr";
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app_handle.emit(event_name, ScrapeProgress {
                current_page: current_page - 1,
                total_pages,
                message: "抓取任务已被用户取消".to_string(),
                status: "error".to_string(),
            });
            return Ok("抓取任务已被用户取消".to_string());
        }

        let end_page = (current_page + concurrency - 1).min(total_pages);
        let _ = app_handle.emit(event_name, ScrapeProgress {
            current_page: end_page,
            total_pages,
            message: format!("并发获取 Skidrow/Reloaded 第 {} - {} 页数据...", current_page, end_page),
            status: "fetching".to_string(),
        });

        let mut handles = Vec::new();
        for p in current_page..=end_page {
            let url = if p == 1 {
                "https://www.skidrowreloaded.com/".to_string()
            } else {
                format!("https://www.skidrowreloaded.com/page/{}/", p)
            };
            
            let handle = tokio::task::spawn_blocking(move || {
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

                if let Ok(out) = cmd.output() {
                    if out.status.success() {
                        let html = String::from_utf8_lossy(&out.stdout).to_string();
                        return parse_sr_html(&html);
                    }
                }
                Vec::new()
            });
            handles.push(handle);
        }

        let mut all_parsed = Vec::new();
        for h in handles {
            if let Ok(parsed) = h.await {
                all_parsed.extend(parsed);
            }
        }

        if !all_parsed.is_empty() {
            let mut seen = std::collections::HashSet::new();
            all_parsed.retain(|t| seen.insert(t.id.clone()));

            let (new_consecutive, new_added) = tokio::task::spawn_blocking(move || -> Result<(usize, usize), String> {
                let mut conn = crate::db::get_connection().map_err(|e| e.to_string())?;
                let mut local_consecutive = consecutive_existing;
                let mut newly_added = 0;
                
                if let Ok(tx) = conn.transaction() {
                    for t in all_parsed {
                        let exists: bool = tx.query_row(
                            "SELECT EXISTS(SELECT 1 FROM skidrow_reloaded WHERE id = ?)",
                            params![t.id],
                            |row| row.get(0),
                        ).unwrap_or(false);
                        
                        if exists {
                            local_consecutive += 1;
                        } else {
                            local_consecutive = 0;
                        }
                        
                        let base_name = t.base_name.as_deref().unwrap_or("");
                        let rows_affected = tx.execute(
                            "INSERT INTO skidrow_reloaded (id, title, url, image_url, category, date, published_ts, comments, base_name) 
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) 
                             ON CONFLICT(id) DO UPDATE SET 
                                title = ?2, url = ?3, image_url = ?4, category = ?5, date = ?6, published_ts = ?7, comments = ?8, base_name = ?9",
                            params![t.id, t.title, t.url, t.image_url, t.category, t.date, t.published_ts, t.comments, base_name],
                        ).unwrap_or(0);
                        
                        if rows_affected > 0 && !exists {
                            newly_added += 1;
                        }
                    }
                    let _ = tx.commit();
                }
                Ok((local_consecutive, newly_added))
            }).await.map_err(|e| e.to_string())??;
            
            consecutive_existing = new_consecutive;
            total_new_added += new_added;
            
            if consecutive_existing >= 5 {
                break;
            }
        }

        current_page = end_page + 1;
        sleep(Duration::from_millis(200)).await;
    }

    // 阶段二：当游戏清单获取完成后，和游戏索引模块一致，去 Steam 获取评价信息
    let event_name = "scrape-progress-sr";
    if cancel_flag.load(Ordering::Relaxed) {
        let _ = app_handle.emit(event_name, ScrapeProgress {
            current_page: current_page - 1,
            total_pages,
            message: "抓取任务已被用户取消".to_string(),
            status: "error".to_string(),
        });
        return Ok("抓取任务已被用户取消".to_string());
    }

    // 自动用最新清洗规则校准 Skidrow 记录的 base_name
    if let Ok(conn) = crate::db::get_connection() {
        let all_rows: Vec<(String, String, Option<String>)> = {
            if let Ok(mut stmt) = conn.prepare("SELECT id, title, base_name FROM skidrow_reloaded") {
                if let Ok(rows) = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))) {
                    rows.filter_map(|r| r.ok()).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };
        if let Ok(mut update_stmt) = conn.prepare("UPDATE skidrow_reloaded SET base_name = ? WHERE id = ?") {
            let _ = conn.execute_batch("BEGIN TRANSACTION;");
            for (id, title, existing_base) in all_rows {
                let base = crate::scanner::base_game_name(&title);
                if existing_base.as_deref() != Some(&base) {
                    let _ = update_stmt.execute(params![base, id]);
                }
            }
            let _ = conn.execute_batch("COMMIT;");
            let _ = crate::db::sync_game_metadata_covers(&conn);
        }
    }

    // 找出尚未在 steam_cache 中缓存的 Skidrow 游戏 base_name
    let missing_games: Vec<String> = {
        if let Ok(conn) = crate::db::get_connection() {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT sr.base_name 
                 FROM skidrow_reloaded sr
                 LEFT JOIN steam_db.steam_cache s ON sr.base_name = s.base_name
                 WHERE sr.base_name IS NOT NULL 
                   AND TRIM(sr.base_name) != '' 
                   AND (s.base_name IS NULL OR s.appid IS NULL)
                 GROUP BY sr.base_name
                 ORDER BY MAX(sr.published_ts) DESC"
            ) {
                if let Ok(rows) = stmt.query_map([], |r| r.get(0)) {
                    rows.filter_map(|r| r.ok()).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    };

    let total_missing = missing_games.len();
    if total_missing > 0 {
        let app_handle_clone = app_handle.clone();
        let event_name_clone = event_name.to_string();
        let cancel_flag_clone = cancel_flag.clone();

        let was_cancelled = tokio::task::spawn_blocking(move || {
            sync_steam_reviews_blocking(app_handle_clone, event_name_clone, missing_games, cancel_flag_clone)
        })
        .await
        .map_err(|e| e.to_string())??;

        if was_cancelled {
            return Ok("Steam 信息获取已被用户取消".to_string());
        }
    }

    if let Ok(conn) = crate::db::get_connection() {
        let _ = crate::db::sync_game_metadata_covers(&conn);
    }

    let _ = app_handle.emit(event_name, ScrapeProgress {
        current_page: 100,
        total_pages: 100,
        message: "Skidrow/Reloaded 数据与 Steam 评价同步完成！".to_string(),
        status: "complete".to_string(),
    });

    let msg = if total_new_added > 0 {
        format!("更新完成！本次同步新增了 {} 个 Skidrow/Reloaded 游戏发布，并已同步 Steam 评价。", total_new_added)
    } else {
        "更新完成！Skidrow/Reloaded 游戏列表已是最新，并已同步 Steam 评价。".to_string()
    };
    Ok(msg)
}

#[tauri::command]
fn clear_data_1337x() -> Result<(), String> {
    let conn = crate::db::get_connection().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM torrents_1337x", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_data_sr() -> Result<(), String> {
    let conn = crate::db::get_connection().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skidrow_reloaded", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化 Steam 数据库（在独立作用域中，确保连接在 get_connection 之前释放）
            {
                let steam_db_path = crate::steam_service::get_steam_db_path();
                let steam_conn = rusqlite::Connection::open(&steam_db_path)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
                crate::steam_service::init_steam_db(&steam_conn)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            } // steam_conn 在这里被 drop，释放对 steam_data.db 的锁

            // 初始化主数据库连接
            let conn = db::get_connection()
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
            // 注册数据校准状态到 Tauri 托管状态
            let dc_state = data_correction::DataCorrectionState {
                is_running: Arc::new(AtomicBool::new(false)),
                cancel_flag: Arc::new(AtomicBool::new(false)),
            };
            let dc_is_running = Arc::clone(&dc_state.is_running);
            let dc_cancel = Arc::clone(&dc_state.cancel_flag);
            app.manage(dc_state);

            // 异步后台校准历史记录中的 base_name（使用事务批量处理，耗时仅几毫秒，完全不阻塞应用启动主线程）
            std::thread::spawn(|| {
                if let Ok(conn) = db::get_connection() {
                    let sr_records: Vec<(String, String, Option<String>)> = {
                        let stmt = conn.prepare("SELECT id, title, base_name FROM skidrow_reloaded").ok();
                        if let Some(mut stmt) = stmt {
                            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                                .ok()
                                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                                .unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    };
                    if !sr_records.is_empty() {
                        if let Ok(mut update_stmt) = conn.prepare("UPDATE skidrow_reloaded SET base_name = ? WHERE id = ?") {
                            let _ = conn.execute_batch("BEGIN TRANSACTION;");
                            for (id, title, existing_base) in sr_records {
                                let base = crate::scanner::base_game_name(&title);
                                if existing_base.as_deref() != Some(&base) {
                                    let _ = update_stmt.execute(params![base, id]);
                                }
                            }
                            let _ = conn.execute_batch("COMMIT;");
                        }
                    }

                    let torrents_records: Vec<(String, String, Option<String>)> = {
                        let stmt = conn.prepare("SELECT torrent_id, name, base_name FROM torrents_1337x").ok();
                        if let Some(mut stmt) = stmt {
                            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                                .ok()
                                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                                .unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    };
                    if !torrents_records.is_empty() {
                        if let Ok(mut update_stmt) = conn.prepare("UPDATE torrents_1337x SET base_name = ? WHERE torrent_id = ?") {
                            let _ = conn.execute_batch("BEGIN TRANSACTION;");
                            for (id, name, existing_base) in torrents_records {
                                let base = crate::scanner::base_game_name(&name);
                                if existing_base.as_deref() != Some(&base) {
                                    let _ = update_stmt.execute(params![base, id]);
                                }
                            }
                            let _ = conn.execute_batch("COMMIT;");
                        }
                    }
                }
            });

            // 自动检测数据版本：升级版本后在后台静默执行一次历史数据纠正与校准
            let app_handle_migration = app.handle().clone();
            std::thread::spawn(move || {
                // 等待 3 秒，等主界面加载完成后再开始，避免抢占启动资源
                std::thread::sleep(std::time::Duration::from_secs(3));
                let need_run = if let Ok(conn) = db::get_connection() {
                    let ver = db::get_config(&conn, "data_version").ok().flatten();
                    ver.as_deref() != Some("1")
                } else {
                    false
                };
                if need_run {
                    let _ = data_correction::run_data_correction(app_handle_migration, dc_cancel, dc_is_running);
                }
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
            fetch_steam_game_info_command,
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
            steam_api::get_steam_free_games_command,
            get_torrents_sr_command,
            scrape_sr_command,
            clear_data_sr,
            clear_data_1337x,
            data_correction::trigger_data_correction_command,
            data_correction::cancel_data_correction_command,
            data_correction::get_data_correction_status_command
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
