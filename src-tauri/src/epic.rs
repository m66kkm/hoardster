use crate::db::{EpicFreeGame, get_epic_free_games, save_epic_free_games, DbState};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

pub fn fetch_and_save_epic_games(conn: &rusqlite::Connection) -> Result<Vec<EpicFreeGame>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://store-site-backend-static.ak.epicgames.com/freeGamesPromotions?locale=zh-CN&country=CN&allowCountries=CN";
    let res = client.get(url).send().map_err(|e| e.to_string())?;
    
    let json: Value = res.json().map_err(|e| e.to_string())?;
    
    let elements = json
        .pointer("/data/Catalog/searchStore/elements")
        .and_then(|e| e.as_array())
        .ok_or_else(|| "Failed to parse elements from Epic API".to_string())?;

    let mut games = Vec::new();

    for el in elements {
        let id = el.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        let title = el.get("title").and_then(|v| v.as_str()).unwrap_or("未知游戏").to_string();
        let description = el.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut image_url = String::new();
        if let Some(images) = el.get("keyImages").and_then(|i| i.as_array()) {
            for img in images {
                if let Some(url) = img.get("url").and_then(|v| v.as_str()) {
                    let type_str = img.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if type_str == "OfferImageWide" || type_str == "Thumbnail" {
                        image_url = url.to_string();
                        break;
                    } else if image_url.is_empty() {
                        image_url = url.to_string();
                    }
                }
            }
        }

        let mut game_url = String::new();
        if let Some(slug) = el.get("productSlug").and_then(|v| v.as_str()) {
            game_url = format!("https://store.epicgames.com/zh-CN/p/{}", slug.replace("/home", ""));
        } else if let Some(offer_mappings) = el.get("offerMappings").and_then(|m| m.as_array()) {
            if let Some(mapping) = offer_mappings.first() {
                if let Some(page_slug) = mapping.get("pageSlug").and_then(|v| v.as_str()) {
                    game_url = format!("https://store.epicgames.com/zh-CN/p/{}", page_slug);
                }
            }
        } else if let Some(catalog_mappings) = el.pointer("/catalogNs/mappings").and_then(|m| m.as_array()) {
            if let Some(mapping) = catalog_mappings.first() {
                if let Some(page_slug) = mapping.get("pageSlug").and_then(|v| v.as_str()) {
                    game_url = format!("https://store.epicgames.com/zh-CN/p/{}", page_slug);
                }
            }
        } else if let Some(slug) = el.get("urlSlug").and_then(|v| v.as_str()) {
            game_url = format!("https://store.epicgames.com/zh-CN/p/{}", slug);
        }

        let promotions = el.get("promotions");
        
        let mut status = String::new();
        let mut start_date = String::new();
        let mut end_date = String::new();

        if let Some(promos) = promotions {
            if !promos.is_null() {
                // Check current promotions
                if let Some(offers) = promos.pointer("/promotionalOffers/0/promotionalOffers")
                    .and_then(|o| o.as_array())
                    .and_then(|a| a.first()) 
                {
                    let discount_pct = offers.pointer("/discountSetting/discountPercentage").and_then(|v| v.as_i64()).unwrap_or(-1);
                    if discount_pct == 0 {
                        status = "现在免费".to_string();
                        start_date = offers.get("startDate").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        end_date = offers.get("endDate").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    }
                }
                
                // If not currently free, check upcoming promotions
                if status.is_empty() {
                    if let Some(offers) = promos.pointer("/upcomingPromotionalOffers/0/promotionalOffers")
                        .and_then(|o| o.as_array())
                        .and_then(|a| a.first()) 
                    {
                        let discount_pct = offers.pointer("/discountSetting/discountPercentage").and_then(|v| v.as_i64()).unwrap_or(-1);
                        if discount_pct == 0 {
                            status = "即将推出".to_string();
                            start_date = offers.get("startDate").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            end_date = offers.get("endDate").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        }
                    }
                }
            }
        }

        // Only include games that have actual promotions (either now or upcoming)
        // Some items are just free-to-play base games, which don't have promotional offers date
        if status.is_empty() {
            continue;
        }

        games.push(EpicFreeGame {
            id,
            title,
            description,
            status,
            start_date,
            end_date,
            image_url,
            game_url,
        });
    }

    if games.is_empty() {
        return Err("No promotional games found".to_string());
    }

    save_epic_free_games(conn, &games).map_err(|e| e.to_string())?;

    Ok(games)
}

#[tauri::command]
pub fn fetch_epic_free_games_command(state: tauri::State<'_, DbState>) -> Result<Vec<EpicFreeGame>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    fetch_and_save_epic_games(&conn)
}

#[tauri::command]
pub fn get_epic_free_games_command(state: tauri::State<'_, DbState>) -> Result<Vec<EpicFreeGame>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_epic_free_games(&conn).map_err(|e| e.to_string())
}
