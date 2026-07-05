use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::sync::Mutex;

/// 数据库连接状态，通过 Mutex 保护 Connection 实现线程安全共享
pub struct DbState(pub Mutex<Connection>);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Game {
    pub id: Option<i64>,
    pub original_name: String,
    pub clean_name: String,
    pub base_name: String,
    pub r#type: String,
    pub source_path: String,
    pub full_path: String,
    pub size: String,
    pub size_bytes: i64,
    pub created: String,
    pub is_exact_dup: bool,
    pub is_version_dup: bool,
    pub is_representative: bool,
    
    // Steam Cache Fields
    pub appid: Option<i64>,
    pub name: Option<String>,
    pub local_cover: Option<String>,
    pub review_score_desc: Option<String>,
    pub positive_percent: Option<i64>,
    pub total_reviews: Option<i64>,
    pub release_date: Option<String>,
    pub genres: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SteamCacheEntry {
    pub base_name: String,
    pub appid: Option<i64>,
    pub name: Option<String>,
    pub local_cover: Option<String>,
    pub review_score_desc: Option<String>,
    pub positive_percent: Option<i64>,
    pub total_reviews: Option<i64>,
    pub release_date: Option<String>,
    pub last_updated: Option<String>,
    pub genres: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EpicFreeGame {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub image_url: String,
    pub game_url: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SteamFreeGame {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub r#type: String,
    pub end_date: String,
    pub image_url: String,
    pub giveaway_url: String,
}

#[derive(serde::Serialize, Debug)]
pub struct DuplicateGroup {
    pub name: String,
    pub games: Vec<Game>,
}

#[derive(serde::Serialize, Debug)]
pub struct FranchiseGroup {
    pub prefix: String,
    pub games: Vec<Game>,
}

#[derive(serde::Serialize, Debug)]
pub struct StatsSummary {
    pub total_scan: i64,
    pub unique_games: i64,
    pub franchise_count: i64,
    pub exact_dups: i64,
    pub version_dups: i64,
}

/// 扫描历史记录
#[derive(serde::Serialize, Debug)]
pub struct ScanHistoryRecord {
    pub id: i64,
    pub started_at: String,
    pub completed_at: String,
    pub total_scanned: i64,
    pub new_games: i64,
    pub new_steam_entries: i64,
    pub status: String,
}

pub fn init_db(conn: &Connection) -> Result<()> {
    // 1. 扫描路径表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scan_paths (
            path TEXT PRIMARY KEY
        )",
        [],
    )?;

    // 首次初始化时填充默认扫描路径
    let count: i64 = conn.query_row("SELECT count(*) FROM scan_paths", [], |r| r.get(0))?;
    if count == 0 {
        let defaults = vec!["E:\\Games", "D:\\Games", "I:\\", "K:\\"];
        for path in defaults {
            let _ = conn.execute("INSERT OR IGNORE INTO scan_paths (path) VALUES (?)", [path]);
        }
    }

    // 2. 游戏表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            original_name TEXT,
            clean_name TEXT,
            base_name TEXT,
            type TEXT,
            source_path TEXT,
            full_path TEXT UNIQUE,
            size TEXT,
            size_bytes INTEGER,
            created TEXT,
            is_exact_dup INTEGER DEFAULT 0,
            is_version_dup INTEGER DEFAULT 0,
            is_representative INTEGER DEFAULT 0
        )",
        [],
    )?;

    // 3. Steam 缓存表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_cache (
            base_name TEXT PRIMARY KEY,
            appid INTEGER,
            name TEXT,
            local_cover TEXT,
            review_score_desc TEXT,
            positive_percent INTEGER,
            total_reviews INTEGER,
            release_date TEXT,
            last_updated TEXT
        )",
        [],
    )?;

    // Add genres column if not exists (for existing databases)
    conn.execute("ALTER TABLE steam_cache ADD COLUMN genres TEXT", []).ok();

    // Migrate existing invalid review descriptions to "评价不可用"
    let _ = conn.execute(
        "UPDATE steam_cache 
         SET review_score_desc = '评价不可用' 
         WHERE review_score_desc IS NULL 
            OR review_score_desc = '' 
            OR review_score_desc LIKE '%篇用户评测%' 
            OR review_score_desc LIKE '%user reviews%' 
            OR review_score_desc LIKE '%Need more user reviews%' 
            OR review_score_desc LIKE '%不需要测评%' 
            OR review_score_desc = '无用户评测'",
        [],
    );

    // 4. 配置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    )?;

    // 5. 扫描历史表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scan_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at TEXT,
            completed_at TEXT,
            total_scanned INTEGER,
            new_games INTEGER,
            new_steam_entries INTEGER,
            status TEXT
        )",
        [],
    )?;

    // 6. 1337x 种子数据表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS torrents_1337x (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            torrent_id TEXT UNIQUE,
            name TEXT,
            url TEXT,
            seeds INTEGER,
            leeches INTEGER,
            date TEXT,
            size TEXT,
            uploader TEXT,
            uploader_url TEXT,
            fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            published_ts INTEGER DEFAULT 0
        )",
        [],
    )?;

    // Add published_ts column if not exists (for existing databases)
    conn.execute("ALTER TABLE torrents_1337x ADD COLUMN published_ts INTEGER DEFAULT 0", []).ok();

    // 7. Epic 免费游戏表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS epic_free_games (
            id TEXT PRIMARY KEY,
            title TEXT,
            description TEXT,
            status TEXT,
            start_date TEXT,
            end_date TEXT,
            image_url TEXT,
            game_url TEXT
        )",
        [],
    )?;

    // 8. Steam 免费获取表 (GamerPower)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_free_games (
            id INTEGER PRIMARY KEY,
            title TEXT,
            description TEXT,
            type TEXT,
            end_date TEXT,
            image_url TEXT,
            giveaway_url TEXT
        )",
        [],
    )?;

    // 初始化默认配置项（仅在对应 key 不存在时插入）
    let default_configs = vec![
        ("theme", "dark"),
        ("page_size", "50"),
        ("installed_drives", ""),
        ("exclude_folders", "System Volume Information;$Recycle.Bin;Recovery;Config.Msi;Documents and Settings;Program Files;Program Files (x86);Windows"),
        ("steam_api_delay_ms", "300"),
        ("steam_api_threads", "10"),
        ("last_scan_time", ""),
        ("language", "schinese"),
    ];
    for (key, value) in default_configs {
        conn.execute(
            "INSERT OR IGNORE INTO config (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
    }

    // 从 exe 所在目录查找 steam_cache.json 进行导入（仅在 steam_cache 表为空时）
    let cache_count: i64 = conn.query_row("SELECT count(*) FROM steam_cache", [], |r| r.get(0))?;
    if cache_count == 0 {
        let mut cache_path = std::env::current_exe().unwrap_or_default();
        cache_path.pop(); // 移除 exe 文件名，保留目录
        cache_path.push("steam_cache.json");

        if let Ok(json_str) = std::fs::read_to_string(&cache_path) {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(obj) = json_val.as_object() {
                    for (base_name, val) in obj {
                        let appid = val.get("AppId").and_then(|v| v.as_i64());
                        let name = val.get("Name").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let local_cover = val.get("LocalCover").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let review_score_desc = val.get("ReviewScoreDesc").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let positive_percent = val.get("PositivePercent").and_then(|v| v.as_i64()).or_else(|| val.get("PositivePercent").and_then(|v| v.as_f64()).map(|f| f as i64));
                        let total_reviews = val.get("TotalReviews").and_then(|v| v.as_i64());
                        let release_date = val.get("ReleaseDate").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let genres = val.get("Genres").and_then(|v| v.as_str()).map(|s| s.to_string());
                        
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO steam_cache (base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, release_date, last_updated, genres)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                            params![
                                base_name,
                                appid,
                                name,
                                local_cover,
                                review_score_desc,
                                positive_percent,
                                total_reviews,
                                release_date,
                                chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                genres
                            ],
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn get_scan_paths(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM scan_paths")?;
    let paths = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>>>()?;
    Ok(paths)
}

pub fn add_scan_path(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("INSERT OR IGNORE INTO scan_paths (path) VALUES (?)", [path])?;
    Ok(())
}

pub fn remove_scan_path(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("DELETE FROM scan_paths WHERE path = ?", [path])?;
    Ok(())
}

pub fn get_steam_cache(conn: &Connection) -> Result<HashMap<String, SteamCacheEntry>> {
    let mut stmt = conn.prepare("SELECT base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, release_date, last_updated, genres FROM steam_cache")?;
    let rows = stmt.query_map([], |row| {
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

    let mut cache = HashMap::new();
    for row in rows {
        let entry = row?;
        cache.insert(entry.base_name.clone(), entry);
    }
    Ok(cache)
}

pub fn clear_steam_cache(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM steam_cache", [])?;
    Ok(())
}

pub fn insert_steam_cache_entry(conn: &Connection, entry: &SteamCacheEntry) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO steam_cache (base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, release_date, last_updated, genres)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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

pub fn save_scanned_games(conn: &Connection, games: &[Game]) -> Result<()> {
    // 清除旧扫描结果
    conn.execute("DELETE FROM games", [])?;

    // 批量插入
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO games (original_name, clean_name, base_name, type, source_path, full_path, size, size_bytes, created, is_exact_dup, is_version_dup, is_representative)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )?;

    for g in games {
        stmt.execute(params![
            g.original_name,
            g.clean_name,
            g.base_name,
            g.r#type,
            g.source_path,
            g.full_path,
            g.size,
            g.size_bytes,
            g.created,
            g.is_exact_dup as i32,
            g.is_version_dup as i32,
            g.is_representative as i32
        ])?;
    }

    Ok(())
}

pub fn get_games_stats(conn: &Connection) -> Result<StatsSummary> {
    let total_scan: i64 = conn.query_row("SELECT count(*) FROM games", [], |row| row.get(0))?;
    let unique_games: i64 = conn.query_row("SELECT count(*) FROM games WHERE is_representative = 1", [], |row| row.get(0))?;
    let exact_dups: i64 = conn.query_row("SELECT count(*) FROM games WHERE is_exact_dup = 1", [], |row| row.get(0))?;
    let version_dups: i64 = conn.query_row("SELECT count(*) FROM games WHERE is_version_dup = 1", [], |row| row.get(0))?;

    let franchise_count: i64 = conn.query_row(
        "SELECT count(*) FROM (
            SELECT substr(clean_name, 1, 4) as prefix
            FROM games
            WHERE length(clean_name) >= 4
            GROUP BY prefix
            HAVING count(*) > 1
        )",
        [],
        |row| row.get(0)
    ).unwrap_or(0);

    Ok(StatsSummary {
        total_scan,
        unique_games,
        franchise_count,
        exact_dups,
        version_dups,
    })
}

pub fn get_games_list(
    conn: &Connection,
    search: &str,
    drive: &str,
    r#type: &str,
    rating: &str,
    sort: &str,
    only_representatives: bool,
    only_installed: bool,
) -> Result<Vec<Game>> {
    let mut query = String::from(
        "SELECT g.id, g.original_name, g.clean_name, g.base_name, g.type, g.source_path, g.full_path, g.size, g.size_bytes, g.created, g.is_exact_dup, g.is_version_dup, g.is_representative,
                s.appid, s.name, s.local_cover, s.review_score_desc, s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_cache s ON g.base_name = s.base_name
         WHERE 1=1"
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if only_representatives {
        query.push_str(" AND g.is_representative = 1");
    }

    if only_installed {
        query.push_str(" AND (g.source_path LIKE 'D:%' OR g.source_path LIKE 'E:%')");
    }

    if !search.is_empty() {
        query.push_str(" AND (g.original_name LIKE ?1 OR g.full_path LIKE ?1 OR g.base_name LIKE ?1)");
        params_vec.push(Box::new(format!("%{}%", search)));
    }

    if !drive.is_empty() {
        let param_index = params_vec.len() + 1;
        query.push_str(&format!(" AND g.source_path = ?{}", param_index));
        params_vec.push(Box::new(drive.to_string()));
    }

    if !r#type.is_empty() {
        let param_index = params_vec.len() + 1;
        query.push_str(&format!(" AND s.genres LIKE ?{}", param_index));
        params_vec.push(Box::new(format!("%{}%", r#type)));
    }

    if !rating.is_empty() {
        let param_index = params_vec.len() + 1;
        query.push_str(&format!(" AND s.review_score_desc = ?{}", param_index));
        params_vec.push(Box::new(rating.to_string()));
    }

    // 排序逻辑
    match sort {
        "name-asc" => query.push_str(" ORDER BY g.original_name COLLATE NOCASE ASC"),
        "name-desc" => query.push_str(" ORDER BY g.original_name COLLATE NOCASE DESC"),
        "steam-desc" => query.push_str(" ORDER BY CASE WHEN s.review_score_desc IS NULL THEN 0 ELSE 1 END DESC, s.positive_percent DESC, s.total_reviews DESC, g.original_name COLLATE NOCASE ASC"),
        "steam-asc" => query.push_str(" ORDER BY CASE WHEN s.review_score_desc IS NULL THEN 0 ELSE 1 END DESC, s.positive_percent ASC, s.total_reviews ASC, g.original_name COLLATE NOCASE ASC"),
        "size-desc" => query.push_str(" ORDER BY g.size_bytes DESC"),
        "size-asc" => query.push_str(" ORDER BY g.size_bytes ASC"),
        _ => query.push_str(" ORDER BY g.original_name COLLATE NOCASE ASC"),
    }

    let mut stmt = conn.prepare(&query)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    
    let rows = stmt.query_map(&param_refs[..], |row| {
        Ok(Game {
            id: row.get(0)?,
            original_name: row.get(1)?,
            clean_name: row.get(2)?,
            base_name: row.get(3)?,
            r#type: row.get(4)?,
            source_path: row.get(5)?,
            full_path: row.get(6)?,
            size: row.get(7)?,
            size_bytes: row.get(8)?,
            created: row.get(9)?,
            is_exact_dup: row.get::<_, i32>(10)? != 0,
            is_version_dup: row.get::<_, i32>(11)? != 0,
            is_representative: row.get::<_, i32>(12)? != 0,
            
            appid: row.get(13)?,
            name: row.get(14)?,
            local_cover: row.get(15)?,
            review_score_desc: row.get(16)?,
            positive_percent: row.get(17)?,
            total_reviews: row.get(18)?,
            release_date: row.get(19)?,
            genres: row.get(20)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn get_duplicates(conn: &Connection, dup_type: &str) -> Result<Vec<DuplicateGroup>> {
    // 1. 获取所有重复游戏
    let query = if dup_type == "exact" {
        "SELECT g.id, g.original_name, g.clean_name, g.base_name, g.type, g.source_path, g.full_path, g.size, g.size_bytes, g.created, g.is_exact_dup, g.is_version_dup, g.is_representative,
                s.appid, s.name, s.local_cover, s.review_score_desc, s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_cache s ON g.base_name = s.base_name
         WHERE g.is_exact_dup = 1
         ORDER BY g.base_name ASC, g.original_name ASC"
    } else {
        "SELECT g.id, g.original_name, g.clean_name, g.base_name, g.type, g.source_path, g.full_path, g.size, g.size_bytes, g.created, g.is_exact_dup, g.is_version_dup, g.is_representative,
                s.appid, s.name, s.local_cover, s.review_score_desc, s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_cache s ON g.base_name = s.base_name
         WHERE g.is_version_dup = 1
         ORDER BY g.base_name ASC, g.original_name ASC"
    };

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([], |row| {
        Ok(Game {
            id: row.get(0)?,
            original_name: row.get(1)?,
            clean_name: row.get(2)?,
            base_name: row.get(3)?,
            r#type: row.get(4)?,
            source_path: row.get(5)?,
            full_path: row.get(6)?,
            size: row.get(7)?,
            size_bytes: row.get(8)?,
            created: row.get(9)?,
            is_exact_dup: row.get::<_, i32>(10)? != 0,
            is_version_dup: row.get::<_, i32>(11)? != 0,
            is_representative: row.get::<_, i32>(12)? != 0,
            
            appid: row.get(13)?,
            name: row.get(14)?,
            local_cover: row.get(15)?,
            review_score_desc: row.get(16)?,
            positive_percent: row.get(17)?,
            total_reviews: row.get(18)?,
            release_date: row.get(19)?,
            genres: row.get(20)?,
        })
    })?;

    let mut grouped: HashMap<String, Vec<Game>> = HashMap::new();
    let mut order = Vec::new();

    for r in rows {
        let game = r?;
        let key = if dup_type == "exact" {
            // 完全重复按 CleanName 分组
            game.clean_name.clone()
        } else {
            // 版本重复按 BaseName 分组
            game.base_name.clone()
        };

        if !grouped.contains_key(&key) {
            order.push(key.clone());
        }
        grouped.entry(key).or_insert_with(Vec::new).push(game);
    }

    let mut result = Vec::new();
    for key in order {
        if let Some(games) = grouped.get(&key) {
            // 使用第一个条目的 base_name 或 original_name 作为分组标题
            let title = if dup_type == "exact" {
                games[0].original_name.clone()
            } else {
                key.to_uppercase()
            };
            result.push(DuplicateGroup {
                name: title,
                games: games.clone(),
            });
        }
    }

    Ok(result)
}

pub fn get_franchises(conn: &Connection) -> Result<Vec<FranchiseGroup>> {
    // 获取所有代表游戏用于系列分组
    let mut stmt = conn.prepare(
        "SELECT g.id, g.original_name, g.clean_name, g.base_name, g.type, g.source_path, g.full_path, g.size, g.size_bytes, g.created, g.is_exact_dup, g.is_version_dup, g.is_representative,
                s.appid, s.name, s.local_cover, s.review_score_desc, s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_cache s ON g.base_name = s.base_name
         WHERE g.is_representative = 1
         ORDER BY g.base_name ASC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Game {
            id: row.get(0)?,
            original_name: row.get(1)?,
            clean_name: row.get(2)?,
            base_name: row.get(3)?,
            r#type: row.get(4)?,
            source_path: row.get(5)?,
            full_path: row.get(6)?,
            size: row.get(7)?,
            size_bytes: row.get(8)?,
            created: row.get(9)?,
            is_exact_dup: row.get::<_, i32>(10)? != 0,
            is_version_dup: row.get::<_, i32>(11)? != 0,
            is_representative: row.get::<_, i32>(12)? != 0,
            
            appid: row.get(13)?,
            name: row.get(14)?,
            local_cover: row.get(15)?,
            review_score_desc: row.get(16)?,
            positive_percent: row.get(17)?,
            total_reviews: row.get(18)?,
            release_date: row.get(19)?,
            genres: row.get(20)?,
        })
    })?;

    let mut games = Vec::new();
    for r in rows {
        games.push(r?);
    }

    // 系列分组算法：通过共享前缀词检测同系列游戏
    let mut groups: HashMap<String, Vec<Game>> = HashMap::new();
    
    for g in games {
        let base = g.base_name.to_uppercase();
        let words: Vec<&str> = base.split_whitespace().collect();
        
        let prefix = if words.len() >= 3 && (words[1] == "OF" || words[1] == "THE" || words[1] == "DE" || words[1] == "A" || words[1] == "AND" || words[1] == "OR" || words[1] == "VS") {
            // 例如 "AGE OF WONDERS", "SINS OF A" -> 使用前 3 个词
            if words.len() >= 4 && words[2] == "A" {
                words[..4].join(" ")
            } else {
                words[..3].join(" ")
            }
        } else if words.len() >= 2 {
            // 例如 "ASSASSINS CREED", "DARK SOULS", "FINAL FANTASY"
            if words[0].len() <= 2 && words.len() >= 3 {
                words[..3].join(" ")
            } else {
                words[..2].join(" ")
            }
        } else {
            words.first().copied().unwrap_or("").to_string()
        };

        if !prefix.is_empty() && prefix.len() > 3 {
            groups.entry(prefix).or_insert_with(Vec::new).push(g);
        }
    }

    let mut result = Vec::new();
    for (prefix, group_games) in groups {
        // 只保留包含多于 1 个游戏的分组
        if group_games.len() > 1 {
            result.push(FranchiseGroup {
                prefix,
                games: group_games,
            });
        }
    }

    // 按字母顺序排序系列
    result.sort_by(|a, b| a.prefix.cmp(&b.prefix));

    Ok(result)
}

pub fn get_config(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?")?;
    let mut rows = stmt.query([key])?;
    if let Some(row) = rows.next()? {
        let val: String = row.get(0)?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

pub fn set_config(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)",
        [key, value],
    )?;
    Ok(())
}

/// 获取所有配置项，返回 HashMap<String, String>，用于前端一次性获取
pub fn get_all_config(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM config")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut config_map = HashMap::new();
    for row in rows {
        let (key, value) = row?;
        config_map.insert(key, value);
    }
    Ok(config_map)
}

pub fn get_all_genres(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT genres FROM steam_cache WHERE genres IS NOT NULL")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    
    let mut genre_set = std::collections::HashSet::new();
    for r in rows {
        if let Ok(genres_str) = r {
            for g in genres_str.split(',') {
                let g_trim = g.trim();
                if !g_trim.is_empty() {
                    genre_set.insert(g_trim.to_string());
                }
            }
        }
    }
    
    let mut genre_list: Vec<String> = genre_set.into_iter().collect();
    genre_list.sort();
    Ok(genre_list)
}

#[derive(serde::Serialize)]
pub struct GenreStat {
    pub name: String,
    pub count: usize,
}

pub fn get_genre_stats(conn: &Connection) -> Result<Vec<GenreStat>> {
    let mut stmt = conn.prepare(
        "SELECT s.genres FROM games g 
         JOIN steam_cache s ON g.base_name = s.base_name 
         WHERE g.is_representative = 1 AND s.genres IS NOT NULL"
    )?;
    
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for r in rows.flatten() {
        for g in r.split(',') {
            let genre = g.trim().to_string();
            if !genre.is_empty() {
                *counts.entry(genre).or_insert(0) += 1;
            }
        }
    }
    
    let mut stats: Vec<GenreStat> = counts.into_iter().map(|(name, count)| GenreStat { name, count }).collect();
    stats.sort_by(|a, b| b.count.cmp(&a.count));
    stats.truncate(15); // Show top 15 genres
    
    Ok(stats)
}

#[derive(serde::Serialize)]
pub struct RatingStat {
    pub name: String,
    pub count: usize,
}

pub fn get_rating_stats(conn: &Connection) -> Result<Vec<RatingStat>> {
    let mut stmt = conn.prepare(
        "SELECT s.review_score_desc FROM games g 
         JOIN steam_cache s ON g.base_name = s.base_name 
         WHERE g.is_representative = 1"
    )?;
    
    let rows = stmt.query_map([], |row| row.get::<_, Option<String>>(0))?;
    
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for r in rows.flatten() {
        let rating = r.unwrap_or_default().trim().to_string();
        
        let final_rating = if rating.is_empty() 
           || rating.contains("篇用户评测") 
           || rating.contains("user reviews") 
           || rating.contains("Need more user reviews")
           || rating.contains("不需要测评") 
        {
            "评价不可用".to_string()
        } else {
            rating
        };
        
        *counts.entry(final_rating).or_insert(0) += 1;
    }
    
    let mut stats: Vec<RatingStat> = counts.into_iter().map(|(name, count)| RatingStat { name, count }).collect();
    stats.sort_by(|a, b| {
        if a.name == "评价不可用" && b.name != "评价不可用" {
            std::cmp::Ordering::Greater
        } else if a.name != "评价不可用" && b.name == "评价不可用" {
            std::cmp::Ordering::Less
        } else {
            b.count.cmp(&a.count)
        }
    });
    
    Ok(stats)
}

/// 插入一条扫描历史记录
pub fn insert_scan_history(
    conn: &Connection,
    started_at: &str,
    completed_at: &str,
    total_scanned: i64,
    new_games: i64,
    new_steam_entries: i64,
    status: &str,
) -> Result<()> {
    let _ = conn.execute(
        "INSERT INTO scan_history (started_at, completed_at, total_scanned, new_games, new_steam_entries, status)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            started_at,
            completed_at,
            total_scanned,
            new_games,
            new_steam_entries,
            status
        ],
    );
    Ok(())
}

pub fn save_epic_free_games(conn: &Connection, games: &[EpicFreeGame]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO epic_free_games (id, title, description, status, start_date, end_date, image_url, game_url)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )?;

    for g in games {
        stmt.execute(params![
            g.id,
            g.title,
            g.description,
            g.status,
            g.start_date,
            g.end_date,
            g.image_url,
            g.game_url
        ])?;
    }
    
    Ok(())
}

pub fn get_epic_free_games(conn: &Connection) -> Result<Vec<EpicFreeGame>> {
    let mut stmt = conn.prepare("SELECT id, title, description, status, start_date, end_date, image_url, game_url FROM epic_free_games")?;
    let rows = stmt.query_map([], |row| {
        Ok(EpicFreeGame {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            status: row.get(3)?,
            start_date: row.get(4)?,
            end_date: row.get(5)?,
            image_url: row.get(6)?,
            game_url: row.get(7)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn save_steam_free_games(conn: &Connection, games: &[SteamFreeGame]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO steam_free_games (id, title, description, type, end_date, image_url, giveaway_url)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )?;

    for g in games {
        stmt.execute(params![
            g.id,
            g.title,
            g.description,
            g.r#type,
            g.end_date,
            g.image_url,
            g.giveaway_url
        ])?;
    }
    
    Ok(())
}

pub fn get_steam_free_games(conn: &Connection) -> Result<Vec<SteamFreeGame>> {
    let mut stmt = conn.prepare("SELECT id, title, description, type, end_date, image_url, giveaway_url FROM steam_free_games")?;
    let rows = stmt.query_map([], |row| {
        Ok(SteamFreeGame {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            r#type: row.get(3)?,
            end_date: row.get(4)?,
            image_url: row.get(5)?,
            giveaway_url: row.get(6)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// 获取最近 20 条扫描历史记录
pub fn get_scan_history(conn: &Connection) -> Result<Vec<ScanHistoryRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, completed_at, total_scanned, new_games, new_steam_entries, status
         FROM scan_history
         ORDER BY id DESC
         LIMIT 20"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ScanHistoryRecord {
            id: row.get(0)?,
            started_at: row.get(1)?,
            completed_at: row.get(2)?,
            total_scanned: row.get(3)?,
            new_games: row.get(4)?,
            new_steam_entries: row.get(5)?,
            status: row.get(6)?,
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Torrent1337x {
    pub id: Option<i64>,
    pub torrent_id: String,
    pub name: String,
    pub url: String,
    pub seeds: i64,
    pub leeches: i64,
    pub date: String,
    pub size: String,
    pub uploader: String,
    pub uploader_url: String,
    pub published_ts: i64,
}

pub fn get_torrents_1337x(conn: &Connection) -> Result<Vec<Torrent1337x>> {
    let mut stmt = conn.prepare(
        "SELECT id, torrent_id, name, url, seeds, leeches, date, size, uploader, uploader_url, published_ts
         FROM torrents_1337x
         ORDER BY published_ts DESC, id ASC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Torrent1337x {
            id: Some(row.get(0)?),
            torrent_id: row.get(1)?,
            name: row.get(2)?,
            url: row.get(3)?,
            seeds: row.get(4)?,
            leeches: row.get(5)?,
            date: row.get(6)?,
            size: row.get(7)?,
            uploader: row.get(8)?,
            uploader_url: row.get(9)?,
            published_ts: row.get(10).unwrap_or(0),
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

pub fn get_db_path() -> std::path::PathBuf {
    let mut exe_path = std::env::current_exe().unwrap_or_default();
    exe_path.pop(); // 移除 exe 文件名，保留目录
    exe_path.push("games.db");
    exe_path
}
