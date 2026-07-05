use crate::db::{Game, SteamCacheEntry, get_scan_paths, get_steam_cache, get_config, insert_steam_cache_entry, save_scanned_games, insert_scan_history};
use reqwest::blocking::Client;
use regex::Regex;
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use rayon::prelude::*;

lazy_static::lazy_static! {
    static ref DIVIDERS_RE: Regex = Regex::new(r"[\._\-/:;]").unwrap();
    static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
    static ref BRACKETS_RE: Regex = Regex::new(r"[\[\]\(\)]").unwrap();
    static ref VERSION_RE: Regex = Regex::new(r"\bv\s?\d+[\d\s\.]*").unwrap();
    static ref BUILD_RE: Regex = Regex::new(r"\bbuild\s?\d+").unwrap();
    static ref YEAR_RE: Regex = Regex::new(r"\b\d{4}\b").unwrap();
}

pub fn clean_name(name: &str) -> String {
    let mut clean = name.to_lowercase();
    if clean.ends_with(".iso") {
        clean.truncate(clean.len() - 4);
    }
    clean = clean.replace('\'', "");
    clean = DIVIDERS_RE.replace_all(&clean, " ").into_owned();
    clean = SPACES_RE.replace_all(&clean, " ").into_owned();
    clean.trim().to_string()
}

pub fn base_game_name(name: &str) -> String {
    let mut clean = name.to_lowercase();
    if clean.ends_with(".iso") {
        clean.truncate(clean.len() - 4);
    }
    if clean.starts_with("pcgame-") {
        clean = clean[7..].to_string();
    }
    
    clean = BRACKETS_RE.replace_all(&clean, " ").into_owned();
    clean = clean.replace('\'', "");
    clean = DIVIDERS_RE.replace_all(&clean, " ").into_owned();
    
    let repack_tags = vec![
        "fitgirl repack", "fitgirl monkey repack", "decepticon repack", "dodi repack",
        "rune", "tenoke", "razor1911", "flt", "voices38", "cpy", "empress", "codex",
        "skidrow", "plaza", "hoodlum", "dinobytes", "unleashed", "delight", "insaneramzes", "p2p",
        "betav1.2.readnfo-mkdev", "read nfo", "readnfo", "proper", "repack", "pre-installed", "cracked",
        "reloaded", "rip", "unlocked", "multi\\s?\\d+", "deluxe\\s+edition", "ultimate\\s+edition", "gold\\s+edition",
        "complete\\s+edition", "director[s\\s\\x27]+cut", "game\\s+of\\s+the\\s+year\\s+edition", "goty", "remastered",
        "definitive\\s+edition", "enhanced\\s+edition", "special\\s+edition", "xxl\\s+edition", "legendary\\s+edition",
        "anniversary\\s+edition", "collector[s\\s\\x27]+edition", "limited\\s+edition", "day\\s+one\\s+edition",
        "standard\\s+edition", "hd\\s+edition", "classic\\s+edition", "premium\\s+edition", "hrdc",
        "elamigos", "gog", "3dm", "ali213", "canek77", "wanterlude", "decepticon", "fitgirl", "dodi",
        "early\\s+access", "portable", "dlc\\s+unlocker", "incl\\s+dlc", "with\\s+update", "with\\s+up\\d+",
        "chs", "cht", "complete\\s+bundle", "bundle", "collection", "steam", "gog\\s+edition", "by\\s+\\w+",
        "digital\\s+deluxe\\s+edition", "digital\\s+edition"
    ];
    
    for tag in repack_tags {
        let pattern = format!(r"\b{}\b", tag);
        if let Ok(re) = Regex::new(&pattern) {
            clean = re.replace_all(&clean, "").into_owned();
        }
    }
    
    clean = VERSION_RE.replace_all(&clean, "").into_owned();
    clean = BUILD_RE.replace_all(&clean, "").into_owned();
    clean = YEAR_RE.replace_all(&clean, "").into_owned();
    clean = SPACES_RE.replace_all(&clean, " ").into_owned();
    
    clean.trim().to_string()
}

use jwalk::WalkDir;

fn get_dir_size<P: AsRef<Path>>(path: P) -> u64 {
    WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} TB", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(serde::Serialize, Clone)]
struct ProgressEvent {
    step: String,
    message: String,
    current: usize,
    total: usize,
}

pub fn run_scan(app_handle: AppHandle, cancel_flag: Arc<AtomicBool>) -> Result<(), String> {
    // 扫描开始时重置取消标志
    cancel_flag.store(false, Ordering::Relaxed);

    let started_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let _ = app_handle.emit(
        "scan-progress",
        ProgressEvent {
            step: "start".to_string(),
            message: "正在连接数据库...".to_string(),
            current: 0,
            total: 100,
        },
    );

    let db_path = crate::db::get_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let scan_paths = get_scan_paths(&conn).map_err(|e| e.to_string())?;
    let cache = get_steam_cache(&conn).map_err(|e| e.to_string())?;

    // 从数据库 config 表读取 exclude_folders 配置
    let exclude_folders_str = get_config(&conn, "exclude_folders")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let exclude_folders: Vec<String> = exclude_folders_str
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();

    // 从数据库 config 表读取 steam_api_delay_ms 配置
    let steam_api_delay_ms: u64 = get_config(&conn, "steam_api_delay_ms")
        .map_err(|e| e.to_string())?
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);

    // 从数据库 config 表读取 steam_api_threads 配置
    let steam_api_threads: usize = get_config(&conn, "steam_api_threads")
        .map_err(|e| e.to_string())?
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let language = get_config(&conn, "language")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "schinese".to_string());

    let _ = app_handle.emit(
        "scan-progress",
        ProgressEvent {
            step: "scan-directories".to_string(),
            message: "开始扫描物理目录...".to_string(),
            current: 0,
            total: 100,
        },
    );

    let mut raw_scanned = Vec::new();

    for (p_idx, path_str) in scan_paths.iter().enumerate() {
        // 检查取消标志
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("扫描已被用户取消".to_string());
        }

        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let _ = app_handle.emit(
            "scan-progress",
            ProgressEvent {
                step: "scan-directories".to_string(),
                message: format!("正在扫描盘符/路径: {}...", path_str),
                current: p_idx,
                total: scan_paths.len(),
            },
        );

        if let Ok(entries) = fs::read_dir(path) {
            let valid_entries: Vec<_> = entries.flatten().collect();

            let games: Vec<Game> = valid_entries.into_par_iter().filter_map(|entry| {
                // 检查取消标志
                if cancel_flag.load(Ordering::Relaxed) {
                    return None;
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => return None,
                };

                let name = entry.file_name().to_string_lossy().into_owned();
                let lower_name = name.to_lowercase();

                // 跳过系统/隐藏文件夹
                if name.starts_with('$') || name.starts_with('.') {
                    return None;
                }

                // 使用从 config 表读取的 exclude_folders 统一过滤所有扫描路径下的非游戏系统文件夹
                if exclude_folders.contains(&lower_name) {
                    return None;
                }

                let is_dir = metadata.is_dir();
                let is_iso = entry.path().extension().map_or(false, |ext| ext == "iso");

                if is_dir || is_iso {
                    let r#type = if is_dir { "Directory".to_string() } else { "ISO".to_string() };
                    let full_path = entry.path().to_string_lossy().into_owned();

                    let created_str = if let Ok(created_time) = metadata.created() {
                        let datetime: chrono::DateTime<chrono::Local> = created_time.into();
                        datetime.format("%Y-%m-%d %H:%M").to_string()
                    } else if let Ok(modified_time) = metadata.modified() {
                        let datetime: chrono::DateTime<chrono::Local> = modified_time.into();
                        datetime.format("%Y-%m-%d %H:%M").to_string()
                    } else {
                        "未知时间".to_string()
                    };

                    // 计算大小
                    let size_bytes = if is_dir {
                        get_dir_size(&entry.path())
                    } else {
                        metadata.len()
                    };

                    let size_str = format_size(size_bytes);
                    let clean = clean_name(&name);
                    let base = base_game_name(&name);

                    Some(Game {
                        id: None,
                        original_name: name,
                        clean_name: clean,
                        base_name: base,
                        r#type,
                        source_path: path_str.clone(),
                        full_path,
                        size: size_str,
                        size_bytes: size_bytes as i64,
                        created: created_str,
                        is_exact_dup: false,
                        is_version_dup: false,
                        is_representative: false,
                        appid: None,
                        name: None,
                        local_cover: None,
                        review_score_desc: None,
                        positive_percent: None,
                        total_reviews: None,
                        release_date: None,
                        genres: None,
                    })
                } else {
                    None
                }
            }).collect();

            if cancel_flag.load(Ordering::Relaxed) {
                return Err("扫描已被用户取消".to_string());
            }

            raw_scanned.extend(games);
        }
    }

    let _ = app_handle.emit(
        "scan-progress",
        ProgressEvent {
            step: "scan-directories".to_string(),
            message: format!("扫描完成。共检索到 {} 个文件对象。", raw_scanned.len()),
            current: raw_scanned.len(),
            total: raw_scanned.len(),
        },
    );

    // 内存中的分组逻辑
    let mut base_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, game) in raw_scanned.iter().enumerate() {
        base_groups.entry(game.base_name.clone()).or_insert_with(Vec::new).push(idx);
    }

    for (_base_name, idx_list) in base_groups.iter() {
        let is_exact;
        let is_version;

        if idx_list.len() > 1 {
            // 在此 BaseName 组内按 CleanName 分组
            let mut clean_groups: HashMap<String, Vec<usize>> = HashMap::new();
            for &idx in idx_list {
                clean_groups.entry(raw_scanned[idx].clean_name.clone()).or_insert_with(Vec::new).push(idx);
            }

            if clean_groups.len() > 1 {
                is_version = true;
                is_exact = false;
            } else {
                is_version = false;
                is_exact = true;
            }
        } else {
            is_version = false;
            is_exact = false;
        }

        // 设置重复标记
        for &idx in idx_list {
            raw_scanned[idx].is_exact_dup = is_exact;
            raw_scanned[idx].is_version_dup = is_version;
        }

        // 选择代表（最短 original_name 长度）
        let mut best_idx = idx_list[0];
        let mut min_len = raw_scanned[best_idx].original_name.len();
        for &idx in idx_list.iter().skip(1) {
            let len = raw_scanned[idx].original_name.len();
            if len < min_len {
                min_len = len;
                best_idx = idx;
            }
        }
        raw_scanned[best_idx].is_representative = true;
    }

    // 识别需要查询 Steam 缓存的游戏
    let mut representatives = Vec::new();
    for game in &raw_scanned {
        if game.is_representative {
            representatives.push(game.base_name.clone());
        }
    }
    representatives.sort();
    representatives.dedup();

    let new_games: Vec<String> = representatives
        .into_iter()
        .filter(|base| !cache.contains_key(base))
        .collect();

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    // 在 exe 所在目录创建 covers 文件夹
    let mut covers_dir = std::env::current_exe().unwrap_or_default();
    covers_dir.pop(); // 移除 exe 文件名，保留目录
    covers_dir.push("covers");
    let _ = fs::create_dir_all(&covers_dir);

    let mut new_steam_entries: i64 = 0;

    if !new_games.is_empty() {
        let total_new = new_games.len();
        
        let new_games_arc = Arc::new(std::sync::Mutex::new(new_games.clone().into_iter().enumerate()));
        let (tx, rx) = std::sync::mpsc::channel();
        let covers_dir_arc = Arc::new(covers_dir);
        
        for _ in 0..steam_api_threads {
            let tx_clone = tx.clone();
            let games_clone = Arc::clone(&new_games_arc);
            let cancel_clone = Arc::clone(&cancel_flag);
            let covers_dir_clone = Arc::clone(&covers_dir_arc);
            let client_clone = client.clone();
            let lang = language.clone();
            
            std::thread::spawn(move || {
                loop {
                    if cancel_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let next_game = {
                        let mut iter = games_clone.lock().unwrap();
                        iter.next()
                    };
                    
                    match next_game {
                        Some((g_idx, base_name)) => {
                            if steam_api_delay_ms > 0 {
                                std::thread::sleep(Duration::from_millis(steam_api_delay_ms));
                            }

                            let encoded = urlencoding::encode(&base_name);
                            let search_url = format!("https://store.steampowered.com/api/storesearch/?term={}&l={}&cc=CN", encoded, lang);

                            let mut entry = SteamCacheEntry {
                                base_name: base_name.clone(),
                                appid: None,
                                name: None,
                                local_cover: None,
                                review_score_desc: None,
                                positive_percent: None,
                                total_reviews: None,
                                release_date: None,
                                last_updated: Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()),
                                genres: None,
                            };

                            if let Ok(res) = client_clone.get(&search_url).send() {
                                if let Ok(res_str) = res.text() {
                                    if let Ok(res_json) = serde_json::from_str::<Value>(&res_str) {
                                        if let Some(items) = res_json.get("items").and_then(|i| i.as_array()) {
                                            if !items.is_empty() {
                                                let mut best_match = None;
                                                for item in items {
                                                    if item.get("type").and_then(|t| t.as_str()) == Some("app") {
                                                        let name_str = item.get("name").and_then(|n| n.as_str()).unwrap_or("").to_lowercase();
                                                        // Fallback matching to exact english name if present, otherwise steam's top search is usually accurate
                                                        if name_str == base_name {
                                                            best_match = Some(item);
                                                            break;
                                                        }
                                                    }
                                                }
                                                if best_match.is_none() {
                                                    for item in items {
                                                        if item.get("type").and_then(|t| t.as_str()) == Some("app") {
                                                            best_match = Some(item);
                                                            break;
                                                        }
                                                    }
                                                }
                                                if best_match.is_none() {
                                                    best_match = Some(&items[0]);
                                                }

                                                if let Some(best) = best_match {
                                                    if let Some(app_id) = best.get("id").and_then(|id| id.as_i64()) {
                                                        entry.appid = Some(app_id);
                                                        entry.name = best.get("name").and_then(|n| n.as_str()).map(String::from);
                                                        
                                                        let cover_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg", app_id);
                                                        let cover_filename = format!("{}.jpg", app_id);
                                                        
                                                        let local_path = covers_dir_clone.join(&cover_filename);

                                                        let mut download_success = false;
                                                        if let Ok(head_res) = client_clone.head(&cover_url).send() {
                                                            if head_res.status().is_success() {
                                                                if let Ok(img_res) = client_clone.get(&cover_url).send() {
                                                                    if img_res.status().is_success() {
                                                                        if let Ok(img_bytes) = img_res.bytes() {
                                                                            let _ = fs::write(&local_path, &img_bytes);
                                                                            download_success = true;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if !download_success {
                                                            let fallback_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/capsule_616x353.jpg", app_id);
                                                            if let Ok(img_res) = client_clone.get(&fallback_url).send() {
                                                                if img_res.status().is_success() {
                                                                    if let Ok(img_bytes) = img_res.bytes() {
                                                                        let _ = fs::write(&local_path, &img_bytes);
                                                                        download_success = true;
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if download_success {
                                                            entry.local_cover = Some(format!("covers/{}", cover_filename));
                                                        }

                                                        let review_url = format!("https://store.steampowered.com/appreviews/{}?json=1&l={}", app_id, lang);
                                                        if let Ok(review_res) = client_clone.get(&review_url).send().and_then(|r| r.json::<Value>()) {
                                                            if let Some(qs) = review_res.get("query_summary") {
                                                                if let Some(total) = qs.get("total_reviews").and_then(|t| t.as_i64()) {
                                                                    if total > 0 {
                                                                        entry.total_reviews = Some(total);
                                                                        let raw_desc = qs.get("review_score_desc").and_then(|s| s.as_str()).map(String::from);
                                                                        if let Some(desc) = raw_desc {
                                                                            if desc.is_empty() 
                                                                               || desc.contains("篇用户评测") 
                                                                               || desc.contains("user reviews") 
                                                                               || desc.contains("Need more user reviews")
                                                                               || desc.contains("不需要测评") 
                                                                            {
                                                                                entry.review_score_desc = Some("评价不可用".to_string());
                                                                            } else {
                                                                                entry.review_score_desc = Some(desc);
                                                                            }
                                                                        } else {
                                                                            entry.review_score_desc = Some("评价不可用".to_string());
                                                                        }
                                                                        
                                                                        if let Some(positive) = qs.get("total_positive").and_then(|p| p.as_i64()) {
                                                                            let percent = (positive as f64 / total as f64 * 100.0).round() as i64;
                                                                            entry.positive_percent = Some(percent);
                                                                        }
                                                                    } else {
                                                                        entry.review_score_desc = Some("评价不可用".to_string());
                                                                        entry.total_reviews = Some(0);
                                                                        entry.positive_percent = Some(0);
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        let details_url = format!("https://store.steampowered.com/api/appdetails?appids={}&filters=basic,release_date,genres&l={}&cc=CN", app_id, lang);
                                                        if let Ok(details_res) = client_clone.get(&details_url).send().and_then(|r| r.json::<Value>()) {
                                                            if let Some(data) = details_res.get(&app_id.to_string()).and_then(|app| app.get("data")) {
                                                                if let Some(loc_name) = data.get("name").and_then(|n| n.as_str()) {
                                                                    entry.name = Some(loc_name.to_string());
                                                                }
                                                                if let Some(date_val) = data.get("release_date").and_then(|rd| rd.get("date")) {
                                                                    entry.release_date = date_val.as_str().map(String::from);
                                                                }
                                                                if let Some(genres_array) = data.get("genres").and_then(|g| g.as_array()) {
                                                                    let genres_str: Vec<String> = genres_array.iter()
                                                                        .filter_map(|g| g.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()))
                                                                        .collect();
                                                                    if !genres_str.is_empty() {
                                                                        entry.genres = Some(genres_str.join(", "));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            let _ = tx_clone.send((g_idx, base_name, entry));
                        }
                        None => break,
                    }
                }
            });
        }
        
        drop(tx); // drop the original sender
        
        let mut processed = 0;
        for (_g_idx, base_name, entry) in rx {
            if cancel_flag.load(Ordering::Relaxed) {
                let completed_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = insert_scan_history(
                    &conn,
                    &started_at,
                    &completed_at,
                    raw_scanned.len() as i64,
                    new_games.len() as i64,
                    new_steam_entries,
                    "cancelled",
                );
                return Err("扫描已被用户取消".to_string());
            }

            processed += 1;
            let _ = app_handle.emit(
                "scan-progress",
                ProgressEvent {
                    step: "query-steam".to_string(),
                    message: format!("正在向 Steam API 检索新游戏 ({} / {}): {}", processed, total_new, base_name),
                    current: processed,
                    total: total_new,
                },
            );

            let _ = insert_steam_cache_entry(&conn, &entry);
            new_steam_entries += 1;
        }
    }

    let _ = app_handle.emit(
        "scan-progress",
        ProgressEvent {
            step: "saving".to_string(),
            message: "正在保存扫描结果到本地数据库...".to_string(),
            current: 95,
            total: 100,
        },
    );

    // 将所有扫描结果保存到 games 表
    save_scanned_games(&conn, &raw_scanned).map_err(|e| e.to_string())?;

    // 记录扫描历史
    let completed_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = insert_scan_history(
        &conn,
        &started_at,
        &completed_at,
        raw_scanned.len() as i64,
        new_games.len() as i64,
        new_steam_entries,
        "completed",
    );

    // 更新 last_scan_time 配置
    let _ = crate::db::set_config(&conn, "last_scan_time", &completed_at);

    let _ = app_handle.emit(
        "scan-progress",
        ProgressEvent {
            step: "complete".to_string(),
            message: "所有扫描及入库操作已顺利完成！".to_string(),
            current: 100,
            total: 100,
        },
    );

    Ok(())
}
