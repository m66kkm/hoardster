import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Gift, Calendar, RefreshCw, ExternalLink } from "lucide-react";

type EpicGame = {
  id: string;
  title: string;
  description: string;
  status: string;
  start_date: string;
  end_date: string;
  image_url: string;
  game_url: string;
};

export default function EpicGamesPanel({ showToast }: { showToast: (msg: string) => void }) {
  const [games, setGames] = useState<EpicGame[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const sortGames = (gamesArray: EpicGame[]) => {
    return [...gamesArray].sort((a, b) => {
      if (a.status === "现在免费" && b.status !== "现在免费") return -1;
      if (a.status !== "现在免费" && b.status === "现在免费") return 1;
      return new Date(a.start_date).getTime() - new Date(b.start_date).getTime();
    });
  };

  const loadGames = async () => {
    try {
      const data = await invoke<EpicGame[]>("get_epic_free_games_command");
      setGames(sortGames(data));
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
      const data = await invoke<EpicGame[]>("fetch_epic_free_games_command");
      setGames(sortGames(data));
      showToast("获取 Epic 免费游戏情报成功！");
    } catch (err: any) {
      showToast("获取失败: " + err);
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
            <Gift size={22} style={{ color: "var(--primary-accent)" }} />
            Epic 喜加一情报
          </h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            实时获取 Epic 平台当前免费及即将推出的免费游戏信息。
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
          {isLoading ? "正在获取..." : "获取Epic信息"}
        </button>
      </div>

      <div style={{ display: "flex", gap: "1.5rem", flexWrap: "wrap" }}>
        {games.map(game => {
            const startDate = new Date(game.start_date);
            const endDate = new Date(game.end_date);
            const formattedDate = `${startDate.getMonth() + 1}月${startDate.getDate()}日 - ${endDate.getMonth() + 1}月${endDate.getDate()}日`;
            
            return (
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
                  if (game.game_url) {
                      import('@tauri-apps/plugin-shell').then(({ open }) => {
                          open(game.game_url);
                      }).catch(() => {
                          window.open(game.game_url, "_blank");
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
              title="点击前往 Epic 商城领取"
              >
                <div style={{ width: "100%", height: "180px", background: "#0f172a", position: "relative" }}>
                   {game.image_url ? (
                       <img src={game.image_url} alt={game.title} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
                   ) : (
                       <div style={{ display: "flex", height: "100%", alignItems: "center", justifyContent: "center", color: "#666" }}>暂无图片</div>
                   )}
                   <div style={{ 
                       position: "absolute", 
                       bottom: "10px", 
                       left: "10px",
                       padding: "6px 10px",
                       borderRadius: "6px",
                       fontSize: "0.85rem",
                       fontWeight: "bold",
                       background: game.status === "现在免费" ? "rgba(0, 120, 215, 0.9)" : "rgba(100, 100, 100, 0.9)",
                       color: "#fff",
                       boxShadow: "0 2px 4px rgba(0,0,0,0.3)"
                   }}>
                       {game.status}
                   </div>
                </div>
                <div style={{ padding: "1.25rem", flex: 1, display: "flex", flexDirection: "column" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "0.5rem" }}>
                        <h3 style={{ margin: "0 0 0.75rem 0", fontSize: "1.2rem", fontWeight: 600, color: "var(--text-primary)" }}>{game.title}</h3>
                        <ExternalLink size={16} style={{ color: "var(--text-secondary)", flexShrink: 0, marginTop: "4px" }} />
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "0.85rem", color: "var(--text-secondary)", marginBottom: "0.75rem" }}>
                        <Calendar size={14} />
                        <span>{game.status === "现在免费" ? `当前免费，${endDate.getMonth() + 1}月${endDate.getDate()}日 ${endDate.getHours().toString().padStart(2, '0')}:${endDate.getMinutes().toString().padStart(2, '0')}截止` : `免费 ${formattedDate}`}</span>
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
            );
        })}
      </div>
    </div>
  );
}
