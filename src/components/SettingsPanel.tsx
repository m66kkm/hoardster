import { useState, type RefObject } from "react";
import { Plus, Play, RefreshCw, Trash2, Sparkles, X, CheckCircle } from "lucide-react";
import ScanProgress from "./ScanProgress";
import { useTranslation } from "react-i18next";
import { STEAM_LANGUAGES } from "../i18n";
import { invoke } from "@tauri-apps/api/core";
import { ask, message } from "@tauri-apps/plugin-dialog";
import { useScrape } from "../hooks/useScrape";
import { useDataCorrection } from "../hooks/useDataCorrection";

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

  const {
    isRunning: isCorrecting,
    phase: correctingPhase,
    current: correctingCurrent,
    total: correctingTotal,
    progressPercent,
    message: correctingMessage,
    lastRun: dataCorrectionLastRun,
    startCorrection,
    cancelCorrection
  } = useDataCorrection();

  const [isClearing, setIsClearing] = useState(false);
  const [isClearingSR, setIsClearingSR] = useState(false);

  const handleClear1337xData = async () => {
    const confirmed = await ask(t("confirmClearDataDesc") || "Are you sure you want to clear the 1337x database?", {
      title: t("confirmClearDataTitle") || "Confirm Clear Data",
      kind: "warning",
    });
    
    if (!confirmed) return;

    try {
      setIsClearing(true);
      await invoke("clear_data_1337x");
      await message(t("clearDataSuccessDesc") || "The database has been successfully cleared.", {
        title: t("clearDataSuccessTitle") || "Clear Data Success",
        kind: "info",
      });
    } catch (e) {
      console.error("Failed to clear 1337x data:", e);
      await message("Failed to clear database.", {
        title: "Error",
        kind: "error",
      });
    } finally {
      setIsClearing(false);
    }
  };

  const handleClearSRData = async () => {
    const confirmed = await ask(t("srConfirmClearDataDesc") || "Are you sure you want to clear the Skidrow/Reloaded database?", {
      title: t("confirmClearDataTitle") || "Confirm Clear Data",
      kind: "warning",
    });
    
    if (!confirmed) return;

    try {
      setIsClearingSR(true);
      await invoke("clear_data_sr");
      await message(t("srClearDataSuccessDesc") || "The database has been successfully cleared.", {
        title: t("clearDataSuccessTitle") || "Clear Data Success",
        kind: "info",
      });
    } catch (e) {
      console.error("Failed to clear SR data:", e);
      await message("Failed to clear database.", {
        title: "Error",
        kind: "error",
      });
    } finally {
      setIsClearingSR(false);
    }
  };

  return (
    <div className="panel" style={{ display: "block" }}>
      {activeTab === "general" && (
        <>
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

        <div className="settings-section" style={{ marginTop: "2rem" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
            <h3 style={{ margin: 0 }}>{t("dataCorrectionTitle")}</h3>
            <div style={{ display: "flex", gap: "0.5rem" }}>
              {isCorrecting ? (
                <button 
                  className="action-btn" 
                  onClick={cancelCorrection} 
                  style={{ 
                    width: "auto", 
                    padding: "0.5rem 1rem", 
                    fontSize: "0.9rem",
                    backgroundColor: "rgba(239, 68, 68, 0.2)",
                    borderColor: "var(--danger-color)",
                    color: "#fff"
                  }}
                >
                  <X size={16} style={{ marginRight: "0.4rem" }} />
                  {t("dataCorrectionBtnCancel")}
                </button>
              ) : (
                <button 
                  className="action-btn" 
                  onClick={startCorrection} 
                  style={{ width: "auto", padding: "0.5rem 1rem", fontSize: "0.9rem" }}
                >
                  <Sparkles size={16} style={{ marginRight: "0.4rem" }} />
                  {t("dataCorrectionBtnStart")}
                </button>
              )}
            </div>
          </div>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.85rem", marginBottom: "0.5rem" }}>
            {t("dataCorrectionDesc")}
          </p>
          <div style={{ fontSize: "0.8rem", color: "var(--text-secondary)", marginBottom: "1rem" }}>
            {t("dataCorrectionLastRun")}: <span style={{ color: "var(--text-primary)" }}>{dataCorrectionLastRun || t("dataCorrectionNeverRun")}</span>
          </div>

          {isCorrecting && (
            <div style={{ marginTop: "1rem", background: "rgba(255, 255, 255, 0.03)", padding: "1rem", borderRadius: "8px", border: "1px solid var(--panel-border)" }}>
              <div style={{ display: "flex", justifyContent: "space-between", fontSize: "0.85rem", marginBottom: "0.5rem" }}>
                <span style={{ color: "var(--primary-accent)", display: "flex", alignItems: "center", gap: "0.4rem" }}>
                  <RefreshCw size={14} className="animate-spin" />
                  {correctingMessage || t("dataCorrectionBtnRunning")}
                </span>
                <span style={{ color: "var(--text-secondary)" }}>
                  {correctingTotal > 0 ? `${correctingCurrent} / ${correctingTotal} (${progressPercent}%)` : ""}
                </span>
              </div>
              <div style={{ width: "100%", height: "6px", backgroundColor: "var(--bg-lighter)", borderRadius: "3px", overflow: "hidden" }}>
                <div style={{ width: `${progressPercent}%`, height: "100%", backgroundColor: "var(--primary-accent)", transition: "width 0.3s ease" }} />
              </div>
            </div>
          )}

          {!isCorrecting && correctingMessage && correctingPhase === "completed" && (
            <div style={{ marginTop: "0.75rem", fontSize: "0.85rem", color: "#10b981", display: "flex", alignItems: "center", gap: "0.4rem" }}>
              <CheckCircle size={15} />
              <span>{correctingMessage}</span>
            </div>
          )}
        </div>
        </>
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
                    if ((window as any).__TAURI_PLUGIN_DIALOG__) {
                      const { message } = await import("@tauri-apps/plugin-dialog");
                      message("Steam缓存已清空，开始重新获取数据！", { title: '成功', kind: 'info' });
                    }
                    startScan();
                    const container = document.querySelector('.tab-content-scrollable');
                    if (container) {
                      container.scrollTo({ top: 0, behavior: 'smooth' });
                    }
                  } catch (e) {
                    console.error("clear_steam_cache_command error:", e);
                    if ((window as any).__TAURI_PLUGIN_DIALOG__) {
                       const { message } = await import("@tauri-apps/plugin-dialog");
                       message(`Error clearing cache: ${e}`, { title: 'Error', kind: 'error' });
                    }
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

          <div className="setting-group" style={{ marginBottom: "2rem" }}>
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

          <div className="setting-group" style={{ borderColor: "rgba(239, 68, 68, 0.2)", marginTop: "1rem" }}>
            <div className="setting-header">
              <div>
                <h3 className="setting-title" style={{ color: "#ef4444" }}>{t("srClearTitle") || "清空所有 Skidrow/Reloaded 数据"}</h3>
                <p className="setting-desc">{t("srClearDesc") || "清空本地数据库中已缓存的所有 Skidrow/Reloaded 列表信息。清空后可以重新抓取。"}</p>
              </div>
              <button 
                className="action-btn" 
                onClick={handleClearSRData} 
                disabled={isClearingSR}
                style={{ 
                  height: "36px", 
                  padding: "0 1rem", 
                  fontSize: "0.9rem",
                  backgroundColor: "rgba(239, 68, 68, 0.1)",
                  color: "#ef4444",
                  border: "1px solid rgba(239, 68, 68, 0.3)"
                }}
              >
                <Trash2 size={16} className={isClearingSR ? "animate-spin" : ""} style={{ marginRight: "0.5rem" }} />
                {isClearingSR ? t("intelBtnClearing") : t("intelBtnClear")}
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
