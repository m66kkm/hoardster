import { useState } from "react";
import type { Game } from "../types";
import { getTypeLabel } from "../utils/helpers";
import { useTranslation } from "react-i18next";
import { Copy, Check } from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import CommonPosterCard from "./shared/CommonPosterCard";

interface GameCardProps {
  game: Game;
  onOpenFolder: (path: string) => void;
  onContextMenu?: (e: React.MouseEvent, game: Game) => void;
  hideTypeTag?: boolean;
  hideLocation?: boolean;
}

export default function GameCard({
  game,
  onOpenFolder,
  onContextMenu,
  hideLocation = false,
}: GameCardProps) {
  const { t } = useTranslation();
  const { showToast } = useAppStore();
  const [copied, setCopied] = useState(false);

  const title = game.name || game.original_name;
  const typeLabel = getTypeLabel(game.type, t);

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
    <CommonPosterCard
      title={title}
      coverUrl={game.local_cover}
      fallbackSeed={game.original_name}
      appid={game.appid}
      baseName={game.base_name}
      reviewScoreDesc={game.review_score_desc}
      positivePercent={game.positive_percent}
      recentReviewScoreDesc={game.recent_review_score_desc}
      recentPositivePercent={game.recent_positive_percent}
      onClick={() => onOpenFolder(game.full_path)}
      onContextMenu={(e) => onContextMenu?.(e, game)}
      tooltip={`${game.original_name}\nSteam类型: ${game.genres || "未知"}\n文件类别: ${typeLabel}\n路径: ${game.full_path}\n大小: ${game.size}\n\n(左键打开文件夹 / 右键操作菜单 / 点击 Steam 评价打开 Steam)`}
      metaSecondary={
        <>
          <div style={{ display: "flex", gap: "0.25rem", alignItems: "center" }}>
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
          {!hideLocation && (
            <span style={{ opacity: 0.8, fontSize: "0.75rem", fontWeight: 600 }}>
              {game.source_path ? game.source_path.substring(0, 2) : ""}
            </span>
          )}
        </>
      }
    />
  );
}
