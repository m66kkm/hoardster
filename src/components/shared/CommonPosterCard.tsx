import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink, Gamepad2 } from "lucide-react";
import {
  getRatingColorClass,
  getReviewScoreText,
  getSteamStoreUrl,
  getCoverUrl,
  getGradientsForName,
  openExternalUrl,
} from "../../utils/helpers";

export interface SteamStoreBadgeProps {
  appid?: number | null;
  baseName?: string | null;
  title: string;
  onClick?: (e: React.MouseEvent) => void;
  style?: React.CSSProperties;
}

/**
 * 通用 Steam ↗ 商店徽标按钮
 */
export function SteamStoreBadge({
  appid,
  baseName,
  title,
  onClick,
  style,
}: SteamStoreBadgeProps) {
  const { t } = useTranslation();

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onClick) {
      onClick(e);
    } else {
      const url = getSteamStoreUrl(appid || undefined, baseName || undefined, title);
      openExternalUrl(url);
    }
  };

  return (
    <span
      className="badge badge-dir"
      onClick={handleClick}
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
        gap: "3px",
        ...style,
      }}
    >
      <Gamepad2 size={10} /> Steam ↗
    </span>
  );
}

export interface CommonPosterCardProps {
  // 标题与悬浮提示
  title: string;
  tooltip?: string;

  // 点击与右键事件
  onClick?: (e: React.MouseEvent) => void;
  onContextMenu?: (e: React.MouseEvent) => void;

  // 封面图（支持单个 URL 或候选 URL 数组依次兜底）
  coverUrl?: string | null;
  coverUrls?: (string | null | undefined)[];
  fallbackIcon?: React.ReactNode;
  fallbackGradient?: string;
  fallbackSeed?: string;

  // Steam 评价与元数据
  appid?: number | null;
  baseName?: string | null;
  reviewScoreDesc?: number | string | null;
  positivePercent?: number | string | null;
  recentReviewScoreDesc?: number | string | null;
  recentPositivePercent?: number | string | null;
  onSteamClick?: (e: React.MouseEvent) => void;

  // 卡片底部信息插槽
  metaPrimary?: React.ReactNode;   // 顶部行：如发布时间、评论/热度、做种数等
  metaSecondary?: React.ReactNode; // 底部行：类型徽标、Steam 按钮、文件大小等
  children?: React.ReactNode;
}

/**
 * 通用游戏卡片组件
 * 提供海报/横幅自适应展示、横版毛玻璃背景模糊、Steam 评价角标浮层、
 * 渐变兜底占位图以及灵活插槽，供本地游戏库与游戏情报站等共同复用。
 */
export default function CommonPosterCard({
  title,
  tooltip,
  onClick,
  onContextMenu,
  coverUrl,
  coverUrls,
  fallbackIcon,
  fallbackGradient,
  fallbackSeed,
  appid,
  baseName,
  reviewScoreDesc,
  positivePercent,
  recentReviewScoreDesc,
  recentPositivePercent,
  onSteamClick,
  metaPrimary,
  metaSecondary,
  children,
}: CommonPosterCardProps) {
  const { t } = useTranslation();

  // 候选封面列表去重与规范化
  const sources = useMemo(() => {
    const list = coverUrls && coverUrls.length > 0 ? coverUrls : (coverUrl ? [coverUrl] : []);
    const candidates = list
      .filter((u): u is string => Boolean(u && u.trim().length > 0))
      .map((u) => getCoverUrl(u) || u);
    return Array.from(new Set(candidates));
  }, [coverUrl, coverUrls]);

  const [srcIndex, setSrcIndex] = useState(0);

  // 封面源变更时重置索引
  useEffect(() => {
    setSrcIndex(0);
  }, [sources.join("|")]);

  const currentSrc = srcIndex < sources.length ? sources[srcIndex] : null;

  const [isLandscape, setIsLandscape] = useState(() => {
    if (!currentSrc) return false;
    return currentSrc.includes("header") || currentSrc.includes("capsule");
  });

  useEffect(() => {
    setIsLandscape(Boolean(currentSrc && (currentSrc.includes("header") || currentSrc.includes("capsule"))));
  }, [currentSrc]);

  // 评价计算
  const hasRating = reviewScoreDesc !== undefined && reviewScoreDesc !== null && reviewScoreDesc !== "" && Number(reviewScoreDesc) > 0;
  const ratingText = hasRating ? getReviewScoreText(t, reviewScoreDesc) : "";
  const hasPercent = positivePercent !== undefined && positivePercent !== null && Number(positivePercent) > 0;

  const hasRecentRating = recentReviewScoreDesc !== undefined && recentReviewScoreDesc !== null && recentReviewScoreDesc !== "" && Number(recentReviewScoreDesc) > 0;
  const recentRatingText = hasRecentRating ? getReviewScoreText(t, recentReviewScoreDesc) : "";
  const hasRecentPercent = recentPositivePercent !== undefined && recentPositivePercent !== null && Number(recentPositivePercent) > 0;

  const showRecent = hasRecentRating && hasRating && Number(recentReviewScoreDesc) !== Number(reviewScoreDesc);

  const handleSteamClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onSteamClick) {
      onSteamClick(e);
    } else {
      const url = getSteamStoreUrl(appid || undefined, baseName || undefined, title);
      openExternalUrl(url);
    }
  };

  const handleImageError = () => {
    setIsLandscape(false);
    setSrcIndex((prev) => prev + 1);
  };

  return (
    <div
      className="poster-card"
      onClick={onClick}
      onContextMenu={onContextMenu}
      title={tooltip || title}
    >
      {/* Steam 评价角标浮层 */}
      {hasRating && (
        <div
          className={`rating-overlay ${getRatingColorClass(reviewScoreDesc)}`}
          onClick={handleSteamClick}
          title={`${ratingText}${hasPercent ? ` (${positivePercent}%)` : ""}${showRecent ? `\n近期: ${recentRatingText}${hasRecentPercent ? ` (${recentPositivePercent}%)` : ""}` : ""}\n${t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"}`}
        >
          {hasPercent && <span>👍 {positivePercent}%</span>}
          <span className="rating-desc">{ratingText}</span>
          {showRecent && (
            <span className="rating-recent" style={{ fontSize: "0.6rem", opacity: 0.85, display: "block", lineHeight: 1.2 }}>
              近期: <span className={getRatingColorClass(recentReviewScoreDesc)} style={{ color: "inherit" }}>{recentRatingText}</span>
              {hasRecentPercent && ` ${recentPositivePercent}%`}
            </span>
          )}
          <ExternalLink size={10} style={{ opacity: 0.8, marginLeft: 2 }} />
        </div>
      )}

      {/* 封面图片展示或兜底背景 */}
      {currentSrc ? (
        isLandscape ? (
          <div className="poster-landscape-wrapper">
            <img
              className="poster-landscape-bg"
              src={currentSrc}
              alt=""
              aria-hidden="true"
            />
            <div className="poster-landscape-inner">
              <img
                className="poster-landscape-banner"
                src={currentSrc}
                alt={title}
                loading="lazy"
                referrerPolicy="no-referrer"
                onError={handleImageError}
              />
            </div>
          </div>
        ) : (
          <img
            className="poster-img"
            src={currentSrc}
            alt={title}
            loading="lazy"
            referrerPolicy="no-referrer"
            onLoad={(e) => {
              const img = e.currentTarget;
              if (img.naturalWidth && img.naturalHeight) {
                if (img.naturalWidth / img.naturalHeight > 0.85) {
                  setIsLandscape(true);
                }
              }
            }}
            onError={handleImageError}
          />
        )
      ) : (
        <div
          className="poster-fallback"
          style={{ background: fallbackGradient || getGradientsForName(fallbackSeed || title) }}
        >
          <div className="poster-fallback-icon">{fallbackIcon || "🎮"}</div>
          <div className="poster-fallback-title" title={title}>
            {title}
          </div>
          <div
            style={{
              marginTop: "auto",
              width: "100%",
              display: "flex",
              flexDirection: "column",
              color: "rgba(255,255,255,0.85)",
            }}
          >
            {metaPrimary && (
              <div
                className="poster-meta"
                style={{
                  marginBottom: "0.25rem",
                  color: "rgba(255,255,255,0.7)",
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                {metaPrimary}
              </div>
            )}
            {metaSecondary && (
              <div
                className="poster-meta"
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                {metaSecondary}
              </div>
            )}
            {children}
          </div>
        </div>
      )}

      {/* 封面图片有效时，底部浮动信息栏 */}
      {currentSrc && (
        <div className="poster-info">
          <div className="poster-title" title={title}>
            {title}
          </div>
          {metaPrimary && (
            <div
              className="poster-meta"
              style={{
                marginBottom: "0.25rem",
                color: "rgba(255,255,255,0.7)",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
              }}
            >
              {metaPrimary}
            </div>
          )}
          {metaSecondary && (
            <div
              className="poster-meta"
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
              }}
            >
              {metaSecondary}
            </div>
          )}
          {children}
        </div>
      )}
    </div>
  );
}
