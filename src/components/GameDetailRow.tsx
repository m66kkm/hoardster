import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink, Gamepad2, Copy, Check } from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import type { Game } from "../types";
import { 
  getRatingColorClass, 
  getCoverUrl, 
  getGradientsForName, 
  getReviewScoreText, 
  getTypeBadgeClass,
  getTypeLabel,
  getSteamStoreUrl,
  openExternalUrl
} from "../utils/helpers";

interface GameDetailRowProps {
  game: Game;
  onCopyPath: (path: string, gameName: string) => void;
  onOpenFolder: (path: string) => void;
  onContextMenu?: (e: React.MouseEvent, game: Game) => void;
}

export default function GameDetailRow({ game, onCopyPath, onOpenFolder, onContextMenu }: GameDetailRowProps) {
  const { t } = useTranslation();
  const { showToast } = useAppStore();
  const [copied, setCopied] = useState(false);

  const title = game.name || game.original_name;
  const cover = getCoverUrl(game.local_cover);

  const handleCopyTitle = (e: React.MouseEvent) => {
    e.stopPropagation();
    e.preventDefault();
    if (!title) return;
    navigator.clipboard.writeText(title).then(() => {
      setCopied(true);
      showToast(t("toastGameNameCopied", { name: title }) || `已复制游戏名称: ${title}`);
      setTimeout(() => setCopied(false), 1500);
    }).catch((err) => {
      console.error("Failed to copy game title:", err);
      showToast(t("toastGameNameCopyFailed") || "复制游戏名称失败");
    });
  };

  return (
    <div 
      className="detail-row" 
      onClick={() => onCopyPath(game.full_path, game.original_name)}
      onContextMenu={(e) => {
        if (onContextMenu) {
          onContextMenu(e, game);
        }
      }}
    >
      <div className="detail-cover-container">
        {cover ? (
          <img className="detail-cover-img" src={cover} alt={game.original_name} loading="lazy" referrerPolicy="no-referrer" />
        ) : (
          <div className="detail-fallback" style={{ background: getGradientsForName(game.original_name) }}>🎮</div>
        )}
      </div>
      <div className="detail-content">
        <div className="detail-header-line">
          <div style={{ display: "flex", alignItems: "center", gap: "0.5rem", minWidth: 0 }}>
            <div className="detail-game-title" title={title}>{title}</div>
            <button
              type="button"
              className={`card-copy-btn ${copied ? "copied" : ""}`}
              onClick={handleCopyTitle}
              title={copied ? (t("toastGameNameCopied", { name: title }) || "已复制") : (t("copyGameName") || "复制游戏名称")}
              aria-label={t("copyGameName") || "复制游戏名称"}
            >
              {copied ? <Check size={12} /> : <Copy size={12} />}
            </button>
          </div>
        </div>
        <div className="detail-path-line">
          <span className={`badge ${getTypeBadgeClass(game.type)}`}>{getTypeLabel(game.type, t)}</span>
          <span className="code-path" onClick={(e) => { e.stopPropagation(); onCopyPath(game.full_path, game.original_name); }} title={t("copyPathMsg")}>{game.full_path}</span>
        </div>
        <div className="detail-info-line">
          <div className="detail-info-item">{t("sizeLabel")} <strong>{game.size}</strong></div>
          <div className="detail-info-item">{t("physicalPathLabel")} <strong>{game.source_path}</strong></div>
          {game.genres && <div className="detail-info-item">{t("genreLabel")} <strong style={{ color: "var(--accent)" }}>{game.genres}</strong></div>}
        </div>
      </div>
      <div className="detail-right-meta">
        {game.review_score_desc !== undefined && game.review_score_desc !== null && game.review_score_desc !== "" && Number(game.review_score_desc) > 0 ? (
          <span 
            className={`rating-text ${getRatingColorClass(game.review_score_desc)} clickable-rating`}
            onClick={(e) => {
              e.stopPropagation();
              openExternalUrl(getSteamStoreUrl(game.appid, game.base_name, game.name || game.original_name));
            }}
            title={`${t("steamRatingHover", { percent: game.positive_percent, total: game.total_reviews })}${game.recent_review_score_desc && Number(game.recent_review_score_desc) > 0 ? `\n近期: ${getReviewScoreText(t, game.recent_review_score_desc)}${game.recent_positive_percent ? ` (${game.recent_positive_percent}%)` : ""}` : ""}\n${t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"}`}
          >
            {game.positive_percent !== undefined && game.positive_percent !== null && Number(game.positive_percent) > 0 ? `👍 ${game.positive_percent}% ` : ""}
            ({getReviewScoreText(t, game.review_score_desc)})
            {game.recent_review_score_desc && Number(game.recent_review_score_desc) > 0 && Number(game.recent_review_score_desc) !== Number(game.review_score_desc) && (
              <span style={{ fontSize: "0.75rem", marginLeft: "0.35rem", opacity: 0.85 }}>
                [近期: <strong className={getRatingColorClass(game.recent_review_score_desc)} style={{ color: "inherit" }}>{getReviewScoreText(t, game.recent_review_score_desc)}</strong>{game.recent_positive_percent ? ` ${game.recent_positive_percent}%` : ""}]
              </span>
            )}
            <ExternalLink size={10} style={{ marginLeft: 4, opacity: 0.7 }} />
          </span>
        ) : (
          <span style={{ fontSize: "0.75rem", color: "var(--text-secondary)", opacity: 0.5 }}>{t("noRating") || "暂无评价"}</span>
        )}
        <div style={{ fontSize: "0.75rem", color: "var(--text-secondary)" }}>{t("releaseLabel")} <span style={{ color: "var(--text-primary)" }}>{game.release_date || "未知"}</span></div>
        <div style={{ fontSize: "0.725rem", color: "var(--text-secondary)", opacity: 0.8 }}>{t("createdLabel")} <span style={{ color: "var(--text-primary)" }}>{game.created || "未知"}</span></div>
      </div>
      <div style={{ marginLeft: "1rem", display: "flex", gap: "0.5rem" }} onClick={(e) => e.stopPropagation()}>
        {(game.appid || (game.review_score_desc && Number(game.review_score_desc) > 0)) && (
          <button 
            className="view-btn" 
            onClick={() => openExternalUrl(getSteamStoreUrl(game.appid, game.base_name, game.name || game.original_name))} 
            title={t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"} 
            style={{ padding: "0.4rem", color: "var(--primary-accent)" }}
          >
            <Gamepad2 size={14} />
          </button>
        )}
        <button className="view-btn" onClick={() => onOpenFolder(game.full_path)} title={t("openInExplorer")} style={{ padding: "0.4rem" }}>
          <ExternalLink size={14} />
        </button>
      </div>
    </div>
  );
}
