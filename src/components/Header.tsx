import { Settings, Database, Home, Radar } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../stores/useAppStore";
import { useScrapeStore } from "../stores/useScrapeStore";

export default function Header() {
  const { t } = useTranslation();
  const { menuMode, setMenuMode } = useAppStore();
  const isAnyScraping = useScrapeStore((s) => Object.values(s.tasks).some((t) => t.isScraping));

  return (
    <header>
      <div>
        <h1>{t("headerTitle")}</h1>
        <div className="subtitle">{t("headerSubtitle")}</div>
      </div>
      <div style={{ display: "flex", gap: "1rem", alignItems: "center" }}>
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
      </div>
    </header>
  );
}
