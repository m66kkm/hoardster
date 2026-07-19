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
        let db_path = db::get_db_path();
        if let Ok(conn) = Connection::open(db_path) {
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

            let db_path = db::get_db_path();
            let session_time_clone = session_time.clone();
            let (new_consecutive, new_added) = tokio::task::spawn_blocking(move || -> Result<(usize, usize), String> {
                let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;
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
                        
                        let rows_affected = tx.execute(
                            "INSERT OR IGNORE INTO torrents_1337x (
                                torrent_id, name, url, seeds, leeches, date, size, uploader, uploader_url, fetched_at, published_ts
                             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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

    Ok(format!("更新完成！本次同步新增了 {} 个游戏种子。", total_new_added))
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
        let db_path = db::get_db_path();
        if let Ok(conn) = Connection::open(db_path) {
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

            let db_path = db::get_db_path();
            let (new_consecutive, new_added) = tokio::task::spawn_blocking(move || -> Result<(usize, usize), String> {
                let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;
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
                        
                        let rows_affected = tx.execute(
                            "INSERT INTO skidrow_reloaded (id, title, url, image_url, category, date, published_ts, comments) 
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) 
                             ON CONFLICT(id) DO UPDATE SET 
                                title = ?2, url = ?3, image_url = ?4, category = ?5, date = ?6, published_ts = ?7, comments = ?8",
                            params![t.id, t.title, t.url, t.image_url, t.category, t.date, t.published_ts, t.comments],
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

    Ok(format!("更新完成！本次同步新增了 {} 个 Skidrow/Reloaded 游戏发布。", total_new_added))
}

#[tauri::command]
fn clear_data_1337x() -> Result<(), String> {
    let db_path = db::get_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM torrents_1337x", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_data_sr() -> Result<(), String> {
    let db_path = db::get_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skidrow_reloaded", []).map_err(|e| e.to_string())?;
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
            steam_api::get_steam_free_games_command,
            get_torrents_sr_command,
            scrape_sr_command,
            clear_data_sr,
            clear_data_1337x
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
