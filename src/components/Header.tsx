import { useEffect, useState } from "react";
import { Settings, Database, Home, Radar, Minus, Square, Copy, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../stores/useAppStore";
import { useLocalGamesStore } from "../stores/useLocalGamesStore";
import { useScrapeStore } from "../stores/useScrapeStore";
import { getCurrentWindow } from "@tauri-apps/api/window";

export default function Header() {
  const { t } = useTranslation();
  const { menuMode, setMenuMode } = useAppStore();
  const { setActiveTab } = useLocalGamesStore();
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

  const handleHeaderMouseDown = async (e: React.MouseEvent) => {
    // 阻止向顶层 container 冒泡，防止重复触发拖动
    e.stopPropagation();

    // 仅响应鼠标左键按下，并且目标不是按钮或可交互元素
    if (e.button === 0) {
      const target = e.target as HTMLElement;
      if (target.closest("button") || target.closest("a") || target.closest("input")) {
        return;
      }

      if (e.detail % 2 === 0) {
        // 双击：切换最大化/向下还原（不调用 startDragging，防止 Windows 拖拽打断最大化导致闪烁）
        handleToggleMaximize();
      } else {
        // 单击：开始拖动窗口
        try {
          await appWindow.startDragging();
        } catch (err) {
          console.error("Failed to start dragging:", err);
        }
      }
    }
  };

  return (
    <header onMouseDown={handleHeaderMouseDown}>
      <div style={{ cursor: "default", flex: 1 }}>
        <h1>{t("headerTitle")}</h1>
        <div className="subtitle">{t("headerSubtitle")}</div>
      </div>

      {/* 顶部右侧系统控制区（固定在窗口最右上角，与传统应用窗口按钮行为与位置一致） */}
      <div 
        className="header-right-controls" 
        onMouseDown={handleHeaderMouseDown}
      >
        <div className="header-nav-group">
          <button 
            className={`header-nav-btn ${menuMode === "home" ? "active" : ""}`}
            onClick={() => setMenuMode("home")}
            title={t("homeBtn")}
          >
            <Home size={17} />
          </button>
          <button 
            className={`header-nav-btn ${menuMode === "local" ? "active" : ""}`}
            onClick={() => {
              setMenuMode("local");
              setActiveTab("installed");
            }}
            title={t("gameIndexBtn")}
          >
            <Database size={17} />
          </button>
          <button 
            className={`header-nav-btn ${menuMode === "radar" ? "active" : ""}`}
            onClick={() => setMenuMode("radar")}
            style={{ position: "relative" }}
            title={t("radarBtn")}
          >
            <Radar size={17} />
            {isAnyScraping && (
              <span className="radar-badge animate-pulse" />
            )}
          </button>
          <button 
            className={`header-nav-btn ${menuMode === "settings" ? "active" : ""}`}
            onClick={() => setMenuMode("settings")}
            title={t("settingsBtn")}
          >
            <Settings size={17} />
          </button>
        </div>

        {/* 分隔线 */}
        <div className="header-window-divider" />

        {/* 传统桌面窗口标题栏控制按钮：最小化、最大化/还原、关闭 */}
        <div className="window-controls-group">
          <button 
            className="win-caption-btn win-caption-min"
            onClick={handleMinimize}
            title={t("titlebarMinimize") || "最小化"}
          >
            <Minus size={16} />
          </button>
          <button 
            className="win-caption-btn win-caption-max"
            onClick={handleToggleMaximize}
            title={isMaximized ? (t("titlebarRestore") || "向下还原") : (t("titlebarMaximize") || "最大化")}
          >
            {isMaximized ? <Copy size={13} style={{ transform: "rotate(90deg)" }} /> : <Square size={13} />}
          </button>
          <button 
            className="win-caption-btn win-caption-close"
            onClick={handleClose}
            title={t("titlebarClose") || "关闭"}
          >
            <X size={16} />
          </button>
        </div>
      </div>
    </header>
  );
}
