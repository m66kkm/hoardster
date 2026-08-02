use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::collections::HashMap;
use reqwest::blocking::Client;
use serde_json::Value;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SteamCacheEntry {
    pub base_name: String,
    pub appid: Option<i64>,
    pub name: Option<String>,
    pub local_cover: Option<String>,
    pub review_score_desc: Option<i32>,
    pub positive_percent: Option<i64>,
    pub total_reviews: Option<i64>,
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
            release_date TEXT,
            last_updated TEXT,
            genres TEXT
        )",
        [],
    )?;
    Ok(())
}

pub fn get_steam_cache(conn: &Connection) -> Result<HashMap<String, SteamCacheEntry>> {
    let mut stmt = conn.prepare("SELECT base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, release_date, last_updated, genres FROM steam_cache")?;
    let cache_iter = stmt.query_map([], |row| {
        Ok(SteamCacheEntry {
            base_name: row.get(0)?,
            appid: row.get(1)?,
            name: row.get(2)?,
            local_cover: row.get(3)?,
            review_score_desc: row.get(4)?,
            positive_percent: row.get(5)?,
            total_reviews: row.get(6)?,
            release_date: row.get(7)?,
            last_updated: row.get(8)?,
            genres: row.get(9)?,
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
        "INSERT OR REPLACE INTO steam_cache (base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, release_date, last_updated, genres)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            entry.base_name,
            entry.appid,
            entry.name,
            entry.local_cover,
            entry.review_score_desc,
            entry.positive_percent,
            entry.total_reviews,
            entry.release_date,
            entry.last_updated,
            entry.genres
        ],
    )?;
    Ok(())
}

/// 独立的 Steam 游戏信息获取服务接口
pub fn fetch_steam_game_info(client: &Client, base_name: &str, lang: &str) -> Option<SteamCacheEntry> {
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
        release_date: None,
        last_updated: Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()),
        genres: None,
    };

    if let Ok(res) = client.get(&search_url).send() {
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
                                entry.name = best.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                                
                                let tiny_image = best.get("tiny_image").and_then(|t| t.as_str()).unwrap_or("");
                                let mut best_cover_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg", app_id);
                                
                                // Parse hash from tiny_image (e.g. .../apps/123/HASH/capsule...)
                                if let Some(apps_idx) = tiny_image.find(&format!("/apps/{}/", app_id)) {
                                    let remainder = &tiny_image[apps_idx + format!("/apps/{}/", app_id).len()..];
                                    if let Some(slash_idx) = remainder.find('/') {
                                        let hash = &remainder[..slash_idx];
                                        let hash_library_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/{}/library_600x900_2x.jpg", app_id, hash);
                                        
                                        // Try HEAD request for hash_library_url
                                        if let Ok(res) = client.head(&hash_library_url).send() {
                                            if res.status().is_success() {
                                                best_cover_url = hash_library_url;
                                            } else {
                                                // Try without hash (legacy) or fallback to tiny_image
                                                if let Ok(res2) = client.head(&best_cover_url).send() {
                                                    if !res2.status().is_success() {
                                                        best_cover_url = tiny_image.to_string();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // No hash found, try legacy library URL, fallback to tiny_image
                                    if let Ok(res2) = client.head(&best_cover_url).send() {
                                        if !res2.status().is_success() && !tiny_image.is_empty() {
                                            best_cover_url = tiny_image.to_string();
                                        }
                                    }
                                }
                                
                                let cover_url = best_cover_url;
                                // 获取详情
                                let details_url = format!("https://store.steampowered.com/api/appdetails?appids={}&filters=basic,release_date,genres&l={}&cc=CN", app_id, lang);
                                if let Ok(det_res) = client.get(&details_url).send() {
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

                                // 评价
                                let review_url = format!("https://store.steampowered.com/appreviews/{}?json=1&l={}", app_id, lang);
                                if let Ok(rev_res) = client.get(&review_url).send() {
                                    if let Ok(rev_json) = rev_res.json::<Value>() {
                                        if let Some(query_summary) = rev_json.get("query_summary") {
                                            if let Some(score) = query_summary.get("review_score").and_then(|d| d.as_i64()) {
                                                entry.review_score_desc = Some(score as i32);
                                            }
                                            if let Some(pct) = query_summary.get("total_positive").and_then(|t| t.as_f64()) {
                                                if let Some(total) = query_summary.get("total_reviews").and_then(|t| t.as_f64()) {
                                                    if total > 0.0 {
                                                        entry.positive_percent = Some(((pct / total) * 100.0) as i64);
                                                        entry.total_reviews = Some(total as i64);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // 设置封面URL（调用者可据此决定是否下载）
                                entry.local_cover = Some(cover_url);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Some(entry)
}
