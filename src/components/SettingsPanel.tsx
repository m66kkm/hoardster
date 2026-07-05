import { useState, type RefObject } from "react";
import { Plus, Play, RefreshCw, Trash2 } from "lucide-react";
import ScanProgress from "./ScanProgress";
import { useTranslation } from "react-i18next";
import { STEAM_LANGUAGES } from "../i18n";
import { invoke } from "@tauri-apps/api/core";
import { useScrape } from "../hooks/useScrape";

interface SettingsPanelProps {
  activeTab: string;
  scanPaths: string[];
  addScanPath: () => void;
  removeScanPath: (path: string) => void;
  startScan: () => void;
  cancelScan: () => void;
  isScanning: boolean;
  scanProgress: number;
  scanMessage: string;
  scanLogs: string[];
  loggerRef: RefObject<HTMLDivElement | null>;
  steamApiThreads: number;
  saveSteamApiThreads: (threads: number) => void;
  clearLogs: () => void;
  language: string;
  saveLanguage: (lang: string) => void;
}

export default function SettingsPanel({
  activeTab,
  scanPaths,
  addScanPath,
  removeScanPath,
  startScan,
  cancelScan,
  isScanning,
  scanProgress,
  scanMessage,
  scanLogs,
  loggerRef,
  steamApiThreads,
  saveSteamApiThreads,
  clearLogs,
  language,
  saveLanguage
}: SettingsPanelProps) {
  const { t } = useTranslation();

  const { 
    isScraping: isScrapingLeechers, 
    scrapeProgress: progressLeechers, 
    scrapeMessage: msgLeechers, 
    startScrape: startLeechers 
  } = useScrape({ mode: "leechers" });

  const { 
    isScraping: isScrapingSeeders, 
    scrapeProgress: progressSeeders, 
    scrapeMessage: msgSeeders, 
    startScrape: startSeeders 
  } = useScrape({ mode: "seeders" });

  const [isClearing, setIsClearing] = useState(false);

  const handleClear1337xData = async () => {
    try {
      setIsClearing(true);
      await invoke("clear_data_1337x");
    } catch (e) {
      console.error("Failed to clear 1337x data:", e);
    } finally {
      setIsClearing(false);
    }
  };

  return (
    <div className="panel" style={{ display: "block" }}>
      {activeTab === "general" && (
        <div className="settings-section">
          <h3>{t("language")}</h3>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.85rem", marginBottom: "1rem" }}>
            {t("languageDesc")}
          </p>
          <div className="path-input-group" style={{ maxWidth: "300px" }}>
            <select 
              value={language} 
              onChange={(e) => saveLanguage(e.target.value)}
              className="path-input"
              style={{ padding: "0.5rem", background: "var(--panel-bg)", color: "var(--text-primary)", border: "1px solid var(--panel-border)", borderRadius: "8px" }}
            >
              {STEAM_LANGUAGES.map(lang => (
                <option key={lang.code} value={lang.code}>{lang.name}</option>
              ))}
            </select>
          </div>
        </div>
      )}

      {activeTab === "local" && (
        <>
          <div className="settings-section">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.5rem" }}>
              <h3 style={{ margin: 0 }}>{t("scanPathsMgmt")}</h3>
              <button className="action-btn" onClick={addScanPath} style={{ width: "auto", padding: "0.5rem 1rem", fontSize: "0.9rem" }}>
                <Plus size={16} />
                {t("addPath")}
              </button>
            </div>

            <div className="paths-list">
              {scanPaths.map((path) => (
                <div key={path} className="path-item">
                  <span className="path-text">{path}</span>
                  <button className="remove-btn" onClick={() => removeScanPath(path)}>
                    {t("remove")}
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div className="settings-section">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
              <h3 style={{ margin: 0 }}>{t("startSync")}</h3>
              <div style={{ display: "flex", gap: "1rem" }}>
                {!isScanning ? (
                  <button className="action-btn" onClick={startScan} style={{ width: "auto", padding: "0.5rem 1rem", fontSize: "0.9rem" }}>
                    <Play size={16} />
                    {t("startScan")}
                  </button>
                ) : (
                  <button className="action-btn" onClick={cancelScan} style={{ width: "auto", padding: "0.5rem 1rem", fontSize: "0.9rem", backgroundColor: "rgba(239, 68, 68, 0.2)", borderColor: "var(--danger-color)", color: "#fff" }}>
                    {t("stopScan")}
                  </button>
                )}
              </div>
            </div>
            <p style={{ color: "var(--text-secondary)", fontSize: "0.85rem", marginBottom: "1rem" }}>
              {t("scanDesc")}
            </p>

            <ScanProgress
              isScanning={isScanning}
              scanProgress={scanProgress}
              scanMessage={scanMessage}
              scanLogs={scanLogs}
              loggerRef={loggerRef}
              onClose={clearLogs}
            />
          </div>

          <div className="settings-section">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
              <h3 style={{ margin: 0 }}>{t("apiThreads")}</h3>
              <button 
                className="action-btn" 
                onClick={async () => {
                  try {
                    await invoke("clear_steam_cache_command");
                    startScan();
                  } catch (e) {
                    console.error(e);
                  }
                }} 
                style={{ width: "auto", padding: "0.5rem 1rem", fontSize: "0.9rem" }}
                title={t("clearSteamCacheDesc")}
              >
                <RefreshCw size={14} />
                {t("clearSteamCache")}
              </button>
            </div>
            <p style={{ color: "var(--text-secondary)", fontSize: "0.85rem", marginBottom: "0.5rem" }}>
              {t("clearSteamCacheDesc")}
            </p>
            <p style={{ color: "var(--text-secondary)", fontSize: "0.85rem", marginBottom: "1rem" }}>
              {t("threadsDesc")}
            </p>
            <div className="path-input-group" style={{ maxWidth: "300px" }}>
              <input
                type="number"
                min="1"
                max="100"
                value={steamApiThreads}
                onChange={(e) => saveSteamApiThreads(parseInt(e.target.value) || 1)}
                className="path-input"
              />
              <span style={{ marginLeft: "1rem", color: "var(--text-secondary)" }}>{t("threads")}</span>
            </div>
          </div>
        </>
      )}

      {activeTab === "intel" && (
        <>
          <h2 style={{ fontSize: "1.1rem", fontWeight: 600, color: "var(--text-primary)", margin: "0 0 1rem 0" }}>
            {t("intelTitle")}
          </h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", margin: "0 0 2rem 0" }}>
            {t("intelDesc")}
          </p>

          <div className="setting-group" style={{ marginBottom: "2rem" }}>
            <div className="setting-header">
              <div>
                <h3 className="setting-title">{t("intelLeechTitle")}</h3>
                <p className="setting-desc">{t("intelLeechDesc")}</p>
              </div>
              <button 
                className="action-btn" 
                onClick={startLeechers} 
                disabled={isScrapingLeechers}
                style={{ height: "36px", padding: "0 1rem", fontSize: "0.9rem" }}
              >
                <RefreshCw size={16} className={isScrapingLeechers ? "animate-spin" : ""} style={{ marginRight: "0.5rem" }} />
                {isScrapingLeechers ? t("intelBtnScraping") : t("intelBtnFetch")}
              </button>
            </div>
            {isScrapingLeechers && (
              <div style={{ marginTop: "1rem" }}>
                <div style={{ width: "100%", height: "6px", backgroundColor: "var(--bg-lighter)", borderRadius: "3px", overflow: "hidden" }}>
                  <div style={{ width: `${progressLeechers}%`, height: "100%", backgroundColor: "var(--primary-accent)", transition: "width 0.3s ease" }} />
                </div>
                <p style={{ fontSize: "0.85rem", color: "var(--text-secondary)", marginTop: "0.5rem", textAlign: "right" }}>
                  {msgLeechers} ({progressLeechers}%)
                </p>
              </div>
            )}
          </div>

          <div className="setting-group">
            <div className="setting-header">
              <div>
                <h3 className="setting-title">{t("intelSeedTitle")}</h3>
                <p className="setting-desc">{t("intelSeedDesc")}</p>
              </div>
              <button 
                className="action-btn" 
                onClick={startSeeders} 
                disabled={isScrapingSeeders}
                style={{ height: "36px", padding: "0 1rem", fontSize: "0.9rem" }}
              >
                <RefreshCw size={16} className={isScrapingSeeders ? "animate-spin" : ""} style={{ marginRight: "0.5rem" }} />
                {isScrapingSeeders ? t("intelBtnScraping") : t("intelBtnFetch")}
              </button>
            </div>
            {isScrapingSeeders && (
              <div style={{ marginTop: "1rem" }}>
                <div style={{ width: "100%", height: "6px", backgroundColor: "var(--bg-lighter)", borderRadius: "3px", overflow: "hidden" }}>
                  <div style={{ width: `${progressSeeders}%`, height: "100%", backgroundColor: "var(--primary-accent)", transition: "width 0.3s ease" }} />
                </div>
                <p style={{ fontSize: "0.85rem", color: "var(--text-secondary)", marginTop: "0.5rem", textAlign: "right" }}>
                  {msgSeeders} ({progressSeeders}%)
                </p>
              </div>
            )}
          </div>

          <div className="setting-group" style={{ borderColor: "rgba(239, 68, 68, 0.2)" }}>
            <div className="setting-header">
              <div>
                <h3 className="setting-title" style={{ color: "#ef4444" }}>{t("intelClearTitle")}</h3>
                <p className="setting-desc">{t("intelClearDesc")}</p>
              </div>
              <button 
                className="action-btn" 
                onClick={handleClear1337xData} 
                disabled={isClearing || isScrapingLeechers || isScrapingSeeders}
                style={{ 
                  height: "36px", 
                  padding: "0 1rem", 
                  fontSize: "0.9rem",
                  backgroundColor: "rgba(239, 68, 68, 0.1)",
                  color: "#ef4444",
                  border: "1px solid rgba(239, 68, 68, 0.3)"
                }}
              >
                <Trash2 size={16} className={isClearing ? "animate-spin" : ""} style={{ marginRight: "0.5rem" }} />
                {isClearing ? t("intelBtnClearing") : t("intelBtnClear")}
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
