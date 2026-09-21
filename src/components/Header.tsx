import { useEffect, useState } from "react";
import { Settings, Database, Home, Radar, Minus, Square, Copy, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../stores/useAppStore";
import { useScrapeStore } from "../stores/useScrapeStore";
import { getCurrentWindow } from "@tauri-apps/api/window";

export default function Header() {
  const { t } = useTranslation();
  const { menuMode, setMenuMode } = useAppStore();
  const isAnyScraping = useScrapeStore((s) => Object.values(s.tasks).some((t) => t.isScraping));

  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const checkMaximized = async () => {
      try {
        const maximized = await appWindow.isMaximized();
        setIsMaximized(maximized);
      } catch (err) {
        console.error("Failed to check maximized state:", err);
      }
    };

    checkMaximized();

    appWindow.onResized(() => {
      checkMaximized();
    }).then((fn) => {
      unlisten = fn;
    }).catch(console.error);

    return () => {
      if (unlisten) unlisten();
    };
  }, [appWindow]);

  const handleMinimize = async () => {
    try {
      await appWindow.minimize();
    } catch (err) {
      console.error("Failed to minimize window:", err);
    }
  };

  const handleToggleMaximize = async () => {
    try {
      await appWindow.toggleMaximize();
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    } catch (err) {
      console.error("Failed to toggle maximize window:", err);
    }
  };

  const handleClose = async () => {
    try {
      await appWindow.close();
    } catch (err) {
      console.error("Failed to close window:", err);
    }
  };

  const handleStartDragging = async (e: React.MouseEvent) => {
    // 仅响应鼠标左键按下，并且目标不是按钮或可交互元素
    if (e.button === 0) {
      const target = e.target as HTMLElement;
      if (target.closest("button") || target.closest("a") || target.closest("input")) {
        return;
      }
      try {
        await appWindow.startDragging();
      } catch (err) {
        console.error("Failed to start dragging:", err);
      }
    }
  };

  return (
    <header data-tauri-drag-region onMouseDown={handleStartDragging}>
      <div 
        data-tauri-drag-region 
        onMouseDown={handleStartDragging} 
        style={{ cursor: "default", flex: 1 }}
      >
        <h1 data-tauri-drag-region onMouseDown={handleStartDragging}>{t("headerTitle")}</h1>
        <div className="subtitle" data-tauri-drag-region onMouseDown={handleStartDragging}>{t("headerSubtitle")}</div>
      </div>
      <div style={{ display: "flex", gap: "0.75rem", alignItems: "center" }}>
        <button 
          className={`view-btn ${menuMode === "home" ? "active" : ""}`}
          onClick={() => setMenuMode("home")}
          style={{ padding: "0.5rem" }}
          title={t("homeBtn")}
        >
          <Home size={18} />
        </button>
        <button 
          className={`view-btn ${menuMode === "local" ? "active" : ""}`}
          onClick={() => setMenuMode("local")}
          style={{ padding: "0.5rem" }}
          title={t("tabAll")}
        >
          <Database size={18} />
        </button>
        <button 
          className={`view-btn ${menuMode === "radar" ? "active" : ""}`}
          onClick={() => setMenuMode("radar")}
          style={{ padding: "0.5rem", position: "relative" }}
          title={t("radarBtn")}
        >
          <Radar size={18} />
          {isAnyScraping && (
            <span style={{
              position: "absolute",
              top: "4px",
              right: "4px",
              width: "6px",
              height: "6px",
              borderRadius: "50%",
              background: "var(--primary-accent)",
              boxShadow: "0 0 6px var(--primary-accent)"
            }} className="animate-pulse" />
          )}
        </button>
        <button 
          className={`view-btn ${menuMode === "settings" ? "active" : ""}`}
          onClick={() => setMenuMode("settings")}
          style={{ padding: "0.5rem" }}
          title={t("settingsBtn")}
        >
          <Settings size={18} />
        </button>

        {/* 分隔线与窗口控制按钮 */}
        <div className="window-controls-divider" />

        <button 
          className="view-btn"
          onClick={handleMinimize}
          style={{ padding: "0.5rem" }}
          title={t("titlebarMinimize") || "最小化"}
        >
          <Minus size={18} />
        </button>
        <button 
          className="view-btn"
          onClick={handleToggleMaximize}
          style={{ padding: "0.5rem" }}
          title={isMaximized ? (t("titlebarRestore") || "向下还原") : (t("titlebarMaximize") || "最大化")}
        >
          {isMaximized ? <Copy size={16} style={{ transform: "rotate(90deg)" }} /> : <Square size={16} />}
        </button>
        <button 
          className="view-btn view-btn-close"
          onClick={handleClose}
          style={{ padding: "0.5rem" }}
          title={t("titlebarClose") || "关闭"}
        >
          <X size={18} />
        </button>
      </div>
    </header>
  );
}
