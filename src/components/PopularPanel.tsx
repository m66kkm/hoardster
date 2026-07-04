import { Sparkles, Search, Star } from "lucide-react";

interface PopularPanelProps {
  onSearchInLocal: (title: string) => void;
}

import { POPULAR_GAMES } from "../mocks/popularGames";
export default function PopularPanel({ onSearchInLocal }: PopularPanelProps) {


  return (
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header" style={{ marginBottom: "1.5rem" }}>
        <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: "0.5rem" }}>
          <Sparkles size={22} style={{ color: "var(--primary-accent)" }} />
          热门游戏盘点
        </h2>
        <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
          实时盘点当前最受关注和最畅销的热门大作，支持一键在本地盘库数据中检索。
        </p>
      </div>

      <div className="popular-grid" style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))",
        gap: "1.25rem",
        marginTop: "1.5rem"
      }}>
        {POPULAR_GAMES.map((game) => (
          <div 
            key={game.id} 
            className="popular-card" 
            style={{
              background: "rgba(15, 23, 42, 0.55)",
              border: "1px solid var(--panel-border)",
              borderRadius: "14px",
              padding: "1.25rem",
              position: "relative",
              overflow: "hidden",
              transition: "transform 0.2s ease, border-color 0.2s ease",
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-between",
              height: "240px"
            }}
          >
            {/* Background Graphic */}
            <div style={{
              position: "absolute",
              right: "-15px",
              bottom: "-25px",
              fontSize: "8rem",
              opacity: 0.05,
              pointerEvents: "none",
              userSelect: "none"
            }}>
              {game.bgIcon}
            </div>

            {/* Header */}
            <div>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "0.5rem" }}>
                <div>
                  <h3 style={{ margin: 0, fontSize: "1.2rem", fontWeight: 700, color: "var(--text-primary)" }}>{game.name}</h3>
                  <div style={{ fontSize: "0.85rem", color: "var(--text-secondary)", marginTop: "0.15rem" }}>{game.enName}</div>
                </div>
                <div style={{
                  background: "rgba(0, 242, 254, 0.1)",
                  border: "1px solid rgba(0, 242, 254, 0.2)",
                  color: "var(--primary-accent)",
                  padding: "0.2rem 0.5rem",
                  borderRadius: "6px",
                  fontSize: "0.75rem",
                  fontWeight: 600,
                  display: "flex",
                  alignItems: "center",
                  gap: "0.25rem",
                  whiteSpace: "nowrap"
                }}>
                  <Star size={12} fill="var(--primary-accent)" />
                  {game.rating} 好评
                </div>
              </div>

              {/* Desc */}
              <p style={{
                fontSize: "0.85rem",
                color: "var(--text-secondary)",
                marginTop: "0.85rem",
                lineHeight: 1.5,
                display: "-webkit-box",
                WebkitLineClamp: 3,
                WebkitBoxOrient: "vertical",
                overflow: "hidden",
                marginRight: "2.5rem"
              }}>
                {game.desc}
              </p>
            </div>

            {/* Footer */}
            <div style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginTop: "1rem",
              borderTop: "1px solid rgba(255,255,255,0.05)",
              paddingTop: "0.75rem"
            }}>
              <div style={{ fontSize: "0.75rem", color: "var(--text-secondary)" }}>
                <div>类型: <span style={{ color: "var(--text-primary)" }}>{game.genre}</span></div>
                <div style={{ marginTop: "0.15rem" }}>发行: <span style={{ color: "var(--text-primary)" }}>{game.release}</span></div>
              </div>
              <button
                className="action-btn"
                onClick={() => onSearchInLocal(game.name)}
                style={{
                  padding: "0.4rem 0.75rem",
                  fontSize: "0.75rem",
                  borderRadius: "6px",
                  display: "inline-flex",
                  alignItems: "center",
                  gap: "0.35rem"
                }}
              >
                <Search size={12} />
                检索本地库
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
