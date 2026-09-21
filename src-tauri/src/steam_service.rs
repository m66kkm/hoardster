use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::collections::HashMap;
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SteamCacheEntry {
    pub base_name: String,
    pub appid: Option<i64>,
    pub name: Option<String>,
    pub local_cover: Option<String>,
    pub review_score_desc: Option<i32>,
    pub positive_percent: Option<i64>,
    pub total_reviews: Option<i64>,
    pub recent_review_score_desc: Option<i32>,
    pub recent_positive_percent: Option<i64>,
    pub release_date: Option<String>,
    pub last_updated: Option<String>,
    pub genres: Option<String>,
}

pub fn get_steam_db_path() -> PathBuf {
    let mut exe_path = std::env::current_exe().unwrap_or_default();
    exe_path.pop(); // 移除 exe 文件名，保留目录
    exe_path.push("steam_data.db");
    exe_path
}

pub fn init_steam_db(conn: &Connection) -> Result<()> {
    // Set busy_timeout FIRST (WAL switch needs exclusive lock, must be able to wait)
    conn.pragma_update(None, "busy_timeout", 10000).ok();
    // Enable WAL mode on steam_data.db itself (non-fatal: if stale locks prevent it, DELETE mode still works)
    conn.pragma_update(None, "journal_mode", "WAL").ok();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_cache (
            base_name TEXT PRIMARY KEY,
            appid INTEGER,
            name TEXT,
            local_cover TEXT,
            review_score_desc INTEGER,
            positive_percent INTEGER,
            total_reviews INTEGER,
            recent_review_score_desc INTEGER,
            recent_positive_percent INTEGER,
            release_date TEXT,
            last_updated TEXT,
            genres TEXT
        )",
        [],
    )?;
    // Migration: add recent review columns if they don't exist (for existing databases)
    conn.execute("ALTER TABLE steam_cache ADD COLUMN recent_review_score_desc INTEGER", []).ok();
    conn.execute("ALTER TABLE steam_cache ADD COLUMN recent_positive_percent INTEGER", []).ok();
    Ok(())
}

pub fn get_steam_cache(conn: &Connection) -> Result<HashMap<String, SteamCacheEntry>> {
    let mut stmt = conn.prepare("SELECT base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, recent_review_score_desc, recent_positive_percent, release_date, last_updated, genres FROM steam_cache")?;
    let cache_iter = stmt.query_map([], |row| {
        Ok(SteamCacheEntry {
            base_name: row.get(0)?,
            appid: row.get(1)?,
            name: row.get(2)?,
            local_cover: row.get(3)?,
            review_score_desc: row.get(4)?,
            positive_percent: row.get(5)?,
            total_reviews: row.get(6)?,
            recent_review_score_desc: row.get(7)?,
            recent_positive_percent: row.get(8)?,
            release_date: row.get(9)?,
            last_updated: row.get(10)?,
            genres: row.get(11)?,
        })
    })?;

    let mut map = HashMap::new();
    for entry in cache_iter {
        let entry = entry?;
        map.insert(entry.base_name.clone(), entry);
    }
    Ok(map)
}

pub fn clear_steam_cache(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM steam_cache", [])?;
    Ok(())
}

pub fn insert_steam_cache_entry(conn: &Connection, entry: &SteamCacheEntry) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO steam_cache (base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, recent_review_score_desc, recent_positive_percent, release_date, last_updated, genres)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            entry.base_name,
            entry.appid,
            entry.name,
            entry.local_cover,
            entry.review_score_desc,
            entry.positive_percent,
            entry.total_reviews,
            entry.recent_review_score_desc,
            entry.recent_positive_percent,
            entry.release_date,
            entry.last_updated,
            entry.genres
        ],
    )?;
    Ok(())
}

/// 独立的 Steam 游戏信息获取服务接口（返回结果与是否触发403/429频控标识）
pub fn fetch_steam_game_info_ext(client: &Client, base_name: &str, lang: &str) -> (Option<SteamCacheEntry>, bool) {
    let mut got_403 = false;
    let encoded = urlencoding::encode(base_name);
    let search_url = format!("https://store.steampowered.com/api/storesearch/?term={}&l={}&cc=CN", encoded, lang);

    let mut entry = SteamCacheEntry {
        base_name: base_name.to_string(),
        appid: None,
        name: None,
        local_cover: None,
        review_score_desc: None,
        positive_percent: None,
        total_reviews: None,
        recent_review_score_desc: None,
        recent_positive_percent: None,
        release_date: None,
        last_updated: Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()),
        genres: None,
    };

    let mut app_info: Option<(i64, String, String)> = None; // (app_id, name, tiny_image)

    // 1. 优先尝试 Steam 商店搜索接口
    if let Ok(res) = client.get(&search_url).send() {
        if res.status() == reqwest::StatusCode::FORBIDDEN || res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            got_403 = true;
        } else if res.status().is_success() {
            if let Ok(res_str) = res.text() {
                if let Ok(res_json) = serde_json::from_str::<Value>(&res_str) {
                    if let Some(items) = res_json.get("items").and_then(|i| i.as_array()) {
                        if !items.is_empty() {
                            let mut best_match = None;
                            for item in items {
                                if item.get("type").and_then(|t| t.as_str()) == Some("app") {
                                    let name_str = item.get("name").and_then(|n| n.as_str()).unwrap_or("").to_lowercase();
                                    if name_str == base_name.to_lowercase() {
                                        best_match = Some(item);
                                        break;
                                    }
                                }
                            }
                            if best_match.is_none() {
                                best_match = items.iter().find(|i| i.get("type").and_then(|t| t.as_str()) == Some("app")).or_else(|| items.first());
                            }

                            if let Some(best) = best_match {
                                if let Some(app_id) = best.get("id").and_then(|id| id.as_i64()) {
                                    let name = best.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                                    let tiny = best.get("tiny_image").and_then(|t| t.as_str()).unwrap_or("").to_string();
                                    app_info = Some((app_id, name, tiny));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. 如果商店搜索受限 (如 403 限流) 或未命中，回退到 Steam Community 搜索接口
    if app_info.is_none() {
        let comm_url = format!("https://steamcommunity.com/actions/SearchApps/{}", encoded);
        if let Ok(comm_res) = client.get(&comm_url).send() {
            if comm_res.status() == reqwest::StatusCode::FORBIDDEN || comm_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                got_403 = true;
            } else if comm_res.status().is_success() {
                if let Ok(comm_json) = comm_res.json::<Value>() {
                    if let Some(arr) = comm_json.as_array() {
                        if !arr.is_empty() {
                            let mut best = None;
                            for item in arr {
                                let name_str = item.get("name").and_then(|n| n.as_str()).unwrap_or("").to_lowercase();
                                if name_str == base_name.to_lowercase() {
                                    best = Some(item);
                                    break;
                                }
                            }
                            if best.is_none() {
                                best = arr.first();
                            }
                            if let Some(item) = best {
                                if let Some(id_str) = item.get("appid").and_then(|i| i.as_str()) {
                                    if let Ok(app_id) = id_str.parse::<i64>() {
                                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                                        let logo = item.get("logo").and_then(|l| l.as_str()).unwrap_or("").to_string();
                                        app_info = Some((app_id, name, logo));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 若依然未找到匹配游戏，直接返回
    let (app_id, app_name, tiny_image) = match app_info {
        Some(info) => info,
        None => return (None, got_403),
    };

    entry.appid = Some(app_id);
    entry.name = Some(app_name);

    // 解析封面图 URL
    let mut best_cover_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg", app_id);
    if let Some(apps_idx) = tiny_image.find(&format!("/apps/{}/", app_id)) {
        let remainder = &tiny_image[apps_idx + format!("/apps/{}/", app_id).len()..];
        if let Some(slash_idx) = remainder.find('/') {
            let hash = &remainder[..slash_idx];
            let hash_library_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/{}/library_600x900_2x.jpg", app_id, hash);
            if let Ok(res) = client.head(&hash_library_url).send() {
                if res.status().is_success() {
                    best_cover_url = hash_library_url;
                } else if let Ok(res2) = client.head(&best_cover_url).send() {
                    if !res2.status().is_success() && !tiny_image.is_empty() {
                        best_cover_url = tiny_image.clone();
                    }
                }
            }
        }
    } else if let Ok(res2) = client.head(&best_cover_url).send() {
        if !res2.status().is_success() && !tiny_image.is_empty() {
            best_cover_url = tiny_image;
        }
    }
    entry.local_cover = Some(best_cover_url);

    // 获取游戏详情（发行日期、流派）
    let details_url = format!("https://store.steampowered.com/api/appdetails?appids={}&filters=basic,release_date,genres&l={}&cc=CN", app_id, lang);
    if let Ok(det_res) = client.get(&details_url).send() {
        if det_res.status() == reqwest::StatusCode::FORBIDDEN || det_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            got_403 = true;
        } else if det_res.status().is_success() {
            if let Ok(det_json) = det_res.json::<Value>() {
                let app_id_str = app_id.to_string();
                if let Some(data) = det_json.get(&app_id_str).and_then(|d| d.get("data")) {
                    if let Some(release_date) = data.get("release_date").and_then(|r| r.get("date")).and_then(|d| d.as_str()) {
                        entry.release_date = Some(release_date.to_string());
                    }
                    if let Some(genres) = data.get("genres").and_then(|g| g.as_array()) {
                        let genre_names: Vec<String> = genres.iter()
                            .filter_map(|g| g.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()))
                            .collect();
                        if !genre_names.is_empty() {
                            entry.genres = Some(genre_names.join(", "));
                        }
                    }
                }
            }
        }
    }

    // 获取评价（好评率与描述）- 全部评测
    let review_url = format!("https://store.steampowered.com/appreviews/{}?json=1&language=all&l={}&purchase_type=all", app_id, lang);
    if let Ok(rev_res) = client.get(&review_url).send() {
        if rev_res.status() == reqwest::StatusCode::FORBIDDEN || rev_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            got_403 = true;
        } else if rev_res.status().is_success() {
            if let Ok(rev_json) = rev_res.json::<Value>() {
                if let Some(query_summary) = rev_json.get("query_summary") {
                    let score = query_summary.get("review_score").and_then(|d| d.as_i64()).unwrap_or(0);
                    entry.review_score_desc = Some(score as i32);
                    let total = query_summary.get("total_reviews").and_then(|t| t.as_f64()).unwrap_or(0.0);
                    let pct = query_summary.get("total_positive").and_then(|t| t.as_f64()).unwrap_or(0.0);
                    
                    if total >= 10.0 && score > 0 {
                        entry.positive_percent = Some(((pct / total) * 100.0) as i64);
                        entry.total_reviews = Some(total as i64);
                    } else if total > 0.0 {
                        entry.total_reviews = Some(total as i64);
                        entry.positive_percent = None;
                    }
                }
            }
        }
    }

    // 获取最近评测（近30天）
    let recent_review_url = format!("https://store.steampowered.com/appreviews/{}?json=1&language=all&l={}&purchase_type=all&day_range=30", app_id, lang);
    if let Ok(rev_res) = client.get(&recent_review_url).send() {
        if rev_res.status() == reqwest::StatusCode::FORBIDDEN || rev_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            got_403 = true;
        } else if rev_res.status().is_success() {
            if let Ok(rev_json) = rev_res.json::<Value>() {
                if let Some(query_summary) = rev_json.get("query_summary") {
                    let score = query_summary.get("review_score").and_then(|d| d.as_i64()).unwrap_or(0);
                    let total = query_summary.get("total_reviews").and_then(|t| t.as_f64()).unwrap_or(0.0);
                    let pct = query_summary.get("total_positive").and_then(|t| t.as_f64()).unwrap_or(0.0);
                    if total >= 10.0 && score > 0 {
                        entry.recent_review_score_desc = Some(score as i32);
                        entry.recent_positive_percent = Some(((pct / total) * 100.0) as i64);
                    }
                }
            }
        }
    }

    (Some(entry), got_403)
}

/// 保持原签名的兼容入口
pub fn fetch_steam_game_info(client: &Client, base_name: &str, lang: &str) -> Option<SteamCacheEntry> {
    fetch_steam_game_info_ext(client, base_name, lang).0
}

/// 自适应频控控制器：当出现 403 时降为单线程保护期 1 分钟；保护期恢复时每 1 分钟 +1 并发，若 +1 后遇 403 则回滚 -1
pub struct ThrottleController {
    max_threads: usize,
    current_threads: usize,
    in_protective_ramp: bool,
    last_change: Instant,
    last_403_event: Instant,
}

impl ThrottleController {
    pub fn new(max_threads: usize) -> Self {
        Self {
            max_threads,
            current_threads: max_threads,
            in_protective_ramp: false,
            last_change: Instant::now(),
            last_403_event: Instant::now() - Duration::from_secs(100),
        }
    }

    /// 检查是否平稳运行满 1 分钟；若是且处于保护恢复期，则并发 +1，直到恢复至配置值
    pub fn check_ramp(&mut self) -> Option<(usize, String)> {
        if self.in_protective_ramp {
            if self.last_change.elapsed() >= Duration::from_secs(60) {
                self.last_change = Instant::now();
                self.current_threads += 1;
                if self.current_threads >= self.max_threads {
                    self.current_threads = self.max_threads;
                    self.in_protective_ramp = false;
                    Some((
                        self.current_threads,
                        format!(
                            "Steam 频控完全解除，并发已恢复至配置值 ({} 线程)",
                            self.max_threads
                        ),
                    ))
                } else {
                    Some((
                        self.current_threads,
                        format!(
                            "Steam 保护期平稳运行1分钟，并发提升至 {} / {} 线程，将以新并发继续观察1分钟",
                            self.current_threads, self.max_threads
                        ),
                    ))
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 遭遇 403 频控时的降级 / 回滚逻辑
    pub fn on_403(&mut self, base_name: &str) -> (usize, String) {
        let now = Instant::now();
        // 防短时间突发多次 403 导致并发连续骤降（3秒内的 403 视为同一次频控突发）
        let is_same_burst = self.last_403_event.elapsed() < Duration::from_secs(3);
        self.last_403_event = now;

        if !self.in_protective_ramp {
            // 首次遭遇 403：进入保护期，主动降为 1 线程并发
            self.in_protective_ramp = true;
            self.current_threads = 1;
            self.last_change = now;
            (
                1,
                format!(
                    "⚠️ 遭遇 Steam 频控(403)，已进入保护期降为单线程，将持续运行1分钟: {}",
                    base_name
                ),
            )
        } else {
            // 已在保护恢复期中，提升后再次遭遇 403：回滚到 -1 的状态
            let old = self.current_threads;
            if !is_same_burst {
                self.current_threads = self.current_threads.saturating_sub(1).max(1);
            }
            self.last_change = now; // 重置当前并发档位的1分钟观察计时

            if old > 1 {
                (
                    self.current_threads,
                    format!(
                        "⚠️ 并发提升至 {} 线程时再次遭遇 403，已回滚至 {} 线程并持续1分钟: {}",
                        old, self.current_threads, base_name
                    ),
                )
            } else {
                (
                    1,
                    format!(
                        "⚠️ 单线程保护期内仍有 403 频控，已重置1分钟冷却计时: {}",
                        base_name
                    ),
                )
            }
        }
    }

    pub fn get_current_threads(&self) -> usize {
        self.current_threads
    }

    pub fn is_ramping(&self) -> bool {
        self.in_protective_ramp
    }
}

