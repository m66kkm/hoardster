use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::collections::HashMap;
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::{Duration, Instant};
use regex::Regex;

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
    #[serde(default)]
    pub is_manual: Option<bool>,
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
            genres TEXT,
            is_manual INTEGER DEFAULT 0
        )",
        [],
    )?;
    // Migration: add recent review columns if they don't exist (for existing databases)
    conn.execute("ALTER TABLE steam_cache ADD COLUMN recent_review_score_desc INTEGER", []).ok();
    conn.execute("ALTER TABLE steam_cache ADD COLUMN recent_positive_percent INTEGER", []).ok();
    conn.execute("ALTER TABLE steam_cache ADD COLUMN is_manual INTEGER DEFAULT 0", []).ok();
    Ok(())
}

pub fn get_steam_cache(conn: &Connection) -> Result<HashMap<String, SteamCacheEntry>> {
    let mut stmt = conn.prepare("SELECT base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, recent_review_score_desc, recent_positive_percent, release_date, last_updated, genres, is_manual FROM steam_cache")?;
    let cache_iter = stmt.query_map([], |row| {
        let is_manual: Option<i32> = row.get(12).ok();
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
            is_manual: is_manual.map(|v| v != 0),
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
    let is_man = entry.is_manual.map(|v| if v { 1 } else { 0 });
    conn.execute(
        "INSERT INTO steam_cache (base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, recent_review_score_desc, recent_positive_percent, release_date, last_updated, genres, is_manual)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(base_name) DO UPDATE SET
            appid = CASE WHEN steam_cache.is_manual = 1 THEN steam_cache.appid ELSE excluded.appid END,
            name = CASE WHEN steam_cache.is_manual = 1 THEN steam_cache.name ELSE COALESCE(excluded.name, steam_cache.name) END,
            local_cover = CASE
                WHEN steam_cache.is_manual = 1 THEN steam_cache.local_cover
                ELSE COALESCE(excluded.local_cover, steam_cache.local_cover)
            END,
            review_score_desc = excluded.review_score_desc,
            positive_percent = excluded.positive_percent,
            total_reviews = excluded.total_reviews,
            recent_review_score_desc = excluded.recent_review_score_desc,
            recent_positive_percent = excluded.recent_positive_percent,
            release_date = COALESCE(excluded.release_date, steam_cache.release_date),
            last_updated = excluded.last_updated,
            genres = COALESCE(excluded.genres, steam_cache.genres),
            is_manual = COALESCE(steam_cache.is_manual, excluded.is_manual, 0)",
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
            entry.genres,
            is_man
        ],
    )?;
    Ok(())
}

lazy_static::lazy_static! {
    static ref SEQUEL_NUM_RE: Regex = Regex::new(r"\d+").unwrap();
    static ref ROMAN_NUM_RE: Regex = Regex::new(r"(?i)\b(ii|iii|iv|v|vi|vii|viii|ix|x)\b").unwrap();
    static ref STEAM_APPID_URL_RE: Regex = Regex::new(r"(?:/app/|appid=)(\d+)").unwrap();
}

/// 提取游戏名称中的续作编号（包括阿拉伯数字 2-20 与罗马数字 II-X）
pub fn extract_sequel_numbers(text: &str) -> Vec<u32> {
    let mut nums = Vec::new();

    // 1. 阿拉伯数字 (2..=20)，排除如 1998, 2077 等发行年份，并排除如 4K/8K 分辨率或 1080p
    for m in SEQUEL_NUM_RE.find_iter(text) {
        let end_idx = m.end();
        let remainder = &text[end_idx..];
        if remainder.starts_with('k') || remainder.starts_with('K') || remainder.starts_with('p') || remainder.starts_with('P') {
            continue;
        }

        if let Ok(val) = m.as_str().parse::<u32>() {
            if val >= 2 && val <= 20 {
                if !nums.contains(&val) {
                    nums.push(val);
                }
            }
        }
    }

    // 2. Unicode 罗马数字 (Ⅱ..Ⅹ / ⅱ..ⅹ)
    for ch in text.chars() {
        let val = match ch {
            'ⅱ' | 'Ⅱ' => 2,
            'ⅲ' | 'Ⅲ' => 3,
            'ⅳ' | 'Ⅳ' => 4,
            'ⅴ' | 'Ⅴ' => 5,
            'ⅵ' | 'Ⅵ' => 6,
            'ⅶ' | 'Ⅶ' => 7,
            'ⅷ' | 'Ⅷ' => 8,
            'ⅸ' | 'Ⅸ' => 9,
            'ⅹ' | 'Ⅹ' => 10,
            _ => 0,
        };
        if val > 0 && !nums.contains(&val) {
            nums.push(val);
        }
    }

    // 3. ASCII 罗马数字
    // 3a. 标准单词边界匹配 (如 Civilization VI, Street Fighter V)
    let lower = text.to_lowercase();
    for m in ROMAN_NUM_RE.find_iter(&lower) {
        let val = match m.as_str().trim() {
            "ii" => 2,
            "iii" => 3,
            "iv" => 4,
            "v" => 5,
            "vi" => 6,
            "vii" => 7,
            "viii" => 8,
            "ix" => 9,
            "x" => 10,
            _ => 0,
        };
        if val > 0 && !nums.contains(&val) {
            nums.push(val);
        }
    }

    // 3b. 针对紧贴中文字符的罗马数字（如 银河文明IV, 最终幻想VII）
    for roman_str in &["viii", "vii", "iii", "xii", "xiv", "vi", "iv", "ix", "ii", "x", "v"] {
        if let Some(pos) = lower.find(roman_str) {
            let before = &lower[..pos];
            let after = &lower[pos + roman_str.len()..];
            let before_ok = before.is_empty() || before.chars().last().map_or(false, |c| !c.is_ascii_alphabetic());
            let after_ok = after.is_empty() || after.chars().next().map_or(false, |c| !c.is_ascii_alphabetic());
            if before_ok && after_ok {
                let val = match *roman_str {
                    "ii" => 2,
                    "iii" => 3,
                    "iv" => 4,
                    "v" => 5,
                    "vi" => 6,
                    "vii" => 7,
                    "viii" => 8,
                    "ix" => 9,
                    "x" => 10,
                    _ => 0,
                };
                if val > 0 && !nums.contains(&val) {
                    nums.push(val);
                }
            }
        }
    }

    nums
}

/// 检查续作序号兼容性（Sequel Invariance 防漂移强规则）
pub fn is_sequel_compatible(query: &str, candidate: &str) -> bool {
    let q_seq = extract_sequel_numbers(query);
    let c_seq = extract_sequel_numbers(candidate);

    if q_seq.is_empty() {
        // Query 没有续作序号，若 candidate 明确带有续作序号 (2..=20)，则坚决互斥拒绝
        if !c_seq.is_empty() {
            // 特殊规则：Skyrim 即 The Elder Scrolls V: Skyrim，允许匹配
            let q_lower = query.to_lowercase();
            let c_lower = candidate.to_lowercase();
            if q_lower.contains("skyrim") && c_lower.contains("skyrim") && c_seq == vec![5] {
                return true;
            }
            return false;
        }
    } else {
        // Query 带有续作序号（例如 2），若 candidate 没有序号或序号不重合，拒绝
        if c_seq.is_empty() {
            return false;
        }
        if !q_seq.iter().any(|num| c_seq.contains(num)) {
            return false;
        }
    }
    true
}

/// 提取核心比较词元
fn get_core_tokens(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .filter(|s| {
            !matches!(
                *s,
                "the" | "a" | "an" | "and" | "of" | "edition" | "repack" | "game" | "version"
            )
        })
        .map(|s| s.to_string())
        .collect()
}

/// 计算候选游戏名称与检索词的综合置信度得分 (0.0 ~ 1.0)
pub fn calculate_match_score(query: &str, candidate_name: &str) -> f64 {
    // 1. 续作序号兼容性校验：不兼容直接判定 0 分拒绝
    if !is_sequel_compatible(query, candidate_name) {
        return 0.0;
    }

    fn norm(s: &str) -> String {
        s.to_lowercase()
            .replace(|c: char| matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '\"' | '\u{201C}' | '\u{201D}' | '®' | '™'), "")
    }

    let q_clean = norm(query);
    let c_clean = norm(candidate_name);

    // 辅助内容降权（Soundtrack, OST, DLC, Pack, Pass, Kit, Wallpapers, Demo, Playtest, Bundle 等）
    let aux_keywords = [
        "soundtrack", "ost", "dlc", "expansion pass", "season pass", "pass",
        "pack", "content", "creation kit", "kit", "wallpapers", "demo", "playtest",
        "script extender", "tool", "sdk", "mod", "bundle", "upgrade",
        "原声带", "文化包", "扩展包", "季票", "试玩", "礼包"
    ];
    let q_lower = query.to_lowercase();
    let c_lower = candidate_name.to_lowercase();
    let mut aux_penalty = 1.0;
    for aux in &aux_keywords {
        if c_lower.contains(aux) && !q_lower.contains(aux) {
            aux_penalty = 0.1;
            break;
        }
    }

    // 2. 完全相同直接满分
    if q_clean == c_clean {
        return (1.0f64 * aux_penalty).min(1.0f64);
    }

    let q_tokens = get_core_tokens(&q_clean);
    let c_tokens = get_core_tokens(&c_clean);

    if q_tokens.is_empty() || c_tokens.is_empty() {
        return 0.0;
    }

    let mut intersection_count = 0;
    for qt in &q_tokens {
        if c_tokens.contains(qt) {
            intersection_count += 1;
        }
    }

    if intersection_count == 0 {
        return 0.0;
    }

    let query_coverage = intersection_count as f64 / q_tokens.len() as f64;
    let candidate_coverage = intersection_count as f64 / c_tokens.len() as f64;
    let union_count = q_tokens.len() + c_tokens.len() - intersection_count;
    let jaccard = intersection_count as f64 / union_count as f64;

    let prefix_bonus = if c_clean.starts_with(&q_clean) || q_clean.starts_with(&c_clean) {
        0.25
    } else {
        0.0
    };

    let score = ((query_coverage * 0.5 + jaccard * 0.25 + prefix_bonus) * aux_penalty).min(1.0);

    if query_coverage < 0.35 && candidate_coverage < 0.35 {
        return 0.0;
    }

    score
}

/// 从 Steam appdetails 返回的 JSON 中提取游戏 data 对象
/// （兼容 Steam 后端内部重定向或根键与请求 ID 不一致的情况，如 289130 返回根键为 718570）
pub fn extract_steam_app_data(det_json: &Value, app_id: u32) -> Option<&Value> {
    let app_id_str = app_id.to_string();
    // 1. 优先尝试直接匹配请求的 app_id
    if let Some(item) = det_json.get(&app_id_str) {
        if item.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
            if let Some(data) = item.get("data") {
                return Some(data);
            }
        }
    }

    // 2. 若根键不匹配，寻找对象中首个 success: true 且包含有效 data 的对象
    if let Some(obj) = det_json.as_object() {
        for (_k, v) in obj {
            if v.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
                if let Some(data) = v.get("data") {
                    return Some(data);
                }
            }
        }
    }

    None
}

/// 独立的 Steam 游戏信息获取服务接口（返回结果与是否触发403/429频控标识）
pub fn fetch_steam_game_info_ext(client: &Client, base_name: &str, lang: &str) -> (Option<SteamCacheEntry>, bool) {
    let mut got_403 = false;

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
        is_manual: Some(false),
    };

    let mut app_info: Option<(i64, String, String)> = None; // (app_id, name, tiny_image)

    // 生成渐进式检索词列表（Query Relaxation 查询松弛）
    let mut search_terms = vec![base_name.trim().to_string()];

    // 若 base_name 经过深度清洗后产生更核心的词（例如剥离了版本修饰），生成核心检索词
    let cleaned = crate::scanner::base_game_name(base_name);
    if !cleaned.is_empty() && cleaned != base_name {
        if !search_terms.contains(&cleaned) {
            search_terms.push(cleaned);
        }
    }

    // 若词数较多（>= 3 个单词），尝试剥离最后一个修饰词
    let words: Vec<&str> = base_name.split_whitespace().collect();
    if words.len() >= 3 {
        let relaxed_tail = words[..words.len() - 1].join(" ");
        if !search_terms.contains(&relaxed_tail) {
            search_terms.push(relaxed_tail);
        }
    }

    // 1. 尝试 Steam 商店搜索接口（结合语言回退、查询松弛与置信度打分）
    let has_cjk = base_name.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    let mut search_languages = Vec::new();
    if has_cjk {
        search_languages.push(lang.to_string());
        if lang != "english" {
            search_languages.push("english".to_string());
        }
    } else {
        search_languages.push("english".to_string());
        if lang != "english" {
            search_languages.push(lang.to_string());
        }
    }

    for current_lang in &search_languages {
        if app_info.is_some() || got_403 {
            break;
        }

        for term in &search_terms {
            if app_info.is_some() || got_403 {
                break;
            }

            let encoded = urlencoding::encode(term);
            let search_url = format!("https://store.steampowered.com/api/storesearch/?term={}&l={}&cc=US", encoded, current_lang);

            if let Ok(res) = client.get(&search_url).send() {
                if res.status() == reqwest::StatusCode::FORBIDDEN || res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    got_403 = true;
                    break;
                } else if res.status().is_success() {
                    if let Ok(res_str) = res.text() {
                        if let Ok(res_json) = serde_json::from_str::<Value>(&res_str) {
                            if let Some(items) = res_json.get("items").and_then(|i| i.as_array()) {
                                let mut best_match = None;
                                let mut best_score = 0.0;

                                for item in items {
                                    let name_str = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    let is_app = item.get("type").and_then(|t| t.as_str()) == Some("app");
                                    let mut score = calculate_match_score(base_name, name_str);

                                    // 非独立游戏（如DLC/音乐包/捆绑包）给予降权
                                    if !is_app {
                                        score *= 0.5;
                                    }

                                    if score > best_score && score >= 0.45 {
                                        best_score = score;
                                        best_match = Some(item);
                                    }
                                }

                                if let Some(best) = best_match {
                                    if let Some(app_id) = best.get("id").and_then(|id| id.as_i64()) {
                                        let name = best.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                                        let tiny = best.get("tiny_image").and_then(|t| t.as_str()).unwrap_or("").to_string();
                                        app_info = Some((app_id, name, tiny));
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

    // 2. 如果商店搜索受限或未命中，回退到 Steam Community 搜索接口（带置信度与续作防护）
    if app_info.is_none() && !got_403 {
        for term in &search_terms {
            if app_info.is_some() || got_403 {
                break;
            }

            let encoded = urlencoding::encode(term);
            let comm_url = format!("https://steamcommunity.com/actions/SearchApps/{}", encoded);
            if let Ok(comm_res) = client.get(&comm_url).send() {
                if comm_res.status() == reqwest::StatusCode::FORBIDDEN || comm_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    got_403 = true;
                    break;
                } else if comm_res.status().is_success() {
                    if let Ok(comm_json) = comm_res.json::<Value>() {
                        if let Some(arr) = comm_json.as_array() {
                            let mut best = None;
                            let mut best_score = 0.0;

                            for item in arr {
                                let name_str = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                let score = calculate_match_score(base_name, name_str);
                                if score > best_score && score >= 0.5 {
                                    best_score = score;
                                    best = Some(item);
                                }
                            }

                            if let Some(item) = best {
                                if let Some(id_str) = item.get("appid").and_then(|i| i.as_str()) {
                                    if let Ok(app_id) = id_str.parse::<i64>() {
                                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                                        let logo = item.get("logo").and_then(|l| l.as_str()).unwrap_or("").to_string();
                                        app_info = Some((app_id, name, logo));
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
    for cc_param in &["&cc=US", ""] {
        let details_url = format!("https://store.steampowered.com/api/appdetails?appids={}&filters=basic,release_date,genres&l={}{}", app_id, lang, cc_param);
        if let Ok(det_res) = client.get(&details_url).send() {
            if det_res.status() == reqwest::StatusCode::FORBIDDEN || det_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                got_403 = true;
                break;
            } else if det_res.status().is_success() {
                if let Ok(det_json) = det_res.json::<Value>() {
                    if let Some(data) = extract_steam_app_data(&det_json, app_id as u32) {
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
                        if let Some(header_img) = data.get("header_image").and_then(|h| h.as_str()) {
                            let mut resolved_url = header_img.to_string();
                            if let Some(apps_idx) = header_img.find(&format!("/apps/{}/", app_id)) {
                                let remainder = &header_img[apps_idx + format!("/apps/{}/", app_id).len()..];
                                if let Some(slash_idx) = remainder.find('/') {
                                    let hash = &remainder[..slash_idx];
                                    let hash_library_url = format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/{}/library_600x900_2x.jpg", app_id, hash);
                                    if let Ok(res) = client.head(&hash_library_url).send() {
                                        if res.status().is_success() {
                                            resolved_url = hash_library_url;
                                        }
                                    }
                                }
                            }
                            entry.local_cover = Some(resolved_url);
                        }
                        break;
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

#[derive(Clone, Debug)]
pub struct SteamSyncTarget {
    pub base_name: String,
    pub appid: Option<u32>,
}

impl SteamSyncTarget {
    pub fn from_name(name: impl Into<String>) -> Self {
        Self {
            base_name: name.into(),
            appid: None,
        }
    }

    pub fn with_appid(name: impl Into<String>, appid: Option<u32>) -> Self {
        Self {
            base_name: name.into(),
            appid,
        }
    }
}

pub enum ProgressReporter {
    ScanProgress {
        step: String,
    },
    ScrapeProgress {
        event_name: String,
    },
}

impl ProgressReporter {
    pub fn emit_progress(&self, app_handle: &tauri::AppHandle, current: usize, total: usize, message: &str, status: &str) {
        use tauri::Emitter;
        match self {
            ProgressReporter::ScanProgress { step } => {
                let _ = app_handle.emit(
                    "scan-progress",
                    serde_json::json!({
                        "step": step,
                        "message": message,
                        "current": current,
                        "total": total,
                    }),
                );
            }
            ProgressReporter::ScrapeProgress { event_name } => {
                let _ = app_handle.emit(
                    event_name,
                    serde_json::json!({
                        "current_page": current as u32,
                        "total_pages": total as u32,
                        "message": message,
                        "status": status,
                    }),
                );
            }
        }
    }
}

/// 通用的多线程 Steam 元数据同步引擎（供 Skidrow/Reloaded、1337x 抓取与本地盘库扫描全场景复用）
pub fn sync_steam_metadata_blocking(
    app_handle: tauri::AppHandle,
    targets: Vec<SteamSyncTarget>,
    reporter: ProgressReporter,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> std::result::Result<usize, String> {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    let total_targets = targets.len();
    if total_targets == 0 {
        return Ok(0);
    }

    // 1. 从数据库读取线程数、安全延迟与语言设置
    let (threads, delay_ms, language) = {
        if let Ok(conn) = crate::db::get_connection() {
            let t_str: String = conn.query_row("SELECT value FROM config WHERE key = 'steam_api_threads'", [], |r| r.get(0)).unwrap_or_else(|_| "10".to_string());
            let d_str: String = conn.query_row("SELECT value FROM config WHERE key = 'steam_api_delay_ms'", [], |r| r.get(0)).unwrap_or_else(|_| "300".to_string());
            let l_str: String = conn.query_row("SELECT value FROM config WHERE key = 'language'", [], |r| r.get(0)).unwrap_or_else(|_| "schinese".to_string());
            (
                t_str.parse::<usize>().unwrap_or(10).max(1).min(50),
                d_str.parse::<u64>().unwrap_or(300),
                l_str,
            )
        } else {
            (10, 300, "schinese".to_string())
        }
    };

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    // 任务队列：(SteamSyncTarget, retry_count)
    let queue: Arc<Mutex<VecDeque<(SteamSyncTarget, usize)>>> = Arc::new(Mutex::new(
        targets.into_iter().map(|t| (t, 0)).collect()
    ));

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
            thread_idx: usize,
            base_name: String,
            entry: Option<SteamCacheEntry>,
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

                // 提取下一个待抓取目标
                let task = {
                    let mut q = queue_clone.lock().unwrap();
                    q.pop_front()
                };

                let (target, retry_count) = match task {
                    Some(item) => {
                        active_tasks_clone.fetch_add(1, Ordering::SeqCst);
                        item
                    }
                    None => {
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

                let (maybe_entry, got_403) = if let Some(app_id) = target.appid {
                    let (res, is_403) = fetch_steam_game_by_appid_ext(&client_clone, app_id, &target.base_name, &lang);
                    if is_403 {
                        (None, true)
                    } else if res.is_some() {
                        (res, false)
                    } else {
                        fetch_steam_game_info_ext(&client_clone, &target.base_name, &lang)
                    }
                } else {
                    fetch_steam_game_info_ext(&client_clone, &target.base_name, &lang)
                };

                if got_403 {
                    let notify_msg = {
                        let mut ctrl = throttle_ctrl_clone.lock().unwrap();
                        let (_c, msg) = ctrl.on_403(&target.base_name);
                        msg
                    };
                    let _ = tx_clone.send(WorkerMessage::StatusNotification { message: notify_msg });

                    if retry_count < 2 {
                        {
                            let mut q = queue_clone.lock().unwrap();
                            q.push_back((target, retry_count + 1));
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
                        let target_paths = [
                            covers_dir_clone.join(&cover_filename),
                            std::path::PathBuf::from("covers").join(&cover_filename),
                            std::path::PathBuf::from("src-tauri/covers").join(&cover_filename),
                            std::path::PathBuf::from("../covers").join(&cover_filename),
                        ];

                        if !target_paths.iter().any(|p| p.exists()) {
                            let mut urls_to_try = Vec::new();
                            if let Some(ref u) = entry.local_cover {
                                if u.starts_with("http") && !urls_to_try.contains(u) {
                                    urls_to_try.push(u.clone());
                                }
                            }
                            urls_to_try.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg", app_id));
                            urls_to_try.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg", app_id));
                            urls_to_try.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg", app_id));
                            urls_to_try.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/capsule_616x353.jpg", app_id));

                            for u in urls_to_try {
                                if let Ok(img_res) = client_clone.get(&u).send() {
                                    if img_res.status().is_success() {
                                        if let Ok(img_bytes) = img_res.bytes() {
                                            if img_bytes.len() > 1024 {
                                                for p in &target_paths {
                                                    if let Some(parent) = p.parent() {
                                                        let _ = std::fs::create_dir_all(parent);
                                                    }
                                                    let _ = std::fs::write(p, &img_bytes);
                                                }
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        entry.local_cover = Some(format!("covers/{}", cover_filename));
                    }
                    Some(entry)
                } else {
                    None
                };

                let _ = tx_clone.send(WorkerMessage::ItemProcessed {
                    thread_idx: thread_idx + 1,
                    base_name: target.base_name,
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
    drop(client);

    let mut processed = 0;
    let mut new_steam_entries = 0;
    let conn = crate::db::get_connection().map_err(|e| e.to_string())?;

    for msg in rx {
        if cancel_flag.load(Ordering::Relaxed) {
            reporter.emit_progress(&app_handle, processed, total_targets, "Steam 信息获取已被用户取消", "error");
            for handle in thread_handles {
                let _ = handle.join();
            }
            return Err("扫描已被用户取消".to_string());
        }

        match msg {
            WorkerMessage::StatusNotification { message } => {
                reporter.emit_progress(&app_handle, processed, total_targets, &message, "fetching");
            }
            WorkerMessage::ItemProcessed { thread_idx, base_name, entry, current_concurrency, is_ramping } => {
                processed += 1;
                let thread_tag = format!("[线程#{}/{}]", thread_idx, threads);
                let ramp_tag = if is_ramping {
                    format!(" [保护恢复中: {}/{}并发]", current_concurrency, threads)
                } else {
                    format!(" [{}并发]", current_concurrency)
                };
                let display_msg = format!("正在获取 Steam 游戏评价 ({} / {}) {}{}: {}", processed, total_targets, thread_tag, ramp_tag, base_name);
                reporter.emit_progress(&app_handle, processed, total_targets, &display_msg, "fetching");

                if let Some(entry) = entry {
                    if entry.appid.is_some() {
                        for attempt in 0..3 {
                            match crate::db::insert_steam_cache_entry(&conn, &entry) {
                                Ok(_) => {
                                    new_steam_entries += 1;
                                    break;
                                }
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

    // 自动同步游戏元数据与封面关联
    let _ = crate::db::sync_game_metadata_covers(&conn);

    Ok(new_steam_entries)
}


/// 解析用户输入的 Steam AppID（支持纯数字、完整商店 URL、社区 URL）
pub fn parse_steam_appid(input: &str) -> Option<u32> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. 尝试从 URL 或路径结构中正则提取 (如 /app/239140 或 appid=239140)
    if let Some(caps) = STEAM_APPID_URL_RE.captures(trimmed) {
        if let Some(m) = caps.get(1) {
            if let Ok(id) = m.as_str().parse::<u32>() {
                if id > 0 {
                    return Some(id);
                }
            }
        }
    }

    // 2. 尝试纯数字解析
    if let Ok(id) = trimmed.parse::<u32>() {
        if id > 0 {
            return Some(id);
        }
    }

    None
}

/// 通过确定的 Steam AppID 精确抓取游戏详情、评价与高清竖版封面（包含是否触发403/429标识）
pub fn fetch_steam_game_by_appid_ext(
    client: &Client,
    app_id: u32,
    base_name: &str,
    lang: &str,
) -> (Option<SteamCacheEntry>, bool) {
    let mut got_403 = false;
    let mut entry = SteamCacheEntry {
        base_name: base_name.to_string(),
        appid: Some(app_id as i64),
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
        is_manual: Some(false),
    };

    // 1. 获取基本详情 (name, release_date, genres)
    // 优先使用 cc=US 避免国区限制屏蔽成人向或锁区游戏（如《洛夫克拉夫特行动：堕落玩偶》）；若获取失败再尝试无 cc 参数
    let mut app_data_opt = None;

    for cc_param in &["&cc=US", ""] {
        let details_url = format!(
            "https://store.steampowered.com/api/appdetails?appids={}&filters=basic,release_date,genres&l={}{}",
            app_id, lang, cc_param
        );
        match client.get(&details_url).send() {
            Ok(det_res) => {
                if det_res.status() == reqwest::StatusCode::FORBIDDEN || det_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    got_403 = true;
                    break;
                }
                if det_res.status().is_success() {
                    if let Ok(det_json) = det_res.json::<Value>() {
                        if let Some(d) = extract_steam_app_data(&det_json, app_id) {
                            app_data_opt = Some(d.clone());
                            break;
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }

    let app_data = match app_data_opt {
        Some(d) => d,
        None => return (None, got_403),
    };

    if let Some(name) = app_data.get("name").and_then(|n| n.as_str()) {
        entry.name = Some(name.to_string());
    }

    let canonical_id = app_data.get("steam_appid").and_then(|id| id.as_i64()).unwrap_or(app_id as i64);
    entry.appid = Some(canonical_id);

    if let Some(release_date) = app_data
        .get("release_date")
        .and_then(|r| r.get("date"))
        .and_then(|d| d.as_str())
    {
        entry.release_date = Some(release_date.to_string());
    }

    if let Some(genres) = app_data.get("genres").and_then(|g| g.as_array()) {
        let genre_names: Vec<String> = genres
            .iter()
            .filter_map(|g| g.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()))
            .collect();
        if !genre_names.is_empty() {
            entry.genres = Some(genre_names.join(", "));
        }
    }

    // 2. 获取全部评测数据
    let review_url = format!(
        "https://store.steampowered.com/appreviews/{}?json=1&language=all&l={}&purchase_type=all",
        app_id, lang
    );
    if let Ok(rev_res) = client.get(&review_url).send() {
        if rev_res.status() == reqwest::StatusCode::FORBIDDEN || rev_res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            got_403 = true;
        } else if rev_res.status().is_success() {
            if let Ok(rev_json) = rev_res.json::<Value>() {
                if let Some(query_summary) = rev_json.get("query_summary") {
                    let score = query_summary.get("review_score").and_then(|d| d.as_i64()).unwrap_or(0);
                    let total = query_summary.get("total_reviews").and_then(|t| t.as_f64()).unwrap_or(0.0);
                    let pct = query_summary.get("total_positive").and_then(|t| t.as_f64()).unwrap_or(0.0);
                    entry.review_score_desc = Some(score as i32);
                    entry.total_reviews = Some(total as i64);
                    if total >= 10.0 && score > 0 {
                        entry.positive_percent = Some(((pct / total) * 100.0) as i64);
                    }
                }
            }
        }
    }

    // 3. 获取近 30 天评测数据
    let recent_review_url = format!(
        "https://store.steampowered.com/appreviews/{}?json=1&language=all&l={}&purchase_type=all&day_range=30",
        app_id, lang
    );
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

    // 4. 下载并存储封面文件（优先竖版 600x900）
    let mut covers_dir = std::env::current_exe().unwrap_or_default();
    covers_dir.pop();
    covers_dir.push("covers");
    let _ = std::fs::create_dir_all(&covers_dir);

    let cover_filename = format!("{}.jpg", canonical_id);
    let target_paths = [
        covers_dir.join(&cover_filename),
        std::path::PathBuf::from("covers").join(&cover_filename),
        std::path::PathBuf::from("src-tauri/covers").join(&cover_filename),
        std::path::PathBuf::from("../covers").join(&cover_filename),
    ];

    let mut candidate_urls = vec![
        format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg", app_id),
        format!("https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900_2x.jpg", app_id),
        format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg", app_id),
        format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg", app_id),
    ];
    if canonical_id != app_id as i64 {
        candidate_urls.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg", canonical_id));
        candidate_urls.push(format!("https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900.jpg", canonical_id));
        candidate_urls.push(format!("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg", canonical_id));
    }

    let save_cover_bytes = |bytes: &[u8]| {
        for path in &target_paths {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, bytes);
        }
        if canonical_id != app_id as i64 {
            let alt_filename = format!("{}.jpg", app_id);
            for p in &[
                covers_dir.join(&alt_filename),
                std::path::PathBuf::from("covers").join(&alt_filename),
                std::path::PathBuf::from("src-tauri/covers").join(&alt_filename),
                std::path::PathBuf::from("../covers").join(&alt_filename),
            ] {
                let _ = std::fs::write(p, bytes);
            }
        }
    };

    let mut downloaded = false;
    for url in &candidate_urls {
        if let Ok(res) = client.get(url).send() {
            if res.status().is_success() {
                if let Ok(bytes) = res.bytes() {
                    if bytes.len() > 1024 {
                        save_cover_bytes(&bytes);
                        downloaded = true;
                        break;
                    }
                }
            }
        }
    }

    if downloaded {
        entry.local_cover = Some(format!("covers/{}.jpg", canonical_id));
    } else if let Some(header_img) = app_data.get("header_image").and_then(|h| h.as_str()) {
        if let Ok(res) = client.get(header_img).send() {
            if res.status().is_success() {
                if let Ok(bytes) = res.bytes() {
                    if bytes.len() > 1024 {
                        save_cover_bytes(&bytes);
                        entry.local_cover = Some(format!("covers/{}.jpg", canonical_id));
                    }
                }
            }
        }
    }

    if entry.local_cover.is_none() {
        entry.local_cover = Some(format!("covers/{}.jpg", canonical_id));
    }

    (Some(entry), got_403)
}

/// 通过确定的 Steam AppID 精确抓取游戏详情、评价与高清竖版封面
pub fn fetch_steam_game_by_appid(
    client: &Client,
    app_id: u32,
    base_name: &str,
    lang: &str,
) -> Result<SteamCacheEntry, String> {
    let (res, got_403) = fetch_steam_game_by_appid_ext(client, app_id, base_name, lang);
    if got_403 {
        return Err("遭遇 Steam 频控限制 (403/429)，请稍后再试".to_string());
    }
    res.ok_or_else(|| format!("未找到 AppID 为 {} 的 Steam 游戏详情，请确认该 ID 是否有效", app_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_steam_appid() {
        assert_eq!(parse_steam_appid("239140"), Some(239140));
        assert_eq!(parse_steam_appid(" 239140 "), Some(239140));
        assert_eq!(
            parse_steam_appid("https://store.steampowered.com/app/239140/Dying_Light_Definitive_Edition/"),
            Some(239140)
        );
        assert_eq!(
            parse_steam_appid("https://store.steampowered.com/app/239140"),
            Some(239140)
        );
        assert_eq!(
            parse_steam_appid("store.steampowered.com/app/239140/"),
            Some(239140)
        );
        assert_eq!(
            parse_steam_appid("https://steamcommunity.com/app/239140"),
            Some(239140)
        );
        assert_eq!(
            parse_steam_appid("https://store.steampowered.com/api/appdetails?appid=239140"),
            Some(239140)
        );
        assert_eq!(
            parse_steam_appid("https://store.steampowered.com/app/1685960/_/"),
            Some(1685960)
        );
        assert_eq!(parse_steam_appid("0"), None);
        assert_eq!(parse_steam_appid("abc"), None);
        assert_eq!(parse_steam_appid(""), None);
    }

    #[test]
    #[ignore]
    fn test_fetch_steam_game_by_appid_fallen_doll() {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap();
        let res = fetch_steam_game_by_appid(&client, 1685960, "Fallen Doll", "schinese");
        assert!(res.is_ok(), "Failed: {:?}", res.err());
        let entry = res.unwrap();
        assert_eq!(entry.appid, Some(1685960));
        assert_eq!(entry.name.as_deref(), Some("洛夫克拉夫特行动：堕落玩偶"));
        assert!(entry.local_cover.is_some());
    }

    #[test]
    #[ignore]
    fn test_fetch_steam_game_by_appid_endless_legend() {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap();
        let res = fetch_steam_game_by_appid(&client, 289130, "endless legend", "schinese");
        println!("Result for 289130: {:?}", res);
        assert!(res.is_ok(), "Failed: {:?}", res.err());
        let entry = res.unwrap();
        assert_eq!(entry.appid, Some(289130));
        println!("Entry name: {:?}", entry.name);
        println!("Entry local_cover: {:?}", entry.local_cover);
    }
}

