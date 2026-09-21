import type { Game } from "../types";
import { 
  getRatingColorClass, 
  getCoverUrl, 
  getGradientsForName, 
  getReviewScoreText, 
  getTypeBadgeClass,
  getSteamStoreUrl,
  openExternalUrl
} from "../utils/helpers";
import { useTranslation } from "react-i18next";
import { ExternalLink, Gamepad2 } from "lucide-react";

interface GameCardProps {
  game: Game;
  onOpenFolder: (path: string) => void;
}

export default function GameCard({ game, onOpenFolder }: GameCardProps) {
  const { t } = useTranslation();
  const cover = getCoverUrl(game.local_cover);

  const hasRating = game.review_score_desc !== undefined && game.review_score_desc !== null && game.review_score_desc !== "" && Number(game.review_score_desc) > 0;
  const ratingText = hasRating ? getReviewScoreText(t, game.review_score_desc) : "";
  const hasPercent = game.positive_percent !== undefined && game.positive_percent !== null && Number(game.positive_percent) > 0;

  const hasRecentRating = game.recent_review_score_desc !== undefined && game.recent_review_score_desc !== null && game.recent_review_score_desc !== "" && Number(game.recent_review_score_desc) > 0;
  const recentRatingText = hasRecentRating ? getReviewScoreText(t, game.recent_review_score_desc) : "";
  const hasRecentPercent = game.recent_positive_percent !== undefined && game.recent_positive_percent !== null && Number(game.recent_positive_percent) > 0;

  // Show recent rating if it differs from all-time rating
  const showRecent = hasRecentRating && hasRating && Number(game.recent_review_score_desc) !== Number(game.review_score_desc);

  const handleSteamClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    const url = getSteamStoreUrl(game.appid, game.base_name, game.name || game.original_name);
    openExternalUrl(url);
  };

  return (
    <div 
      className="poster-card" 
      onClick={() => onOpenFolder(game.full_path)}
      title={`${game.original_name}\nSteam类型: ${game.genres || "未知"}\n文件类别: ${game.type}\n路径: ${game.full_path}\n大小: ${game.size}${hasRating ? `\nSteam评价: ${ratingText}${hasPercent ? ` (${game.positive_percent}%)` : ""}` : ""}\n\n(左键打开文件夹 / 点击 Steam 评价打开 Steam)`}
    >
      {hasRating && (
        <div 
          className={`rating-overlay ${getRatingColorClass(game.review_score_desc)}`}
          onClick={handleSteamClick}
          title={`${ratingText}${hasPercent ? ` (${game.positive_percent}%)` : ""}${showRecent ? `\n近期: ${recentRatingText}${hasRecentPercent ? ` (${game.recent_positive_percent}%)` : ""}` : ""}\n${t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"}`}
        >
          {hasPercent && <span>👍 {game.positive_percent}%</span>}
          <span className="rating-desc">{ratingText}</span>
          {showRecent && (
            <span className="rating-recent" style={{ fontSize: "0.6rem", opacity: 0.85, display: "block", lineHeight: 1.2 }}>
              近期: <span className={getRatingColorClass(game.recent_review_score_desc)} style={{ color: "inherit" }}>{recentRatingText}</span>
              {hasRecentPercent && ` ${game.recent_positive_percent}%`}
            </span>
          )}
          <ExternalLink size={10} style={{ opacity: 0.8, marginLeft: 2 }} />
        </div>
      )}
      
      {cover ? (
        <img className="poster-img" src={cover} alt={game.original_name} loading="lazy" />
      ) : (
        <div className="poster-fallback" style={{ background: getGradientsForName(game.original_name) }}>
          <div className="poster-fallback-icon">🎮</div>
          <div className="poster-fallback-title" title={game.name || game.original_name}>{game.name || game.original_name}</div>
          <div style={{ fontSize: "0.75rem", color: "rgba(255,255,255,0.6)", marginTop: "auto", display: "flex", justifyContent: "space-between", alignItems: "center", width: "100%", flexWrap: "wrap", gap: "0.25rem" }}>
            <div style={{ display: "flex", gap: "0.25rem", alignItems: "center" }}>
              <span className={`badge ${getTypeBadgeClass(game.type)}`} style={{ background: "rgba(255,255,255,0.15)", color: "#fff", border: "none", padding: "0.15rem 0.5rem" }}>
                {game.type}
              </span>
              {(game.appid || hasRating) && (
                <span 
                  className="badge badge-dir"
                  onClick={handleSteamClick}
                  title={t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"}
                  style={{ 
                    fontSize: "0.65rem", 
                    padding: "0.15rem 0.4rem", 
                    background: "rgba(0, 242, 254, 0.15)", 
                    border: "1px solid rgba(0, 242, 254, 0.35)", 
                    color: "#00f2fe",
                    cursor: "pointer",
                    display: "inline-flex",
                    alignItems: "center",
                    gap: "3px"
                  }}
                >
                  <Gamepad2 size={10} /> Steam ↗
                </span>
              )}
            </div>
            <span style={{ fontWeight: 600 }}>{game.source_path.substring(0, 2)}</span>
          </div>
        </div>
      )}
      
      {cover && (
        <div className="poster-info">
          <div className="poster-title" title={game.name || game.original_name}>{game.name || game.original_name}</div>
          <div className="poster-meta" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <div style={{ display: "flex", gap: "0.25rem", alignItems: "center" }}>
              <span className={`badge ${getTypeBadgeClass(game.type)}`}>
                {game.type}
              </span>
              {(game.appid || hasRating) && (
                <span 
                  className="badge badge-dir"
                  onClick={handleSteamClick}
                  title={t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"}
                  style={{ 
                    fontSize: "0.65rem", 
                    padding: "0.15rem 0.4rem", 
                    background: "rgba(0, 242, 254, 0.15)", 
                    border: "1px solid rgba(0, 242, 254, 0.35)", 
                    color: "#00f2fe",
                    cursor: "pointer",
                    display: "inline-flex",
                    alignItems: "center",
                    gap: "3px"
                  }}
                >
                  <Gamepad2 size={10} /> Steam ↗
                </span>
              )}
            </div>
            <span style={{ opacity: 0.8, fontSize: "0.75rem", fontWeight: 600 }}>{game.source_path.substring(0, 2)}</span>
          </div>
        </div>
      )}
    </div>
  );
}
