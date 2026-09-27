import { useEffect } from "react";
import Header from "./components/Header";
import Toast from "./components/Toast";

import HomePage from "./pages/HomePage";
import LocalGamesPage from "./pages/LocalGamesPage";
import RadarPage from "./pages/RadarPage";
import SettingsPage from "./pages/SettingsPage";

import { useAppStore } from "./stores/useAppStore";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";

export default function App() {
  const { menuMode, toastMessage } = useAppStore();
  const { i18n } = useTranslation();
  const appWindow = getCurrentWindow();

  // Initialize theme and language
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", "dark");
    
    // Attempt to read language from config on startup
    invoke<Record<string, string>>("get_all_config_command")
      .then(config => {
        if (config.language) {
          i18n.changeLanguage(config.language);
        }
      })
      .catch(console.error);
  }, [i18n]);

  const handleContainerMouseDown = async (e: React.MouseEvent) => {
    if (e.button === 0) {
      const target = e.target as HTMLElement;
      // 只要不是交互元素或 header，支持在顶层空白处拖动
      if (
        target.closest("header") || 
        target.closest(".header-right-controls") || 
        target.closest("button") || 
        target.closest("a") || 
        target.closest("input") || 
        target.closest("select") || 
        target.closest(".tab-content-scrollable") ||
        target.closest(".poster-card") ||
        target.closest(".stat-card")
      ) {
        return;
      }
      
      if (e.detail % 2 === 0) {
        try {
          await appWindow.toggleMaximize();
        } catch (err) {
          console.error("Failed to toggle maximize:", err);
        }
      } else {
        try {
          await appWindow.startDragging();
        } catch (err) {
          console.error("Failed to drag from container:", err);
        }
      }
    }
  };

  return (
    <div 
      className="container" 
      onMouseDown={handleContainerMouseDown}
    >
      <Header />

      {menuMode === "home" && <HomePage />}
      {menuMode === "local" && <LocalGamesPage />}
      {menuMode === "radar" && <RadarPage />}
      {menuMode === "settings" && <SettingsPage />}

      <Toast message={toastMessage} />
    </div>
  );
}
