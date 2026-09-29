use crate::db::{Game, get_scan_paths, get_steam_cache, get_config, save_scanned_games, insert_scan_history};
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use rayon::prelude::*;

lazy_static::lazy_static! {
    static ref DIVIDERS_RE: Regex = Regex::new(r"[\._\-/:;]").unwrap();
    static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
    static ref BRACKET_CONTENT_RE: Regex = Regex::new(r"\s*[\[\(].*?[\]\)]").unwrap();
    static ref BRACKETS_RE: Regex = Regex::new(r"[\[\]\(\)]").unwrap();
    // 仅精准匹配末尾已知的 Scene / P2P 发布组后缀，防止截断如 Kingdoms of Amalur Re-Reckoning 或 Marvel's Spider-Man
    static ref SCENE_GROUP_RE: Regex = Regex::new(r"(?i)-(rune|tenoke|codex|cpy|skidrow|reloaded|flt|empress|plaza|hoodlum|p2p|insaneramzes|voices38|razor1911|dinobytes|unleashed|delight|0xzeon|hypervisor|tinyiso|doge|kaos|anomaly|simplex|chronos|goldberg|darksiders|prophet|fairlight|hi2u|postmortem|ali213|3dm|vace|deviance|hatred|repack|dodi|fitgirl)$").unwrap();
    // 匹配如 v1.0.12 或 v.1.3065 带点前缀的版本号
    static ref VERSION_DOTTED_RE: Regex = Regex::new(r"(?i)\bv[\s\.]*\d+[\w\.]*\b").unwrap();
    static ref SEMVER_DOTTED_RE: Regex = Regex::new(r"\b\d+\.\d+\.\d+[\w\.]*\b").unwrap();
    static ref BUILD_RE: Regex = Regex::new(r"(?i)\bbuild[\s\.]*\d+\b").unwrap();
    static ref VERSION_SPACED_RE: Regex = Regex::new(r"(?i)\bv\s?\d+(\s+\d+)*\b").unwrap();
    static ref YEAR_RE: Regex = Regex::new(r"\b(199\d|20[0-2]\d|2030)\b").unwrap();
    // 多词组合版本前缀
    static ref KNOWN_EDITION_PREFIX_RE: Regex = Regex::new(r"(?i)\b(digital\s+deluxe|game\s+of\s+the\s+year|game\s+of\s+the\s+yorha|goty|day\s+one|fleet\s+command|director[s\x27]*\s+cut|final\s+cut|extended\s+cut)\s+(edition|cut)\b").unwrap();
    // 通用单修饰词 + edition，如 platinum edition, infernal edition, shadows edition, anniversary edition
    static ref GENERIC_EDITION_WORD_RE: Regex = Regex::new(r"(?i)\b([a-z]{3,})\s+edition\b").unwrap();
    static ref JUST_EDITION_RE: Regex = Regex::new(r"(?i)\bedition\b").unwrap();
    static ref JUST_CUT_RE: Regex = Regex::new(r"(?i)\b(directors?\s*cut|final\s*cut)\b").unwrap();
    static ref GENERIC_DLC_RE: Regex = Regex::new(r"(?i)\b(all\s*dlcs?|\d+\s*dlcs?|dlcs?\s*included|incl\s*dlc|expansion\s*pack|with\s+updates?|bonus\s*content)\b").unwrap();
    static ref APPID_INI_RE: Regex = Regex::new(r"(?i)^\s*(?:app_?id|target_?app_?id|steam_?app_?id|application_?id|game_?app_?id)\s*=\s*(\d{1,9})\b").unwrap();
    static ref APPID_TXT_RE: Regex = Regex::new(r"^\s*(\d{1,9})\b").unwrap();
    static ref ACF_RE: Regex = Regex::new(r"(?i)^appmanifest_(\d{1,9})\.acf$").unwrap();
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
    
    // 1. 剥离末尾已知的 Scene / P2P 发布组后缀
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
    // 注意：下划线 '_' 是 ASCII \w 字符，若版本号前带下划线（如 FallenDoll_0.4.9 或 Game_v1.0），
    // 单词边界 \b 会因前缀也是 \w 而失效，故先将下划线替换为空格以暴露单词边界！
    clean = clean.replace('_', " ");
    clean = VERSION_DOTTED_RE.replace_all(&clean, " ").into_owned();
    clean = SEMVER_DOTTED_RE.replace_all(&clean, " ").into_owned();
    clean = BUILD_RE.replace_all(&clean, " ").into_owned();

    clean = clean.replace('\'', "");
    clean = DIVIDERS_RE.replace_all(&clean, " ").into_owned();

    // 4. 通用模式剥离：先剥离 DLC/附加内容，再剥离多词或通用单词修饰的 Edition / Cut
    let stripped_dlc = GENERIC_DLC_RE.replace_all(&clean, " ").into_owned();
    if !stripped_dlc.trim().is_empty() {
        clean = stripped_dlc;
    }

    clean = KNOWN_EDITION_PREFIX_RE.replace_all(&clean, " ").into_owned();

    // 针对通用单词 + edition（如 platinum edition, infernal edition, shadows edition）
    // 保护罗马数字与续作编号不被当作修饰词剥离（如 Street Fighter V）
    let stripped_generic = GENERIC_EDITION_WORD_RE.replace_all(&clean, |caps: &regex::Captures| {
        let word = caps[1].to_lowercase();
        if matches!(word.as_str(), "i" | "ii" | "iii" | "iv" | "v" | "vi" | "vii" | "viii" | "ix" | "x" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10") {
            caps[0].to_string()
        } else {
            " ".to_string()
        }
    }).into_owned();
    if !stripped_generic.trim().is_empty() {
        clean = stripped_generic;
    }

    clean = JUST_EDITION_RE.replace_all(&clean, " ").into_owned();
    clean = JUST_CUT_RE.replace_all(&clean, " ").into_owned();
    
    let repack_tags = vec![
        "fitgirl repack", "fitgirl monkey repack", "decepticon repack", "dodi repack",
        "rune", "tenoke", "razor1911", "flt", "voices38", "cpy", "empress", "codex",
        "skidrow", "plaza", "hoodlum", "dinobytes", "unleashed", "delight", "insaneramzes", "p2p",
        "betav1.2.readnfo-mkdev", "read nfo", "readnfo", "proper", "repack", "pre-installed", "cracked",
        "reloaded", "rip", "unlocked", "multi\\s?\\d+", "remastered", "hrdc",
        "elamigos", "gog", "3dm", "ali213", "canek77", "wanterlude", "decepticon", "fitgirl", "dodi",
        "early\\s+access", "portable", "dlc\\s+unlocker", "incl\\s+dlc", "with\\s+update", "with\\s+up\\d+",
        "chs", "cht", "complete\\s+bundle", "bundle", "steam", "by\\s+\\w+",
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

const PRUNE_DIRS: &[&str] = &[
    "content", "paks", "assets", "streamingassets", "sound", "audio",
    "music", "video", "movies", "textures", "shadercache", "localization",
    ".git", "node_modules", "easyanticheat"
];

const INI_WHITELIST: &[&str] = &[
    "steam_emu.ini", "steam_api64.ini", "steam_api.ini", "steamconfig.ini",
    "flt.ini", "cpy.ini", "coldclientloader.ini", "hlm.ini", "ds.ini",
    "codex.ini", "stp-steam.ini", "3dmgame.ini", "ali213.ini",
    "smartsteamemu.ini", "creamapi.ini", "cream_api.ini"
];

/// 从游戏目录中精准提取 Steam AppID。
/// 采用三级过滤流水线：
/// 1. 后缀快速拦截（仅放行 .txt, .ini, .acf）
/// 2. 文件名过滤（.txt 仅限 steam_appid.txt；.ini 仅限白名单与特定关键字；.acf 提取 appmanifest_<appid>.acf）
/// 3. 体积防护（仅读取 < 128KB 文本文件）与目录智能剪枝
pub fn detect_steam_appid_from_game_dir<P: AsRef<Path>>(dir: P) -> Option<u32> {
    let base_path = dir.as_ref();
    if !base_path.exists() || !base_path.is_dir() {
        return None;
    }

    struct Candidate {
        priority: u8,
        appid: u32,
        depth: usize,
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut queue: VecDeque<(std::path::PathBuf, usize)> = VecDeque::new();
    queue.push_back((base_path.to_path_buf(), 0));

    while let Some((current_dir, depth)) = queue.pop_front() {
        if depth > 4 {
            continue;
        }

        let entries = match std::fs::read_dir(&current_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_string_lossy();
            let file_name_lower = file_name.to_lowercase();

            if file_type.is_dir() {
                if depth < 4 {
                    // 剪枝纯资源与无配置文件目录
                    if !PRUNE_DIRS.contains(&file_name_lower.as_str())
                        && !file_name_lower.starts_with('.')
                        && !file_name_lower.starts_with('$')
                    {
                        queue.push_back((entry.path(), depth + 1));
                    }
                }
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            // 规则 1：后缀快速拦截
            let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext != "ini" && ext != "txt" && ext != "acf" {
                continue;
            }

            // 规则 2：针对三种文件类型的模式匹配
            // 2a. Steam 原生清单文件 appmanifest_<appid>.acf
            if ext == "acf" {
                if let Some(caps) = ACF_RE.captures(&file_name_lower) {
                    if let Ok(appid) = caps[1].parse::<u32>() {
                        if appid > 0 && appid != 480 && appid != 228980 {
                            candidates.push(Candidate { priority: 1, appid, depth });
                        }
                    }
                }
                continue;
            }

            // 2b. steam_appid.txt
            if file_name_lower == "steam_appid.txt" {
                let file_path = entry.path();
                if let Ok(metadata) = entry.metadata() {
                    if metadata.len() < 65536 {
                        if let Ok(bytes) = std::fs::read(&file_path) {
                            let content = String::from_utf8_lossy(&bytes);
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') || trimmed.starts_with("//") {
                                    continue;
                                }
                                if let Some(caps) = APPID_TXT_RE.captures(trimmed) {
                                    if let Ok(appid) = caps[1].parse::<u32>() {
                                        if appid > 0 && appid != 480 {
                                            let p_lower = file_path.to_string_lossy().to_lowercase();
                                            let prio = if p_lower.contains("_crack") || p_lower.contains("nodvd") { 3 } else { 1 };
                                            candidates.push(Candidate { priority: prio, appid, depth });
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continue;
            }

            // 2c. 各类 .ini 模拟器与包装器配置
            if ext == "ini" {
                let is_target = INI_WHITELIST.contains(&file_name_lower.as_str()) 
                    || file_name_lower.contains("steam") 
                    || file_name_lower.contains("emu");
                if is_target {
                    let file_path = entry.path();
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.len() < 131072 {
                            if let Ok(bytes) = std::fs::read(&file_path) {
                                let content = String::from_utf8_lossy(&bytes);
                                for line in content.lines() {
                                    let trimmed = line.trim();
                                    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                                        continue;
                                    }
                                    if let Some(caps) = APPID_INI_RE.captures(trimmed) {
                                        if let Ok(appid) = caps[1].parse::<u32>() {
                                            if appid > 0 && appid != 480 {
                                                let p_lower = file_path.to_string_lossy().to_lowercase();
                                                let prio = if p_lower.contains("_crack") || p_lower.contains("nodvd") { 3 } else { 1 };
                                                candidates.push(Candidate { priority: prio, appid, depth });
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
        }

        // 早停策略：如果在根目录或前2级子目录找到了主路径高优先级的 AppID，停止深层搜索
        if candidates.iter().any(|c| c.priority == 1 && c.depth <= 2) {
            break;
        }
    }

    candidates.sort_by_key(|c| (c.priority, c.depth));
    candidates.first().map(|c| c.appid)
}

/// 根据用户定义的分类规则判定游戏类型：
/// 1. 本身是 iso 文件，或者目录中包含 iso 文件 => "ISO"
/// 2. 若来自于“归档路径”根目录：即使有 exe 文件或子目录，也统一归属为已归档应用 => "Archive"
/// 3. 若来自于“安装目录”根目录：当前目录中没有子目录，且一级目录中没有 exe 文件 => "Archive"
/// 4. 否则判定为已安装游戏 => "Installed"
pub fn classify_game_type<P: AsRef<Path>>(path: P, is_dir: bool, is_file_iso: bool, is_archived_root: bool) -> String {
    if is_file_iso {
        return "ISO".to_string();
    }

    if is_dir {
        let mut has_subdirs = false;
        let mut has_exe_in_root = false;
        let mut has_iso = false;

        if let Ok(entries) = std::fs::read_dir(path.as_ref()) {
            for entry in entries.flatten() {
                let name_raw = entry.file_name();
                let name = name_raw.to_string_lossy();
                if name.starts_with('.') || name.starts_with('$') {
                    continue;
                }

                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        has_subdirs = true;
                    } else if ft.is_file() {
                        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                            if ext.eq_ignore_ascii_case("exe") {
                                has_exe_in_root = true;
                            } else if ext.eq_ignore_ascii_case("iso") {
                                has_iso = true;
                            }
                        }
                    }
                }
            }
        }

        // 条件 1：目录中包含 iso 文件
        if has_iso || has_nested_iso(path.as_ref(), 4) {
            return "ISO".to_string();
        }

        // 条件 2：来自于归档根路径，无论是否包含 exe 文件或子目录，均归属于归档游戏
        if is_archived_root {
            return "Archive".to_string();
        }

        // 条件 3：来自于安装根路径，但没有子目录且一级目录没有 exe 文件 => Archive
        if !has_subdirs && !has_exe_in_root {
            return "Archive".to_string();
        }
    }

    "Installed".to_string()
}

/// 归一化游戏类型大类用于去重判断：
/// - "installed"（包括 Installed, Directory, 安装等）
/// - "archived"（包括 Archive, archived, ISO, 归档等）
/// 规则：重复仅在同种类型大类内判断，安装与归档互不视为重复。
pub fn normalize_dup_type_category(game_type: &str) -> &'static str {
    match game_type.trim().to_lowercase().as_str() {
        "installed" | "directory" | "安装" => "installed",
        _ => "archived",
    }
}

fn has_nested_iso<P: AsRef<Path>>(path: P, max_depth: usize) -> bool {
    let mut dirs = vec![(path.as_ref().to_path_buf(), 0)];
    while let Some((current_dir, depth)) = dirs.pop() {
        if depth >= max_depth {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let name_raw = entry.file_name();
                let name = name_raw.to_string_lossy();
                if name.starts_with('.') || name.starts_with('$') {
                    continue;
                }
                if let Ok(ft) = entry.file_type() {
                    if ft.is_file() {
                        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                            if ext.eq_ignore_ascii_case("iso") {
                                return true;
                            }
                        }
                    } else if ft.is_dir() {
                        dirs.push((entry.path(), depth + 1));
                    }
                }
            }
        }
    }
    false
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

    for (p_idx, path_entry) in scan_paths.iter().enumerate() {
        let path_str = &path_entry.path;
        let is_archived_root = path_entry.scan_type == "archived";

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
                let is_iso = entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("iso"));

                if is_dir || is_iso {
                    let r#type = classify_game_type(entry.path(), is_dir, is_iso, is_archived_root);
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
                    let detected_appid = if is_dir {
                        detect_steam_appid_from_game_dir(entry.path())
                    } else {
                        None
                    };

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
                        appid: detected_appid.map(|id| id as i64),
                        name: None,
                        local_cover: None,
                        review_score_desc: None,
                        positive_percent: None,
                        total_reviews: None,
                        recent_review_score_desc: None,
                        recent_positive_percent: None,
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

    // 内存中的去重分组逻辑：按照同种类型大类（安装 vs 归档）以及本地识别的 Steam AppID 进行去重判断
    let mut dup_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, game) in raw_scanned.iter().enumerate() {
        let cat = normalize_dup_type_category(&game.r#type);
        let key = if let Some(aid) = game.appid {
            if aid > 0 {
                format!("appid_{}:::{}", aid, cat)
            } else {
                format!("base_{}:::{}", game.base_name, cat)
            }
        } else {
            format!("base_{}:::{}", game.base_name, cat)
        };
        dup_groups.entry(key).or_insert_with(Vec::new).push(idx);
    }

    for (_key, idx_list) in &dup_groups {
        let is_dup = idx_list.len() > 1;
        for &idx in idx_list {
            raw_scanned[idx].is_exact_dup = false;
            raw_scanned[idx].is_version_dup = is_dup;
            raw_scanned[idx].is_representative = false; // 移除代表版本概念
        }
    }

    // 识别需要查询 Steam 缓存的游戏
    let mut detected_appid_map: HashMap<String, u32> = HashMap::new();
    for game in &raw_scanned {
        if let Some(appid) = game.appid {
            if appid > 0 {
                detected_appid_map.entry(game.base_name.clone()).or_insert(appid as u32);
            }
        }
    }

    let mut all_base_names: Vec<String> = raw_scanned.iter().map(|g| g.base_name.clone()).collect();
    all_base_names.sort();
    all_base_names.dedup();

    let new_games: Vec<(String, Option<u32>)> = all_base_names
        .into_iter()
        .filter(|base| {
            match cache.get(base) {
                Some(entry) => entry.review_score_desc.is_none(),
                None => true,
            }
        })
        .map(|base| {
            let appid_opt = detected_appid_map.get(&base).copied();
            (base, appid_opt)
        })
        .collect();

    let mut new_steam_entries: i64 = 0;
    let total_new = new_games.len();

    if !new_games.is_empty() {
        let targets: Vec<crate::steam_service::SteamSyncTarget> = new_games
            .into_iter()
            .map(|(name, appid)| crate::steam_service::SteamSyncTarget::with_appid(name, appid))
            .collect();

        let entries_count = crate::steam_service::sync_steam_metadata_blocking(
            app_handle.clone(),
            targets,
            crate::steam_service::ProgressReporter::ScanProgress {
                step: "query-steam".to_string(),
            },
            cancel_flag.clone(),
        )?;
        new_steam_entries = entries_count as i64;
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

    // 同步 steam_cache 的 AppID、评测及封面元数据至 games 表
    let _ = crate::db::sync_game_metadata_covers(&conn);
    // 扫描并入库 Steam 元数据后，根据最新匹配的 Steam AppID 全量刷新疑似重复判定
    let _ = crate::db::recalculate_existing_games(&conn);

    // 记录扫描历史
    let completed_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = insert_scan_history(
        &conn,
        &started_at,
        &completed_at,
        raw_scanned.len() as i64,
        total_new as i64,
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
        assert_eq!(
            base_game_name("FallenDoll_0.4.9"),
            "fallendoll"
        );
        assert_eq!(
            base_game_name("FallenDoll_v0.4.9"),
            "fallendoll"
        );
    }

    #[test]
    fn test_detect_steam_appid_from_game_dir_txt() {
        let temp_dir = std::env::temp_dir().join(format!("hoardster_test_txt_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let txt_file = temp_dir.join("steam_appid.txt");
        std::fs::write(&txt_file, "1817070\r\n").unwrap();

        let detected = detect_steam_appid_from_game_dir(&temp_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert_eq!(detected, Some(1817070));
    }

    #[test]
    fn test_detect_steam_appid_from_game_dir_ini() {
        let temp_dir = std::env::temp_dir().join(format!("hoardster_test_ini_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let sub_dir = temp_dir.join("Engine").join("Binaries");
        std::fs::create_dir_all(&sub_dir).unwrap();

        let ini_file = sub_dir.join("steam_emu.ini");
        std::fs::write(&ini_file, "[Settings]\r\nAppId = 451050\r\nUserName = Player\r\n").unwrap();

        let detected = detect_steam_appid_from_game_dir(&temp_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert_eq!(detected, Some(451050));
    }

    #[test]
    fn test_detect_steam_appid_with_non_utf8_ansi_art() {
        let temp_dir = std::env::temp_dir().join(format!("hoardster_test_ansi_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let ini_file = temp_dir.join("steam_emu.ini");
        // 包含常见 CODEX / RELOADED 的非 UTF-8 ANSI Art 字节 (0xDC, 0xDB 等)
        let mut bytes = vec![0x23, 0x20, 0xDC, 0xDB, 0xFE, 0x0D, 0x0A];
        bytes.extend_from_slice(b"[Settings]\r\nAppId=690640\r\nUserName=CODEX\r\n");
        std::fs::write(&ini_file, &bytes).unwrap();

        let detected = detect_steam_appid_from_game_dir(&temp_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert_eq!(detected, Some(690640));
    }

    #[test]
    fn test_detect_steam_appid_ignores_spacewar() {
        let temp_dir = std::env::temp_dir().join(format!("hoardster_test_480_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let txt_file = temp_dir.join("steam_appid.txt");
        std::fs::write(&txt_file, "480\r\n").unwrap();

        let detected = detect_steam_appid_from_game_dir(&temp_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert_eq!(detected, None);
    }

    #[test]
    fn test_detect_steam_appid_from_acf() {
        let temp_dir = std::env::temp_dir().join(format!("hoardster_test_acf_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let acf_file = temp_dir.join("appmanifest_1091500.acf");
        std::fs::write(&acf_file, "AppState {\r\n\t\"appid\"\t\"1091500\"\r\n}").unwrap();

        let detected = detect_steam_appid_from_game_dir(&temp_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert_eq!(detected, Some(1091500));
    }

    #[test]
    fn test_detect_steam_appid_prune_directory() {
        let temp_dir = std::env::temp_dir().join(format!("hoardster_test_prune_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let content_dir = temp_dir.join("Content").join("Sub");
        std::fs::create_dir_all(&content_dir).unwrap();

        let txt_file = content_dir.join("steam_appid.txt");
        std::fs::write(&txt_file, "123456\r\n").unwrap();

        let detected = detect_steam_appid_from_game_dir(&temp_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Content 目录在 PRUNE_DIRS 中被剪枝，不应递归扫描
        assert_eq!(detected, None);
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

    #[test]
    fn test_classify_game_type() {
        let temp_dir = std::env::temp_dir().join(format!("hoardster_test_classify_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // 1. 创建包含 exe 文件的目录
        let game_with_exe = temp_dir.join("GameWithExe");
        std::fs::create_dir_all(&game_with_exe).unwrap();
        std::fs::write(game_with_exe.join("game.exe"), b"dummy exe").unwrap();

        // 在安装根路径下：包含 exe => "Installed"
        assert_eq!(classify_game_type(&game_with_exe, true, false, false), "Installed");

        // 在归档根路径下：即使包含 exe => 必须归属为 "Archive"
        assert_eq!(classify_game_type(&game_with_exe, true, false, true), "Archive");

        // 2. 创建普通纯资源目录（无 exe，无子目录）
        let game_empty = temp_dir.join("GameEmpty");
        std::fs::create_dir_all(&game_empty).unwrap();
        std::fs::write(game_empty.join("readme.txt"), b"readme").unwrap();

        // 安装根路径下无 exe 无子目录 => "Archive"
        assert_eq!(classify_game_type(&game_empty, true, false, false), "Archive");
        // 归档根路径下 => "Archive"
        assert_eq!(classify_game_type(&game_empty, true, false, true), "Archive");

        // 3. 包含 iso 文件的目录 => 均为 "ISO"
        let game_with_iso = temp_dir.join("GameWithIso");
        std::fs::create_dir_all(&game_with_iso).unwrap();
        std::fs::write(game_with_iso.join("disc.iso"), b"dummy iso").unwrap();
        assert_eq!(classify_game_type(&game_with_iso, true, false, false), "ISO");
        assert_eq!(classify_game_type(&game_with_iso, true, false, true), "ISO");

        // 4. 单独的 iso 文件 => "ISO"
        let standalone_iso = temp_dir.join("standalone.iso");
        std::fs::write(&standalone_iso, b"dummy iso").unwrap();
        assert_eq!(classify_game_type(&standalone_iso, false, true, false), "ISO");
        assert_eq!(classify_game_type(&standalone_iso, false, true, true), "ISO");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
