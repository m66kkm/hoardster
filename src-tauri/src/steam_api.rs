use crate::db::{SteamFreeGame, get_steam_free_games, save_steam_free_games, DbState};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

pub fn fetch_and_save_steam_games(conn: &rusqlite::Connection) -> Result<Vec<SteamFreeGame>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://www.gamerpower.com/api/giveaways?platform=steam";
    let res = client.get(url).send().map_err(|e| e.to_string())?;
    
    // API returns a list, or an object if there's an error/no active giveaways
    let json: Value = res.json().map_err(|e| e.to_string())?;
    
    let mut games = Vec::new();

    if let Some(arr) = json.as_array() {
        for el in arr {
            let id = el.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let title = el.get("title").and_then(|v| v.as_str()).unwrap_or("未知游戏").to_string();
            let description = el.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let end_date = el.get("end_date").and_then(|v| v.as_str()).unwrap_or("未知时间").to_string();
            let image_url = el.get("image").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let giveaway_url = el.get("open_giveaway").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let r#type = el.get("type").and_then(|v| v.as_str()).unwrap_or("Game").to_string();

            games.push(SteamFreeGame {
                id,
                title,
                description,
                r#type,
                end_date,
                image_url,
                giveaway_url,
            });
        }
    } else {
        // It might be an error object like {"status": 0, "status_message": "No active giveaways..."}
        // which means no current giveaways, so we return an empty list.
    }

    // Save to database
    save_steam_free_games(conn, &games).map_err(|e| e.to_string())?;

    Ok(games)
}

#[tauri::command]
pub fn fetch_steam_free_games_command(state: tauri::State<'_, DbState>) -> Result<Vec<SteamFreeGame>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    fetch_and_save_steam_games(&conn)
}

#[tauri::command]
pub fn get_steam_free_games_command(state: tauri::State<'_, DbState>) -> Result<Vec<SteamFreeGame>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_steam_free_games(&conn).map_err(|e| e.to_string())
}
