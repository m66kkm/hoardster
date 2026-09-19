use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::steam_service::SteamCacheEntry;

/// 数据库连接状态，通过 Mutex 保护 Connection 实现线程安全共享
pub struct DbState(pub Mutex<Connection>);

/// 获取并配置主数据库连接（并自动附加 Steam 数据库）
pub fn get_connection() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;
    
    // Set busy_timeout FIRST so WAL switch can wait for any stale locks
    conn.pragma_update(None, "busy_timeout", 10000).ok();
    // Enable WAL mode for concurrent read/write (non-fatal if stale locks prevent it)
    conn.pragma_update(None, "journal_mode", "WAL").ok();

    // Attach steam_data.db
    let steam_db_path = crate::steam_service::get_steam_db_path();
    conn.execute(
        &format!("ATTACH DATABASE '{}' AS steam_db", steam_db_path.to_string_lossy().replace("'", "''")),
        [],
    )?;
    
    // Re-apply busy_timeout after ATTACH to ensure it covers attached db operations
    conn.pragma_update(None, "busy_timeout", 10000)?;

    Ok(conn)
}

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
    pub review_score_desc: Option<i32>,
    pub positive_percent: Option<i64>,
    pub total_reviews: Option<i64>,
    pub release_date: Option<String>,
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
    pub status: String,
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

    // 确保 attached 的 steam_db.steam_cache 表存在
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_db.steam_cache (
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

    // 迁移旧的字符串评价数据为整数 (Steam review_score_desc values)
    let _ = conn.execute_batch(
        "UPDATE steam_db.steam_cache SET review_score_desc = 9 WHERE review_score_desc = '好评如潮' OR review_score_desc = 'Overwhelmingly Positive';
         UPDATE steam_db.steam_cache SET review_score_desc = 8 WHERE review_score_desc = '特别好评' OR review_score_desc = 'Very Positive';
         UPDATE steam_db.steam_cache SET review_score_desc = 7 WHERE review_score_desc = '好评' OR review_score_desc = 'Positive';
         UPDATE steam_db.steam_cache SET review_score_desc = 6 WHERE review_score_desc = '多半好评' OR review_score_desc = 'Mostly Positive';
         UPDATE steam_db.steam_cache SET review_score_desc = 5 WHERE review_score_desc = '褒贬不一' OR review_score_desc = 'Mixed';
         UPDATE steam_db.steam_cache SET review_score_desc = 4 WHERE review_score_desc = '多半差评' OR review_score_desc = 'Mostly Negative';
         UPDATE steam_db.steam_cache SET review_score_desc = 3 WHERE review_score_desc = '差评' OR review_score_desc = 'Negative';
         UPDATE steam_db.steam_cache SET review_score_desc = 2 WHERE review_score_desc = '特别差评' OR review_score_desc = 'Very Negative';
         UPDATE steam_db.steam_cache SET review_score_desc = 1 WHERE review_score_desc = '差评如潮' OR review_score_desc = 'Overwhelmingly Negative';
         UPDATE steam_db.steam_cache SET review_score_desc = 0 WHERE typeof(review_score_desc) = 'text' AND CAST(review_score_desc AS INTEGER) = 0 AND review_score_desc != '0';"
    );

    // 清理此前网络受限或失败写入的 appid 为 NULL 的无效缓存记录，以便重新向 Steam 查询
    let _ = conn.execute("DELETE FROM steam_db.steam_cache WHERE appid IS NULL", []);
    // 清理评测不足 (score = 0 或 total_reviews < 10) 的误报百分比
    let _ = conn.execute("UPDATE steam_db.steam_cache SET positive_percent = NULL WHERE review_score_desc = 0 OR total_reviews < 10", []);

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
            published_ts INTEGER DEFAULT 0,
            base_name TEXT
        )",
        [],
    )?;

    // Add published_ts column if not exists (for existing databases)
    conn.execute("ALTER TABLE torrents_1337x ADD COLUMN published_ts INTEGER DEFAULT 0", []).ok();
    // Add base_name column if not exists (for existing databases)
    conn.execute("ALTER TABLE torrents_1337x ADD COLUMN base_name TEXT", []).ok();

    // 自动为已有但缺少 base_name 的记录补充清洗后的 base_name
    let unpopulated_1337: Vec<(i64, String)> = {
        let stmt = conn.prepare("SELECT id, name FROM torrents_1337x WHERE base_name IS NULL OR TRIM(base_name) = ''").ok();
        if let Some(mut stmt) = stmt {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    };
    if !unpopulated_1337.is_empty() {
        if let Ok(mut update_stmt) = conn.prepare("UPDATE torrents_1337x SET base_name = ? WHERE id = ?") {
            for (id, name) in unpopulated_1337 {
                let base = crate::scanner::base_game_name(&name);
                let _ = update_stmt.execute(params![base, id]);
            }
        }
    }

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
            giveaway_url TEXT,
            status TEXT DEFAULT '活跃'
        )",
        [],
    )?;

    // 9. Skidrow & Reloaded
    conn.execute(
        "CREATE TABLE IF NOT EXISTS skidrow_reloaded (
            id TEXT PRIMARY KEY,
            title TEXT,
            url TEXT,
            image_url TEXT,
            category TEXT,
            date TEXT,
            fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            published_ts INTEGER DEFAULT 0,
            comments INTEGER DEFAULT 0,
            base_name TEXT
        )",
        [],
    )?;

    // Add comments column if not exists
    conn.execute("ALTER TABLE skidrow_reloaded ADD COLUMN comments INTEGER DEFAULT 0", []).ok();
    // Add base_name column if not exists
    conn.execute("ALTER TABLE skidrow_reloaded ADD COLUMN base_name TEXT", []).ok();

    // 自动为已有但缺少 base_name 的记录补充清洗后的 base_name
    let unpopulated: Vec<(String, String)> = {
        let stmt = conn.prepare("SELECT id, title FROM skidrow_reloaded WHERE base_name IS NULL OR TRIM(base_name) = ''").ok();
        if let Some(mut stmt) = stmt {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    };
    if !unpopulated.is_empty() {
        if let Ok(mut update_stmt) = conn.prepare("UPDATE skidrow_reloaded SET base_name = ? WHERE id = ?") {
            let _ = conn.execute_batch("BEGIN TRANSACTION;");
            for (id, title) in unpopulated {
                let base = crate::scanner::base_game_name(&title);
                let _ = update_stmt.execute(params![base, id]);
            }
            let _ = conn.execute_batch("COMMIT;");
        }
    }

    // 为 base_name 建立索引以加速多表关联查询 (LEFT JOIN steam_db.steam_cache)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_games_base_name ON games(base_name)", []).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_sr_base_name ON skidrow_reloaded(base_name)", []).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_1337_base_name ON torrents_1337x(base_name)", []).ok();

    // Add status column if not exists (for existing databases)
    conn.execute("ALTER TABLE steam_free_games ADD COLUMN status TEXT DEFAULT '活跃'", []).ok();

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
    let cache_count: i64 = conn.query_row("SELECT count(*) FROM steam_db.steam_cache", [], |r| r.get(0))?;
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
                        let review_score_desc = val.get("ReviewScoreDesc").and_then(|v| v.as_i64()).map(|s| s as i32);
                        let positive_percent = val.get("PositivePercent").and_then(|v| v.as_i64()).or_else(|| val.get("PositivePercent").and_then(|v| v.as_f64()).map(|f| f as i64));
                        let total_reviews = val.get("TotalReviews").and_then(|v| v.as_i64());
                        let release_date = val.get("ReleaseDate").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let genres = val.get("Genres").and_then(|v| v.as_str()).map(|s| s.to_string());
                        
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO steam_db.steam_cache (base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, release_date, last_updated, genres)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
    let mut stmt = conn.prepare("SELECT base_name, appid, name, local_cover, CAST(review_score_desc AS INTEGER), positive_percent, total_reviews, release_date, last_updated, genres FROM steam_db.steam_cache")?;
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
    conn.execute("DROP TABLE IF EXISTS steam_db.steam_cache", [])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS steam_db.steam_cache (
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

pub fn insert_steam_cache_entry(conn: &Connection, entry: &SteamCacheEntry) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO steam_db.steam_cache (base_name, appid, name, local_cover, review_score_desc, positive_percent, total_reviews, release_date, last_updated, genres)
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
                s.appid, s.name, s.local_cover, CAST(s.review_score_desc AS INTEGER), s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_db.steam_cache s ON g.base_name = s.base_name
         WHERE 1=1"
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if only_representatives {
        query.push_str(" AND g.is_representative = 1");
    }

    if only_installed {
        query.push_str(" AND g.type = 'Installed'");
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
        query.push_str(&format!(" AND CAST(s.review_score_desc AS INTEGER) = ?{}", param_index));
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
                s.appid, s.name, s.local_cover, CAST(s.review_score_desc AS INTEGER), s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_db.steam_cache s ON g.base_name = s.base_name
         WHERE g.is_exact_dup = 1
         ORDER BY g.base_name ASC, g.original_name ASC"
    } else {
        "SELECT g.id, g.original_name, g.clean_name, g.base_name, g.type, g.source_path, g.full_path, g.size, g.size_bytes, g.created, g.is_exact_dup, g.is_version_dup, g.is_representative,
                s.appid, s.name, s.local_cover, CAST(s.review_score_desc AS INTEGER), s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_db.steam_cache s ON g.base_name = s.base_name
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
                s.appid, s.name, s.local_cover, CAST(s.review_score_desc AS INTEGER), s.positive_percent, s.total_reviews, s.release_date, s.genres
         FROM games g
         LEFT JOIN steam_db.steam_cache s ON g.base_name = s.base_name
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
    let mut stmt = conn.prepare("SELECT genres FROM steam_db.steam_cache WHERE genres IS NOT NULL")?;
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
         JOIN steam_db.steam_cache s ON g.base_name = s.base_name 
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
        "SELECT CAST(s.review_score_desc AS INTEGER) FROM games g 
         JOIN steam_db.steam_cache s ON g.base_name = s.base_name 
         WHERE g.is_representative = 1"
    )?;
    
    let rows = stmt.query_map([], |row| row.get::<_, Option<i32>>(0))?;
    
    let mut counts: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
    
    for r in rows.flatten() {
        let final_rating = r.unwrap_or(0);
        *counts.entry(final_rating).or_insert(0) += 1;
    }
    
    let mut stats: Vec<RatingStat> = counts.into_iter().map(|(name, count)| RatingStat { name: name.to_string(), count }).collect();
    stats.sort_by(|a, b| {
        if a.name == "0" && b.name != "0" {
            std::cmp::Ordering::Greater
        } else if a.name != "0" && b.name == "0" {
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
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE epic_free_games SET status = '已结束' WHERE status != '已结束'", [])?;

    {
        let mut stmt = tx.prepare(
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
    }
    
    tx.commit()?;
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
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE steam_free_games SET status = '已结束' WHERE status != '已结束'", [])?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO steam_free_games (id, title, description, type, end_date, image_url, giveaway_url, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        for g in games {
            stmt.execute(params![
                g.id,
                g.title,
                g.description,
                g.r#type,
                g.end_date,
                g.image_url,
                g.giveaway_url,
                g.status
            ])?;
        }
    }
    
    tx.commit()?;
    Ok(())
}

pub fn get_steam_free_games(conn: &Connection) -> Result<Vec<SteamFreeGame>> {
    let mut stmt = conn.prepare("SELECT id, title, description, type, end_date, image_url, giveaway_url, status FROM steam_free_games")?;
    let rows = stmt.query_map([], |row| {
        Ok(SteamFreeGame {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            r#type: row.get(3)?,
            end_date: row.get(4)?,
            image_url: row.get(5)?,
            giveaway_url: row.get(6)?,
            status: row.get(7)?,
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
    pub base_name: Option<String>,
    pub appid: Option<i64>,
    pub review_score_desc: Option<i32>,
    pub positive_percent: Option<i64>,
    pub total_reviews: Option<i64>,
}

pub fn get_torrents_1337x(conn: &Connection) -> Result<Vec<Torrent1337x>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.torrent_id, t.name, t.url, t.seeds, t.leeches, t.date, t.size, t.uploader, t.uploader_url, t.published_ts,
                t.base_name, s.appid, CAST(s.review_score_desc AS INTEGER), s.positive_percent, s.total_reviews
         FROM torrents_1337x t
         LEFT JOIN steam_db.steam_cache s ON t.base_name = s.base_name
         ORDER BY t.published_ts DESC, t.id ASC"
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
            base_name: row.get(11)?,
            appid: row.get(12)?,
            review_score_desc: row.get(13)?,
            positive_percent: row.get(14)?,
            total_reviews: row.get(15)?,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
/// TorrentSR represents a record from the skidrow_reloaded table
pub struct TorrentSR {
    pub id: String,
    pub title: String,
    pub url: String,
    pub image_url: String,
    pub category: String,
    pub date: String,
    pub fetched_at: String,
    pub published_ts: i64,
    pub comments: i32,
    pub base_name: Option<String>,
    pub appid: Option<i64>,
    pub review_score_desc: Option<i32>,
    pub positive_percent: Option<i64>,
    pub total_reviews: Option<i64>,
}

/// Fetches records from the skidrow_reloaded table with Steam reviews joined
pub fn get_torrents_sr(conn: &Connection) -> Result<Vec<TorrentSR>> {
    let mut stmt = conn.prepare(
        "SELECT sr.id, sr.title, sr.url, sr.image_url, sr.category, sr.date, sr.fetched_at, sr.published_ts, sr.comments,
                sr.base_name, s.appid, CAST(s.review_score_desc AS INTEGER), s.positive_percent, s.total_reviews
         FROM skidrow_reloaded sr
         LEFT JOIN steam_db.steam_cache s ON sr.base_name = s.base_name
         ORDER BY sr.published_ts DESC"
    )?;
    
    let iter = stmt.query_map([], |row| {
        Ok(TorrentSR {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            image_url: row.get(3).unwrap_or_default(),
            category: row.get(4).unwrap_or_default(),
            date: row.get(5).unwrap_or_default(),
            fetched_at: row.get(6)?,
            published_ts: row.get(7).unwrap_or(0),
            comments: row.get(8).unwrap_or(0),
            base_name: row.get(9)?,
            appid: row.get(10)?,
            review_score_desc: row.get(11)?,
            positive_percent: row.get(12)?,
            total_reviews: row.get(13)?,
        })
    })?;

    let mut list = Vec::new();
    for item in iter {
        list.push(item?);
    }
    Ok(list)
}
