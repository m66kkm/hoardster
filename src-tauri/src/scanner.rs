use crate::db::{Game, get_scan_paths, get_steam_cache, get_config, insert_steam_cache_entry, save_scanned_games, insert_scan_history};
use reqwest::blocking::Client;
use regex::Regex;
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
    static ref BRACKET_CONTENT_RE: Regex = Regex::new(r"\s*[\[\(].*?[\]\)]").unwrap();
    static ref BRACKETS_RE: Regex = Regex::new(r"[\[\]\(\)]").unwrap();
    static ref SCENE_GROUP_RE: Regex = Regex::new(r"(?i)-([a-z0-9_]+)$").unwrap();
    static ref VERSION_DOTTED_RE: Regex = Regex::new(r"(?i)\bv\s?\d+[\w\.]*\b").unwrap();
    static ref SEMVER_DOTTED_RE: Regex = Regex::new(r"\b\d+\.\d+\.\d+[\w\.]*\b").unwrap();
    static ref BUILD_RE: Regex = Regex::new(r"(?i)\bbuild[\s\.]*\d+\b").unwrap();
    static ref VERSION_SPACED_RE: Regex = Regex::new(r"(?i)\bv\s?\d+(\s+\d+)*\b").unwrap();
    static ref YEAR_RE: Regex = Regex::new(r"\b(199\d|20[0-2]\d|2030)\b").unwrap();
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
    
    // 1. 剥离末尾的发布组后缀 (如 -P2P, -RUNE, -TENOKE, -0xZeOn, -HYPERVISOR)
    clean = SCENE_GROUP_RE.replace(&clean, "").into_owned();

    // 2. 剥离中括号和小括号及其中的内容（例如 [FitGirl Repack], (v1.0.5 Release + Bonus OST...)）
    let stripped = BRACKET_CONTENT_RE.replace_all(&clean, " ").into_owned();
    if !stripped.trim().is_empty() {
        clean = stripped;
    } else {
        clean = BRACKETS_RE.replace_all(&clean, " ").into_owned();
    }

    // 3. 在将点号(.)等分隔符替换为空格之前，优先剥离带点号的版本号与构建号！
    // 避免 v1.0.12.0 被点号分隔符提前切碎为 "v1 0 12 0"，从而留下 "0 12 0" 等脏后缀
    clean = VERSION_DOTTED_RE.replace_all(&clean, " ").into_owned();
    clean = SEMVER_DOTTED_RE.replace_all(&clean, " ").into_owned();
    clean = BUILD_RE.replace_all(&clean, " ").into_owned();

    clean = clean.replace('\'', "");
    clean = DIVIDERS_RE.replace_all(&clean, " ").into_owned();
    
    let repack_tags = vec![
        "digital\\s+deluxe\\s+edition", "digital\\s+edition",
        "game\\s+of\\s+the\\s+year\\s+edition", "goty",
        "fitgirl repack", "fitgirl monkey repack", "decepticon repack", "dodi repack",
        "rune", "tenoke", "razor1911", "flt", "voices38", "cpy", "empress", "codex",
        "skidrow", "plaza", "hoodlum", "dinobytes", "unleashed", "delight", "insaneramzes", "p2p",
        "betav1.2.readnfo-mkdev", "read nfo", "readnfo", "proper", "repack", "pre-installed", "cracked",
        "reloaded", "rip", "unlocked", "multi\\s?\\d+", "deluxe\\s+edition", "ultimate\\s+edition", "gold\\s+edition",
        "complete\\s+edition", "director[s\\s\\x27]+cut", "remastered",
        "definitive\\s+edition", "enhanced\\s+edition", "special\\s+edition", "xxl\\s+edition", "legendary\\s+edition",
        "anniversary\\s+edition", "collector[s\\s\\x27]+edition", "limited\\s+edition", "day\\s+one\\s+edition",
        "standard\\s+edition", "hd\\s+edition", "classic\\s+edition", "premium\\s+edition", "hrdc",
        "elamigos", "gog", "3dm", "ali213", "canek77", "wanterlude", "decepticon", "fitgirl", "dodi",
        "early\\s+access", "portable", "dlc\\s+unlocker", "incl\\s+dlc", "with\\s+update", "with\\s+up\\d+",
        "chs", "cht", "complete\\s+bundle", "bundle", "collection", "steam", "gog\\s+edition", "by\\s+\\w+",
        "tinyiso", "doge", "kaos", "i_know", "anomaly", "simplex", "chronos", "goldberg", "update", "dlc",
        "0xzeon", "hypervisor"
    ];
    
    for tag in repack_tags {
        let pattern = format!(r"\b{}\b", tag);
        if let Ok(re) = Regex::new(&pattern) {
            clean = re.replace_all(&clean, "").into_owned();
        }
    }
    
    // 兜底剥离以空格形式残留的版本号（如 v 1 0 12）
    clean = VERSION_SPACED_RE.replace_all(&clean, "").into_owned();

    // 剥离常见的发行年份标签（1990-2030），且不误伤如 Cyberpunk 2077 等游戏名
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

fn is_installed_game<P: AsRef<Path>>(path: P) -> bool {
    WalkDir::new(path)
        .skip_hidden(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| {
            if e.file_type().is_dir() {
                return false;
            }
            if let Some(ext) = e.path().extension() {
                if ext.eq_ignore_ascii_case("exe") {
                    let file_name = e.file_name().to_string_lossy().to_lowercase();
                    if !file_name.contains("setup") 
                        && !file_name.contains("install") 
                        && !file_name.contains("autorun") 
                        && !file_name.contains("unins") {
                        return true;
                    }
                }
            }
            false
        })
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

    let conn = crate::db::get_connection().map_err(|e| e.to_string())?;

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
                    let r#type = if is_iso {
                        "ISO".to_string()
                    } else if is_installed_game(entry.path()) {
                        "Installed".to_string()
                    } else {
                        "Archive".to_string()
                    };
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
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
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

                            let maybe_entry = crate::steam_service::fetch_steam_game_info(&client_clone, &base_name, &lang);
                            let entry = if let Some(mut entry) = maybe_entry {
                                // 封面下载逻辑
                                if let Some(cover_url) = entry.local_cover.clone() {
                                    if let Some(app_id) = entry.appid {
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
                                        } else {
                                            entry.local_cover = None;
                                        }
                                    }
                                }
                                Some(entry)
                            } else {
                                None
                            };

                            let _ = tx_clone.send((g_idx, base_name, entry));
                        }
                        None => break,
                    }
                }
            });
        }
        
        drop(tx); // drop the original sender
        
        let mut processed = 0;
        for (_g_idx, base_name, entry_opt) in rx {
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

            if let Some(entry) = entry_opt {
                if entry.appid.is_some() {
                    // Retry up to 3 times on database lock errors
                    for attempt in 0..3 {
                        match insert_steam_cache_entry(&conn, &entry) {
                            Ok(_) => break,
                            Err(e) => {
                                let err_str = e.to_string();
                                if err_str.contains("locked") && attempt < 2 {
                                    println!("Steam cache insert locked, retrying ({}/3): {}", attempt + 1, base_name);
                                    std::thread::sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                                } else {
                                    println!("Error inserting steam cache for {}: {}", base_name, e);
                                    break;
                                }
                            }
                        }
                    }
                    // Update raw_scanned so the first scan has the steam data
                    raw_scanned[_g_idx].appid = entry.appid;
                    raw_scanned[_g_idx].name = entry.name.clone().or(raw_scanned[_g_idx].name.clone());
                    raw_scanned[_g_idx].local_cover = entry.local_cover.clone();
                    raw_scanned[_g_idx].review_score_desc = entry.review_score_desc;
                    raw_scanned[_g_idx].positive_percent = entry.positive_percent.map(|x| x as i64);
                    raw_scanned[_g_idx].total_reviews = entry.total_reviews.map(|x| x as i64);
                    raw_scanned[_g_idx].release_date = entry.release_date.clone();
                    raw_scanned[_g_idx].genres = entry.genres.clone();
                    
                    new_steam_entries += 1;
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_game_name_beast_of_reincarnation() {
        assert_eq!(
            base_game_name("Beast of Reincarnation v1.0.12.0-P2P"),
            "beast of reincarnation"
        );
        assert_eq!(
            base_game_name("Beast.Of.Reincarnation.v1.0.11.0.REPACK-KaOs"),
            "beast of reincarnation"
        );
        assert_eq!(
            base_game_name("Beast of Reincarnation-RUNE"),
            "beast of reincarnation"
        );
        assert_eq!(
            base_game_name("PROHIBEAST v1.0.5-P2P"),
            "prohibeast"
        );
        assert_eq!(
            base_game_name("Cyberpunk 2077 v2.13-GOG"),
            "cyberpunk 2077"
        );
        assert_eq!(
            base_game_name("Black Myth: Wukong v1.0.8.14823"),
            "black myth wukong"
        );
    }

    #[test]
    #[ignore]
    fn test_user_30_games() {
        let titles = vec![
            "PATLABOR the Case Files-GoldBerg",
            "Corsair Cove v1.1.7.246856-P2P",
            "Beast of Reincarnation v1.0.12.0-P2P",
            "Blackwood v20260917-P2P",
            "The Crust v1.0.11-P2P",
            "Valheim v1.0.15-P2P",
            "Tabletop Tavern v1.9.15-P2P",
            "Drill Core v1.261-P2P",
            "Backyard Baseball v1.1.0.19.1-P2P",
            "Astral Ascent v2.6.4-P2P",
            "Artis Impact v1.20-P2P",
            "Stolen Realm v1.3.1-P2P",
            "Reus 2 Jurassic-RUNE",
            "Scarlet Deer Inn v1.025-P2P",
            "Ostranauts v1.0.1.4-P2P",
            "Legends of Dragaea Idle Dungeons v2.1.2c-P2P",
            "Gurei v1.081-P2P",
            "S.T.A.L.K.E.R 2 Heart of Chornobyl v2.0.6-P2P",
            "The Walking Dead Streets of Survival-RUNE",
            "Dune Awakening-RUNE",
            "Police Chief Simulator-TENOKE",
            "lilys world XD-TENOKE",
            "The Guild 1 Remake Europa 1410 Early Access",
            "Reincarnation Insurance Program-P2P",
            "Nioh 3 v2.02-P2P",
            "Conan Exiles Enhanced Complete Edition v2.2.0-P2P",
            "Grand Theft Auto V Enhanced v1.0.1158.16-P2P",
            "007 First Light v1.2.1-P2P",
            "Company of Heroes 3 v2.5.6.50313-P2P",
            "Le Mans Ultimate v1.4.1.5-P2P",
        ];

        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        println!("\n=== 测试 30 个游戏的名称清洗与 Steam 获取结果 ===");
        for (idx, title) in titles.iter().enumerate() {
            let base = base_game_name(title);
            let (maybe_entry, got_403) = crate::steam_service::fetch_steam_game_info_ext(&client, &base, "schinese");
            match maybe_entry {
                Some(entry) => {
                    let appid_str = entry.appid.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string());
                    let name_str = entry.name.unwrap_or_else(|| "None".to_string());
                    let reviews_str = match (entry.positive_percent, entry.total_reviews, entry.review_score_desc) {
                        (Some(pct), Some(tot), Some(desc)) => format!("👍 {}% ({}篇, 评分档位:{})", pct, tot, desc),
                        (_, Some(tot), _) => format!("评价不足 ({}篇)", tot),
                        _ => "暂无评价".to_string(),
                    };
                    println!("[{:02}] 成功: {} => base: \"{}\" | AppID: {} | 游戏: {} | 评价: {}", idx + 1, title, base, appid_str, name_str, reviews_str);
                }
                None => {
                    println!("[{:02}] 未找到: {} => base: \"{}\" | 403限流: {}", idx + 1, title, base, got_403);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(350));
        }
    }
}
