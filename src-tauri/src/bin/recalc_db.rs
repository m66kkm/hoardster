use hoardster_lib::db::{get_connection, insert_steam_cache_entry, get_config};
use hoardster_lib::steam_service::{fetch_steam_game_info_ext, fetch_steam_game_by_appid_ext};
use hoardster_lib::scanner::detect_steam_appid_from_game_dir;
use reqwest::blocking::Client;
use std::time::Duration;
use std::fs;
use std::collections::BTreeMap;

fn main() {
    println!("=== 开始根据最新规则对数据库进行全量重新计算与缓存清洗 ===");

    let conn = match get_connection() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接数据库: {}", e);
            return;
        }
    };

    // 1. 执行全量名称清洗、去重与续作漂移缓存清理
    match hoardster_lib::db::recalculate_existing_games(&conn) {
        Ok((total, changed, purged)) => {
            println!(">> 数据库重新计算完成：");
            println!("   - 处理游戏总数: {}", total);
            println!("   - 发生 base_name 修正的游戏数: {}", changed);
            println!("   - 从 steam_cache 中清理的续作漂移/错误缓存记录数: {}", purged);
        }
        Err(e) => {
            eprintln!("重新计算失败: {}", e);
            return;
        }
    }

    // 2. 查找当前 games 表中在 steam_cache 中缺少有效数据的游戏（优先保留代表游戏路径）
    let mut stmt = match conn.prepare(
        "SELECT g.base_name, g.full_path 
         FROM games g 
         LEFT JOIN steam_db.steam_cache s ON g.base_name = s.base_name 
         WHERE (s.appid IS NULL OR s.appid = 0 OR s.review_score_desc IS NULL)
         ORDER BY g.base_name, g.is_representative DESC"
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("查询待匹配游戏失败: {}", e);
            return;
        }
    };

    let mut pending_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).unwrap();

    for row in rows.filter_map(|r| r.ok()) {
        pending_map.entry(row.0).or_default().push(row.1);
    }
    drop(stmt);

    let total_pending = pending_map.len();
    println!("\n>> 当前有 {} 个游戏需要同步/补充 Steam 数据...", total_pending);

    if total_pending == 0 {
        println!("所有游戏均已有完整的 Steam 数据！");
        return;
    }

    let lang = get_config(&conn, "language")
        .unwrap_or_default()
        .unwrap_or_else(|| "schinese".to_string());

    let client = match Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(10))
        .build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("创建 HTTP 客户端失败: {}", e);
            return;
        }
    };

    let mut covers_dir = std::env::current_exe().unwrap_or_default();
    covers_dir.pop();
    covers_dir.push("covers");
    let _ = fs::create_dir_all(&covers_dir);

    let mut matched_count = 0;
    let mut skipped_count = 0;

    for (idx, (base_name, paths)) in pending_map.iter().enumerate() {
        // 先检查本地目录是否存在 AppID 配置文件
        let mut detected_appid = None;
        for path_str in paths {
            let p = std::path::Path::new(path_str);
            if p.is_dir() {
                if let Some(appid) = detect_steam_appid_from_game_dir(p) {
                    detected_appid = Some(appid);
                    break;
                }
            }
        }

        let (entry_opt, got_403) = if let Some(appid) = detected_appid {
            print!("[{}/{}] 正在匹配 '{}' (本地识别 AppID: {})... ", idx + 1, total_pending, base_name, appid);
            let (res, is_403) = fetch_steam_game_by_appid_ext(&client, appid, base_name, &lang);
            if is_403 {
                (None, true)
            } else if res.is_some() {
                (res, false)
            } else {
                fetch_steam_game_info_ext(&client, base_name, &lang)
            }
        } else {
            print!("[{}/{}] 正在匹配 '{}'... ", idx + 1, total_pending, base_name);
            fetch_steam_game_info_ext(&client, base_name, &lang)
        };

        if got_403 {
            println!("触发 Steam 频控 (403/429)，暂停抓取。");
            break;
        }

        if let Some(mut entry) = entry_opt {
            if let Some(appid) = entry.appid {
                if appid > 0 {
                    let s_name = entry.name.as_deref().unwrap_or("");
                    let from_tag = if detected_appid.is_some() { " [本地精准识别]" } else { "" };
                    println!("=> 匹配成功{}: AppID {} ('{}')", from_tag, appid, s_name);
                    
                    // 下载封面
                    if let Some(ref cover_url) = entry.local_cover {
                        if cover_url.starts_with("http") {
                            let ext = if cover_url.ends_with(".png") { "png" } else { "jpg" };
                            let filename = format!("{}.{}", appid, ext);
                            let cover_path = covers_dir.join(&filename);
                            let mut saved = cover_path.exists();
                            if !saved {
                                if let Ok(resp) = client.get(cover_url).send() {
                                    if let Ok(bytes) = resp.bytes() {
                                        if fs::write(&cover_path, &bytes).is_ok() {
                                            saved = true;
                                        }
                                    }
                                }
                            }
                            if saved {
                                entry.local_cover = Some(format!("covers/{}", filename));
                            }
                        }
                    }

                    let _ = insert_steam_cache_entry(&conn, &entry);
                    matched_count += 1;
                } else {
                    println!("=> 无匹配项");
                    skipped_count += 1;
                }
            } else {
                println!("=> 无匹配项");
                skipped_count += 1;
            }
        } else {
            println!("=> 查询失败");
            skipped_count += 1;
        }

        // 适度限速避免触发 Steam 限制
        std::thread::sleep(Duration::from_millis(350));
    }

    println!("\n=== 匹配处理完毕 ===");
    println!("- 成功更新匹配: {} 个", matched_count);
    println!("- 未找到或跳过: {} 个", skipped_count);
}
