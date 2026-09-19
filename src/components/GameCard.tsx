import type { Game } from "../types";
import { getRatingColorClass, getCoverUrl, getGradientsForName, getReviewScoreText, getTypeBadgeClass } from "../utils/helpers";
import { useTranslation } from "react-i18next";

interface GameCardProps {
  game: Game;
  onOpenFolder: (path: string) => void;
}

export default function GameCard({ game, onOpenFolder }: GameCardProps) {
  const { t } = useTranslation();
  const cover = getCoverUrl(game.local_cover);

  return (
    <div 
      className="poster-card" 
      onClick={() => onOpenFolder(game.full_path)}
      title={`${game.original_name}\nSteam类型: ${game.genres || "未知"}\n文件类别: ${game.type}\n路径: ${game.full_path}\n大小: ${game.size}`}
    >
      {game.review_score_desc !== undefined && game.review_score_desc !== null && game.review_score_desc !== "" && Number(game.review_score_desc) > 0 && (
        <div className={`rating-overlay ${getRatingColorClass(game.review_score_desc)}`}>
          {game.positive_percent !== undefined && game.positive_percent !== null && Number(game.positive_percent) > 0 && (
            <span>👍 {game.positive_percent}%</span>
          )}
          <span className="rating-desc">{getReviewScoreText(t, game.review_score_desc)}</span>
        </div>
      )}
      
      {cover ? (
        <img className="poster-img" src={cover} alt={game.original_name} loading="lazy" />
      ) : (
        <div className="poster-fallback" style={{ background: getGradientsForName(game.original_name) }}>
          <div className="poster-fallback-icon">🎮</div>
          <div className="poster-fallback-title" title={game.name || game.original_name}>{game.name || game.original_name}</div>
          <div style={{ fontSize: "0.75rem", color: "rgba(255,255,255,0.6)", marginTop: "auto", display: "flex", justifyContent: "space-between", width: "100%" }}>
            <span className={`badge ${getTypeBadgeClass(game.type)}`} style={{ background: "rgba(255,255,255,0.15)", color: "#fff", border: "none", padding: "0.15rem 0.5rem" }}>
              {game.type}
            </span>
            <span style={{ fontWeight: 600 }}>{game.source_path.substring(0, 2)}</span>
          </div>
        </div>
      )}
      
      {cover && (
        <div className="poster-info">
          <div className="poster-title" title={game.name || game.original_name}>{game.name || game.original_name}</div>
          <div className="poster-meta">
            <span className={`badge ${getTypeBadgeClass(game.type)}`}>
              {game.type}
            </span>
            <span style={{ opacity: 0.8, fontSize: "0.75rem", fontWeight: 600 }}>{game.source_path.substring(0, 2)}</span>
          </div>
        </div>
      )}
    </div>
  );
}
