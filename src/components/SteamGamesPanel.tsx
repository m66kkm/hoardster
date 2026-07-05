import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Gamepad2, Calendar, RefreshCw, ExternalLink } from "lucide-react";
import { useTranslation } from "react-i18next";

type SteamFreeGame = {
  id: number;
  title: string;
  description: string;
  type: string;
  end_date: string;
  image_url: string;
  giveaway_url: string;
};

export default function SteamGamesPanel({ showToast }: { showToast: (msg: string) => void }) {
  const { t } = useTranslation();
  const [games, setGames] = useState<SteamFreeGame[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const loadGames = async () => {
    try {
      const data = await invoke<SteamFreeGame[]>("get_steam_free_games_command");
      setGames(data);
      if (data.length === 0) {
        fetchGames();
      }
    } catch (err: any) {
      console.error(err);
    }
  };

  const fetchGames = async () => {
    setIsLoading(true);
    try {
      const data = await invoke<SteamFreeGame[]>("fetch_steam_free_games_command");
      setGames(data);
      if (data.length === 0) {
          showToast(t("steamNoEvent"));
      } else {
          showToast(t("steamSuccess"));
      }
    } catch (err: any) {
      showToast(t("steamError") + err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadGames();
  }, []);

  return (
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header" style={{ marginBottom: "1.5rem", display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
        <div>
          <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <Gamepad2 size={22} style={{ color: "var(--primary-accent)" }} />
            {t("steamTitle")}
          </h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            {t("steamDesc")}
          </p>
        </div>
        <button 
          className="action-btn" 
          onClick={fetchGames} 
          disabled={isLoading}
          style={{ 
            padding: "0.4rem 0.9rem", 
            fontSize: "0.85rem", 
            borderRadius: "8px", 
            height: "32px", 
            display: "inline-flex", 
            alignItems: "center", 
            gap: "0.35rem" 
          }}
        >
          <RefreshCw size={14} className={isLoading ? "animate-spin" : ""} />
          {isLoading ? t("steamBtnFetching") : t("steamBtnFetch")}
        </button>
      </div>

      <div style={{ display: "flex", gap: "1.5rem", flexWrap: "wrap" }}>
        {games.length === 0 && !isLoading && (
            <div style={{ color: "var(--text-secondary)", fontSize: "0.9rem", padding: "2rem", width: "100%", textAlign: "center" }}>
                {t("steamEmpty")}
            </div>
        )}
        
        {games.map(game => (
            <div key={game.id} style={{
                background: "rgba(15, 23, 42, 0.4)",
                border: "1px solid var(--panel-border)",
                borderRadius: "12px",
                width: "320px",
                overflow: "hidden",
                display: "flex",
                flexDirection: "column",
                boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
                transition: "transform 0.2s, box-shadow 0.2s",
                cursor: "pointer"
              }}
              onClick={() => {
                  if (game.giveaway_url) {
                      import('@tauri-apps/plugin-shell').then(({ open }) => {
                          open(game.giveaway_url);
                      }).catch(() => {
                          window.open(game.giveaway_url, "_blank");
                      });
                  }
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.transform = "translateY(-4px)";
                e.currentTarget.style.boxShadow = "0 8px 16px rgba(0,0,0,0.3)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = "none";
                e.currentTarget.style.boxShadow = "0 4px 12px rgba(0,0,0,0.15)";
              }}
              title={t("steamGoToStore")}
            >
                <div style={{ width: "100%", height: "180px", background: "#0f172a", position: "relative" }}>
                   {game.image_url ? (
                       <img src={game.image_url} alt={game.title} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
                   ) : (
                       <div style={{ display: "flex", height: "100%", alignItems: "center", justifyContent: "center", color: "#666" }}>{t("steamNoImage")}</div>
                   )}
                   <div style={{ 
                       position: "absolute", 
                       bottom: "10px", 
                       left: "10px",
                       padding: "6px 10px",
                       borderRadius: "6px",
                       fontSize: "0.85rem",
                       fontWeight: "bold",
                       background: "rgba(0, 120, 215, 0.9)",
                       color: "#fff",
                       boxShadow: "0 2px 4px rgba(0,0,0,0.3)"
                   }}>
                       {game.type}
                   </div>
                </div>
                <div style={{ padding: "1.25rem", flex: 1, display: "flex", flexDirection: "column" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "0.5rem" }}>
                        <h3 style={{ margin: "0 0 0.75rem 0", fontSize: "1.2rem", fontWeight: 600, color: "var(--text-primary)" }}>{game.title}</h3>
                        <ExternalLink size={16} style={{ color: "var(--text-secondary)", flexShrink: 0, marginTop: "4px" }} />
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "0.85rem", color: "var(--text-secondary)", marginBottom: "0.75rem" }}>
                        <Calendar size={14} />
                        <span>{t("steamEnd")} {game.end_date === "N/A" || !game.end_date ? t("steamUnknown") : game.end_date}</span>
                    </div>
                    <p style={{ 
                        fontSize: "0.9rem", 
                        color: "var(--text-secondary)", 
                        margin: 0, 
                        display: "-webkit-box", 
                        WebkitLineClamp: 3, 
                        WebkitBoxOrient: "vertical", 
                        overflow: "hidden",
                        lineHeight: 1.5 
                    }}>
                        {game.description}
                    </p>
                </div>
            </div>
        ))}
      </div>
    </div>
  );
}
