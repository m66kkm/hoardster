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

export default function App() {
  const { menuMode, toastMessage } = useAppStore();
  const { i18n } = useTranslation();

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

  return (
    <div className="container">
      <Header />

      {menuMode === "home" && <HomePage />}
      {menuMode === "local" && <LocalGamesPage />}
      {menuMode === "radar" && <RadarPage />}
      {menuMode === "settings" && <SettingsPage />}

      <Toast message={toastMessage} />
    </div>
  );
}
