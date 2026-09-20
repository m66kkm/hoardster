use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use rusqlite::params;
use tauri::{AppHandle, Emitter};

pub struct DataCorrectionState {
    pub is_running: Arc<AtomicBool>,
    pub cancel_flag: Arc<AtomicBool>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct DataCorrectionProgress {
    pub is_running: bool,
    pub phase: String, // "idle", "reviews", "covers", "completed", "cancelled", "error"
    pub current: usize,
    pub total: usize,
    pub current_item: String,
    pub message: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct DataCorrectionStatus {
    pub is_running: bool,
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub last_run: Option<String>,
}

/// 执行完整的数据校准和清洗任务
pub fn run_data_correction(
    app_handle: AppHandle,
    cancel_flag: Arc<AtomicBool>,
    is_running_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    is_running_flag.store(true, Ordering::SeqCst);
    cancel_flag.store(false, Ordering::SeqCst);

    let result = (|| -> Result<(), String> {
        // 1. 获取主数据库连接
        let conn = crate::db::get_connection().map_err(|e| e.to_string())?;

        let lang = crate::db::get_config(&conn, "language")
            .unwrap_or_default()
            .unwrap_or_else(|| "schinese".to_string());

        // 2. 检索需要纠正 Steam 评测的游戏 (appid 存在但 positive_percent 为空或 review_score_desc 为 0)
        let mut query_stmt = conn.prepare(
            "SELECT DISTINCT appid, base_name 
             FROM steam_db.steam_cache 
             WHERE appid IS NOT NULL AND appid > 0 
               AND (positive_percent IS NULL OR review_score_desc = 0)"
        ).map_err(|e| e.to_string())?;

        let review_targets: Vec<(i64, String)> = query_stmt.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?))
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

        drop(query_stmt);

        let total_reviews_to_fix = review_targets.len();

        if total_reviews_to_fix > 0 {
            let client = reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| e.to_string())?;

            for (idx, (appid, base_name)) in review_targets.iter().enumerate() {
                if cancel_flag.load(Ordering::Relaxed) {
                    let _ = app_handle.emit("data_correction_progress", DataCorrectionProgress {
                        is_running: false,
                        phase: "cancelled".to_string(),
                        current: idx,
                        total: total_reviews_to_fix,
                        current_item: base_name.clone(),
                        message: "数据校准任务已取消".to_string(),
                    });
                    return Ok(());
                }

                let current_num = idx + 1;
                let _ = app_handle.emit("data_correction_progress", DataCorrectionProgress {
                    is_running: true,
                    phase: "reviews".to_string(),
                    current: current_num,
                    total: total_reviews_to_fix,
                    current_item: base_name.clone(),
                    message: format!("正在校准 Steam 评测 ({}/{}): {}", current_num, total_reviews_to_fix, base_name),
                });

                // Steam 接口安全限速间隔 (350ms)
                std::thread::sleep(Duration::from_millis(350));

                let url = format!(
                    "https://store.steampowered.com/appreviews/{}?json=1&language=all&l={}&purchase_type=all",
                    appid, lang
                );

                if let Ok(resp) = client.get(&url).send() {
                    let status = resp.status();
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status == reqwest::StatusCode::FORBIDDEN {
                        let _ = app_handle.emit("data_correction_progress", DataCorrectionProgress {
                            is_running: true,
                            phase: "reviews".to_string(),
                            current: current_num,
                            total: total_reviews_to_fix,
                            current_item: base_name.clone(),
                            message: "遇到 Steam 请求频率限制，稍候 5 秒后继续...".to_string(),
                        });
                        std::thread::sleep(Duration::from_secs(5));
                        continue;
                    }

                    if status.is_success() {
                        if let Ok(json) = resp.json::<serde_json::Value>() {
                            if let Some(qs) = json.get("query_summary") {
                                let score = qs.get("review_score").and_then(|d| d.as_i64()).unwrap_or(0);
                                let total_rev = qs.get("total_reviews").and_then(|t| t.as_f64()).unwrap_or(0.0);
                                let pos_rev = qs.get("total_positive").and_then(|t| t.as_f64()).unwrap_or(0.0);

                                let (desc, pct, revs) = if total_rev >= 10.0 && score > 0 {
                                    (Some(score as i32), Some(((pos_rev / total_rev) * 100.0) as i64), Some(total_rev as i64))
                                } else if total_rev > 0.0 {
                                    (Some(score as i32), None, Some(total_rev as i64))
                                } else {
                                    (Some(0), None, Some(0))
                                };

                                let _ = conn.execute(
                                    "UPDATE steam_db.steam_cache 
                                     SET review_score_desc = ?1, positive_percent = ?2, total_reviews = ?3 
                                     WHERE appid = ?4",
                                    params![desc, pct, revs, appid],
                                );
                            }
                        }
                    }
                }
            }
        }

        // 3. 阶段二：校准并补充竖版封面海报
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app_handle.emit("data_correction_progress", DataCorrectionProgress {
                is_running: false,
                phase: "cancelled".to_string(),
                current: total_reviews_to_fix,
                total: total_reviews_to_fix,
                current_item: "".to_string(),
                message: "数据校准任务已取消".to_string(),
            });
            return Ok(());
        }

        let _ = app_handle.emit("data_correction_progress", DataCorrectionProgress {
            is_running: true,
            phase: "covers".to_string(),
            current: total_reviews_to_fix,
            total: total_reviews_to_fix,
            current_item: "".to_string(),
            message: "正在同步与校准竖版封面海报...".to_string(),
        });

        let _ = crate::db::sync_game_metadata_covers(&conn);

        // 4. 阶段三：标记完成
        let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        let _ = crate::db::set_config(&conn, "data_version", "1");
        let _ = crate::db::set_config(&conn, "data_correction_last_run", &now_str);

        let _ = app_handle.emit("data_correction_progress", DataCorrectionProgress {
            is_running: false,
            phase: "completed".to_string(),
            current: total_reviews_to_fix,
            total: total_reviews_to_fix,
            current_item: "".to_string(),
            message: format!("历史数据校准已完成！(共校准 {} 款游戏)", total_reviews_to_fix),
        });

        Ok(())
    })();

    is_running_flag.store(false, Ordering::SeqCst);
    result
}

#[tauri::command]
pub fn trigger_data_correction_command(
    app_handle: AppHandle,
    state: tauri::State<'_, DataCorrectionState>,
) -> Result<(), String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("数据校准任务正在运行中，请勿重复启动".to_string());
    }

    let cancel_flag = Arc::clone(&state.cancel_flag);
    let is_running_flag = Arc::clone(&state.is_running);

    std::thread::spawn(move || {
        let _ = run_data_correction(app_handle, cancel_flag, is_running_flag);
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_data_correction_command(
    state: tauri::State<'_, DataCorrectionState>,
) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn get_data_correction_status_command(
    state: tauri::State<'_, DataCorrectionState>,
    db_state: tauri::State<'_, crate::db::DbState>,
) -> Result<DataCorrectionStatus, String> {
    let is_running = state.is_running.load(Ordering::SeqCst);
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let last_run = crate::db::get_config(&conn, "data_correction_last_run")
        .unwrap_or_default();

    Ok(DataCorrectionStatus {
        is_running,
        phase: if is_running { "running".to_string() } else { "idle".to_string() },
        current: 0,
        total: 0,
        last_run,
    })
}
