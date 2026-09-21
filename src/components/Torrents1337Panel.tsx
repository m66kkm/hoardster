import { useState, useEffect } from "react";
import { Download, ExternalLink, RefreshCw, Gamepad2, ArrowUpCircle } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { mapSteamLangToBCP47 } from "../i18n";
import type { Torrent1337x } from "../types";
import { 
  getRatingColorClass, 
  getReviewScoreText, 
  getSteamStoreUrl,
  getCoverUrl,
  getGradientsForName
} from "../utils/helpers";
import SearchBox from "./shared/SearchBox";
import FilterSelect from "./shared/FilterSelect";
import SortSelect from "./shared/SortSelect";
import Pagination from "./shared/Pagination";

function TorrentCoverImage({ torrent }: { torrent: Torrent1337x }) {
  const [srcIndex, setSrcIndex] = useState(0);
  const [isLandscape, setIsLandscape] = useState(false);

  const cover = getCoverUrl(torrent.local_cover);
  const steamLibraryCover = torrent.appid 
    ? `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${torrent.appid}/library_600x900_2x.jpg` 
    : null;
  const steamCapsuleCover = torrent.appid
    ? `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${torrent.appid}/capsule_616x353.jpg`
    : null;
  const steamHeaderCover = torrent.appid
    ? `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${torrent.appid}/header.jpg`
    : null;

  const candidates = [cover, steamLibraryCover, steamCapsuleCover, steamHeaderCover].filter(
    (url): url is string => Boolean(url && url.length > 0)
  );
  const sources = Array.from(new Set(candidates));

  useEffect(() => {
    setSrcIndex(0);
    setIsLandscape(false);
  }, [torrent.local_cover, torrent.appid]);

  if (srcIndex >= sources.length) {
    return (
      <div className="poster-fallback" style={{ background: getGradientsForName(torrent.name) }}>
        <div className="poster-fallback-icon">🎮</div>
        <div className="poster-fallback-title" title={torrent.name}>{torrent.name}</div>
      </div>
    );
  }

  const currentSrc = sources[srcIndex];

  if (isLandscape) {
    return (
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
            alt={torrent.name}
            referrerPolicy="no-referrer"
            loading="lazy"
            onError={() => {
              setIsLandscape(false);
              setSrcIndex(prev => prev + 1);
            }}
          />
        </div>
      </div>
    );
  }

  return (
    <img 
      className="poster-img"
      src={currentSrc} 
      alt={torrent.name}
      referrerPolicy="no-referrer"
      loading="lazy"
      onLoad={(e) => {
        const img = e.currentTarget;
        if (img.naturalWidth && img.naturalHeight) {
          if (img.naturalWidth / img.naturalHeight > 1.15) {
            setIsLandscape(true);
          }
        }
      }}
      onError={() => {
        setIsLandscape(false);
        setSrcIndex(prev => prev + 1);
      }}
    />
  );
}

const formatPublishDate = (ts: number, originalDate: string, lang: string): string => {
  if (!ts) return originalDate; // Fallback for old data with published_ts = 0
  const d = new Date(ts * 1000);
  const year = d.getFullYear();
  const currentYear = new Date().getFullYear();
  
  const bcp47Lang = mapSteamLangToBCP47(lang);
  if (year === currentYear) {
    return new Intl.DateTimeFormat(bcp47Lang, { month: 'short', day: 'numeric' }).format(d);
  } else {
    return new Intl.DateTimeFormat(bcp47Lang, { year: 'numeric', month: 'short' }).format(d);
  }
};

const parseSizeInBytes = (sizeStr: string): number => {
  const clean = sizeStr.trim().toUpperCase();
  // Match number and unit (e.g., 10.8 GB, 302.6 MB, etc.)
  const match = clean.match(/^([\d.]+)\s*(GB|MB|KB|B)$/);
  if (!match) return 0;
  const num = parseFloat(match[1]);
  const unit = match[2];
  switch (unit) {
    case "GB": return num * 1024 * 1024 * 1024;
    case "MB": return num * 1024 * 1024;
    case "KB": return num * 1024;
    default: return num;
  }
};

const formatSizeGB = (sizeStr: string): string => {
  const bytes = parseSizeInBytes(sizeStr);
  if (bytes === 0) return "0.0 GB";
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(1)} GB`;
};


interface Torrents1337PanelProps {
  showToast: (msg: string) => void;
  isScraping: boolean;
  scrapeProgress: number;
  scrapeMessage: string;
  onStartScrape: () => void;
  onCancelScrape: () => void;
  searchVal: string;
  setSearchVal: (v: string) => void;
  sortVal: string;
  setSortVal: (v: string) => void;
  ratingFilter?: string;
  setRatingFilter?: (v: string) => void;
}

export default function Torrents1337Panel({ 
  showToast, 
  isScraping, 
  scrapeProgress, 
  scrapeMessage, 
  onStartScrape,
  onCancelScrape,
  searchVal,
  setSearchVal,
  sortVal,
  setSortVal,
  ratingFilter,
  setRatingFilter
}: Torrents1337PanelProps) {
  const { t, i18n } = useTranslation();
  const [torrents, setTorrents] = useState<Torrent1337x[]>([]);
  const [localRatingFilter, setLocalRatingFilter] = useState("");
  
  const currentRatingFilter = ratingFilter !== undefined ? ratingFilter : localRatingFilter;
  const handleRatingFilterChange = setRatingFilter || setLocalRatingFilter;

  const ratingFilterOptions = [
    { value: "positive", label: t("srFilterPositive") || "Steam好评 (≥70%)" },
    { value: "very_positive", label: t("srFilterVeryPositive") || "特别好评 (≥80%)" },
    { value: "overwhelmingly_positive", label: t("srFilterOverwhelminglyPositive") || "好评如潮 (≥95%)" },
    { value: "mixed_plus", label: t("srFilterMixedPlus") || "褒贬不一及以上 (≥40%)" },
    { value: "has_rating", label: t("srFilterHasRating") || "仅看有评价" },
  ];
  
  // Pagination State
  const [currentPage, setCurrentPage] = useState<number>(1);
  const pageSize = 30;

  // Scroll to top when page changes
  useEffect(() => {
    const scrollContainer = document.querySelector(".tab-content-scrollable");
    if (scrollContainer) {
      scrollContainer.scrollTo({ top: 0, behavior: "smooth" });
    }
  }, [currentPage]);

  const loadData = () => {
    invoke<Torrent1337x[]>("get_torrents_1337x_command")
      .then((data) => {
        setTorrents(data || []);
      })
      .catch((err) => {
        console.error("加载种子列表失败:", err);
        setTorrents([]);
      });
  };

  useEffect(() => {
    loadData();
  }, [isScraping, scrapeProgress]);

  // Reset page when filters change
  useEffect(() => {
    setCurrentPage(1);
  }, [searchVal, sortVal, currentRatingFilter]);

  // Apply search query and Steam rating filter
  let filteredTorrents = torrents.filter(t => {
    const matchesSearch = 
      t.name.toLowerCase().includes(searchVal.toLowerCase()) ||
      t.uploader.toLowerCase().includes(searchVal.toLowerCase()) ||
      (t.base_name && t.base_name.toLowerCase().includes(searchVal.toLowerCase()));

    if (!matchesSearch) return false;

    if (currentRatingFilter) {
      const scoreDesc = Number(t.review_score_desc) || 0;
      const percent = t.positive_percent !== undefined && t.positive_percent !== null ? Number(t.positive_percent) : null;

      if (currentRatingFilter === "positive") {
        return scoreDesc >= 6 || (percent !== null && percent >= 70 && scoreDesc >= 5);
      }
      if (currentRatingFilter === "very_positive") {
        return scoreDesc >= 8 || (percent !== null && percent >= 80 && scoreDesc >= 6);
      }
      if (currentRatingFilter === "overwhelmingly_positive") {
        return scoreDesc === 9 || (percent !== null && percent >= 95 && (t.total_reviews || 0) >= 500);
      }
      if (currentRatingFilter === "mixed_plus") {
        return scoreDesc >= 5 || (percent !== null && percent >= 40 && scoreDesc > 0);
      }
      if (currentRatingFilter === "has_rating") {
        return scoreDesc > 0 || (percent !== null && percent > 0);
      }
    }

    return true;
  });

  // Apply sorting options
  if (sortVal === "rating-desc") {
    filteredTorrents.sort((a, b) => {
      const hasRatingA = (a.review_score_desc !== undefined && a.review_score_desc !== null && Number(a.review_score_desc) > 0) || ((a.positive_percent || 0) > 0);
      const hasRatingB = (b.review_score_desc !== undefined && b.review_score_desc !== null && Number(b.review_score_desc) > 0) || ((b.positive_percent || 0) > 0);
      if (hasRatingA && !hasRatingB) return -1;
      if (!hasRatingA && hasRatingB) return 1;
      if (!hasRatingA && !hasRatingB) return b.published_ts - a.published_ts;

      const scoreA = a.positive_percent !== undefined && a.positive_percent !== null ? Number(a.positive_percent) : 0;
      const scoreB = b.positive_percent !== undefined && b.positive_percent !== null ? Number(b.positive_percent) : 0;
      if (scoreA !== scoreB) return scoreB - scoreA;
      const descA = Number(a.review_score_desc) || 0;
      const descB = Number(b.review_score_desc) || 0;
      if (descA !== descB) return descB - descA;
      return (b.total_reviews || 0) - (a.total_reviews || 0);
    });
  } else if (sortVal === "rating-asc") {
    filteredTorrents.sort((a, b) => {
      const hasRatingA = (a.review_score_desc !== undefined && a.review_score_desc !== null && Number(a.review_score_desc) > 0) || ((a.positive_percent || 0) > 0);
      const hasRatingB = (b.review_score_desc !== undefined && b.review_score_desc !== null && Number(b.review_score_desc) > 0) || ((b.positive_percent || 0) > 0);
      if (hasRatingA && !hasRatingB) return -1;
      if (!hasRatingA && hasRatingB) return 1;
      if (!hasRatingA && !hasRatingB) return b.published_ts - a.published_ts;

      const scoreA = a.positive_percent !== undefined && a.positive_percent !== null ? Number(a.positive_percent) : 0;
      const scoreB = b.positive_percent !== undefined && b.positive_percent !== null ? Number(b.positive_percent) : 0;
      if (scoreA !== scoreB) return scoreA - scoreB;
      const descA = Number(a.review_score_desc) || 0;
      const descB = Number(b.review_score_desc) || 0;
      if (descA !== descB) return descA - descB;
      return (a.total_reviews || 0) - (b.total_reviews || 0);
    });
  } else if (sortVal === "seeds-desc") {
    filteredTorrents.sort((a, b) => b.seeds - a.seeds);
  } else if (sortVal === "seeds-asc") {
    filteredTorrents.sort((a, b) => a.seeds - b.seeds);
  } else if (sortVal === "leeches-desc") {
    filteredTorrents.sort((a, b) => b.leeches - a.leeches);
  } else if (sortVal === "leeches-asc") {
    filteredTorrents.sort((a, b) => a.leeches - b.leeches);
  } else if (sortVal === "size-desc") {
    filteredTorrents.sort((a, b) => parseSizeInBytes(b.size) - parseSizeInBytes(a.size));
  } else if (sortVal === "size-asc") {
    filteredTorrents.sort((a, b) => parseSizeInBytes(a.size) - parseSizeInBytes(b.size));
  } else if (sortVal === "name-asc") {
    filteredTorrents.sort((a, b) => a.name.localeCompare(b.name));
  } else if (sortVal === "name-desc") {
    filteredTorrents.sort((a, b) => b.name.localeCompare(a.name));
  } else if (sortVal === "date-asc") {
    filteredTorrents.sort((a, b) => a.published_ts - b.published_ts);
  } else {
    // Default sort by date desc
    filteredTorrents.sort((a, b) => b.published_ts - a.published_ts);
  }

  // Paginate filtered torrents
  const totalItems = filteredTorrents.length;
  const totalPages = Math.ceil(totalItems / pageSize) || 1;
  const paginatedTorrents = filteredTorrents.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  const sortOptions = [
    { value: "date-asc", label: t("t1337SortDateAsc") },
    { value: "rating-desc", label: t("t1337SortRatingDesc") },
    { value: "rating-asc", label: t("t1337SortRatingAsc") },
    { value: "seeds-desc", label: t("t1337SortSeedsDesc") },
    { value: "seeds-asc", label: t("t1337SortSeedsAsc") },
    { value: "leeches-desc", label: t("t1337SortLeechesDesc") },
    { value: "leeches-asc", label: t("t1337SortLeechesAsc") },
    { value: "size-desc", label: t("t1337SortSizeDesc") },
    { value: "size-asc", label: t("t1337SortSizeAsc") },
    { value: "name-asc", label: t("t1337SortNameAsc") },
    { value: "name-desc", label: t("t1337SortNameDesc") }
  ];

  const handleOpenUrl = (url: string) => {
    invoke("open_url_command", { url })
      .catch((err) => {
        console.error("无法打开链接:", err);
        window.open(url, "_blank");
      });
  };

  const handleCopyUrl = (url: string, name: string) => {
    navigator.clipboard.writeText(url).then(() => {
      showToast(`${t("t1337CopySuccess")}${name}`);
    }).catch(() => {
      showToast(t("t1337CopyFail"));
    });
  };

  return (
    <div className="panel" style={{ display: "block" }}>
      {/* Syncing Progress Banner */}
      {isScraping && (
        <div style={{
          background: "rgba(0, 242, 254, 0.05)",
          border: "1px solid rgba(0, 242, 254, 0.2)",
          borderRadius: "12px",
          padding: "1rem 1.25rem",
          marginBottom: "1.5rem",
          display: "flex",
          flexDirection: "column",
          gap: "0.5rem",
          boxShadow: "0 0 15px rgba(0, 242, 254, 0.05)"
        }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <span style={{ fontWeight: 600, fontSize: "0.9rem", color: "var(--primary-accent)", display: "flex", alignItems: "center", gap: "0.5rem" }}>
              <span className="animate-pulse" style={{ display: "inline-block", width: "8px", height: "8px", background: "var(--primary-accent)", borderRadius: "50%" }}></span>
              {scrapeMessage}
            </span>
            <span style={{ fontSize: "0.85rem", fontWeight: 700, color: "var(--primary-accent)", fontFamily: "'Outfit', sans-serif" }}>
              {scrapeProgress}%
            </span>
          </div>
          <div style={{
            background: "rgba(255,255,255,0.05)",
            height: "6px",
            borderRadius: "3px",
            overflow: "hidden"
          }}>
            <div style={{
              background: "var(--accent-gradient)",
              width: `${scrapeProgress}%`,
              height: "100%",
              transition: "width 0.3s ease"
            }} />
          </div>
          <div style={{ display: "flex", justifyContent: "flex-end" }}>
            <button 
              onClick={onCancelScrape}
              style={{
                background: "rgba(239, 68, 68, 0.12)",
                border: "1px solid rgba(239, 68, 68, 0.2)",
                color: "#ef4444",
                padding: "0.25rem 0.75rem",
                borderRadius: "6px",
                fontSize: "0.75rem",
                fontWeight: 650,
                cursor: "pointer",
                transition: "all 0.2s ease"
              }}
              onMouseEnter={(e) => { e.currentTarget.style.background = "rgba(239, 68, 68, 0.2)"; }}
              onMouseLeave={(e) => { e.currentTarget.style.background = "rgba(239, 68, 68, 0.12)"; }}
            >
              {t("t1337BtnCancelSync")}
            </button>
          </div>
        </div>
      )}

      <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "1.5rem", flexWrap: "wrap", gap: "1rem" }}>
        <div>
          <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <Download size={22} style={{ color: "var(--primary-accent)" }} />
            {t("t1337Title")}
          </h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            {t("t1337Desc")}
          </p>
        </div>
        <button 
          className="action-btn" 
          onClick={onStartScrape} 
          disabled={isScraping}
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
          <RefreshCw size={14} className={isScraping ? "animate-spin" : ""} />
          {isScraping ? t("scraping") : t("t1337BtnFetch")}
        </button>
      </div>

      <section className="controls-row" style={{ display: "flex", gap: "1rem", alignItems: "center", marginBottom: "1.5rem" }}>
        <SearchBox value={searchVal} onChange={setSearchVal} />
        <FilterSelect 
          value={currentRatingFilter} 
          onChange={handleRatingFilterChange} 
          options={ratingFilterOptions} 
          allLabel={t("srFilterAllRatings") || "所有Steam评价"} 
        />
        <SortSelect 
          value={sortVal} 
          onChange={setSortVal} 
          options={sortOptions} 
          defaultLabel={t("t1337SortDateDesc")} 
        />
      </section>

      <div className="posters-grid">
        {paginatedTorrents.map((torrent) => {
          const uniqueKey = torrent.torrent_id || torrent.name;
          const hasRating = torrent.review_score_desc !== undefined && torrent.review_score_desc !== null && torrent.review_score_desc !== "" && Number(torrent.review_score_desc) > 0;
          const ratingText = hasRating ? getReviewScoreText(t, torrent.review_score_desc) : "";
          const hasPercent = torrent.positive_percent !== undefined && torrent.positive_percent !== null && Number(torrent.positive_percent) > 0;

          const hasRecentRating = torrent.recent_review_score_desc !== undefined && torrent.recent_review_score_desc !== null && torrent.recent_review_score_desc !== "" && Number(torrent.recent_review_score_desc) > 0;
          const recentRatingText = hasRecentRating ? getReviewScoreText(t, torrent.recent_review_score_desc) : "";
          const hasRecentPercent = torrent.recent_positive_percent !== undefined && torrent.recent_positive_percent !== null && Number(torrent.recent_positive_percent) > 0;
          const showRecent = hasRecentRating && hasRating && Number(torrent.recent_review_score_desc) !== Number(torrent.review_score_desc);

          return (
            <div 
              key={uniqueKey} 
              className="poster-card"
              onClick={() => handleOpenUrl(torrent.url)}
              onContextMenu={(e) => {
                e.preventDefault();
                handleCopyUrl(torrent.url, torrent.name);
              }}
              title={`${torrent.name}\n${formatPublishDate(torrent.published_ts, torrent.date, i18n.language)}\n大小: ${formatSizeGB(torrent.size)} | 做种: ${torrent.seeds.toLocaleString()} | 下载: ${torrent.leeches.toLocaleString()}${torrent.uploader ? ` | 发布者: ${torrent.uploader}` : ""}${hasRating ? `\nSteam评价: ${ratingText}${hasPercent ? ` (${torrent.positive_percent}%)` : ""}${showRecent ? `\n近期评价: ${recentRatingText}${hasRecentPercent ? ` (${torrent.recent_positive_percent}%)` : ""}` : ""}` : "\nSteam评价: 暂无评价"}\n\n(左键打开发布页 / 点击 Steam 评价打开 Steam / 右键复制链接)`}
            >
              {hasRating && (
                <div 
                  className={`rating-overlay ${getRatingColorClass(torrent.review_score_desc)}`}
                  onClick={(e) => {
                    e.stopPropagation();
                    const steamUrl = getSteamStoreUrl(torrent.appid, torrent.base_name, torrent.name);
                    handleOpenUrl(steamUrl);
                  }}
                  title={`${ratingText}${hasPercent ? ` (${torrent.positive_percent}%)` : ""}${showRecent ? `\n近期: ${recentRatingText}${hasRecentPercent ? ` (${torrent.recent_positive_percent}%)` : ""}` : ""}\n${t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"}`}
                >
                  {hasPercent && <span>👍 {torrent.positive_percent}%</span>}
                  <span className="rating-desc">{ratingText}</span>
                  {showRecent && (
                    <span className="rating-recent" style={{ fontSize: "0.6rem", opacity: 0.85, display: "block", lineHeight: 1.2 }}>
                      近期: <span className={getRatingColorClass(torrent.recent_review_score_desc)} style={{ color: "inherit" }}>{recentRatingText}</span>
                      {hasRecentPercent && ` ${torrent.recent_positive_percent}%`}
                    </span>
                  )}
                  <ExternalLink size={10} style={{ opacity: 0.8, marginLeft: 2 }} />
                </div>
              )}

              <TorrentCoverImage torrent={torrent} />
              
              <div className="poster-info">
                <div className="poster-title" title={torrent.name}>
                  {torrent.name}
                </div>
                
                <div className="poster-meta" style={{ marginBottom: "0.25rem", color: "rgba(255,255,255,0.7)", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <span>{formatPublishDate(torrent.published_ts, torrent.date, i18n.language)}</span>
                  <span 
                    style={{ display: "flex", alignItems: "center", gap: "0.25rem", color: "#10b981", fontWeight: 650 }}
                    title={`做种数 (Seeds): ${torrent.seeds.toLocaleString()} / 下载数 (Leeches): ${torrent.leeches.toLocaleString()}`}
                  >
                    <ArrowUpCircle size={12} />
                    {torrent.seeds.toLocaleString()}
                  </span>
                </div>
                
                <div className="poster-meta" style={{ display: "flex", flexWrap: "wrap", gap: "0.3rem", justifyContent: "flex-start", alignItems: "center" }}>
                  {(torrent.appid || hasRating) && (
                    <span 
                      className="badge badge-dir" 
                      onClick={(e) => {
                        e.stopPropagation();
                        const steamUrl = getSteamStoreUrl(torrent.appid, torrent.base_name, torrent.name);
                        handleOpenUrl(steamUrl);
                      }}
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
                  {torrent.size && (
                    <span 
                      className="badge" 
                      style={{ 
                        fontSize: "0.65rem", 
                        padding: "0.15rem 0.35rem", 
                        background: "rgba(255,255,255,0.18)", 
                        border: "none", 
                        color: "#fff",
                        fontFamily: "'Outfit', sans-serif" 
                      }}
                    >
                      {formatSizeGB(torrent.size)}
                    </span>
                  )}
                  {torrent.uploader && (
                    <span 
                      className="badge" 
                      style={{ 
                        fontSize: "0.65rem", 
                        padding: "0.15rem 0.35rem", 
                        background: "rgba(255,255,255,0.08)", 
                        border: "none", 
                        color: "rgba(255,255,255,0.7)",
                        maxWidth: "90px",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap"
                      }}
                      title={`发布者: ${torrent.uploader}`}
                    >
                      {torrent.uploader}
                    </span>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>
      
      {filteredTorrents.length === 0 && !isScraping && (
        <div style={{ textAlign: "center", padding: "3rem", color: "var(--text-secondary)" }}>
          {torrents.length === 0 
            ? (t("t1337Empty") || "暂无数据，请点击右上角获取。") 
            : (t("srEmptyFiltered") || "没有找到符合筛选条件的发布记录。")}
        </div>
      )}

      <Pagination 
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        pageSize={pageSize}
        onPageChange={setCurrentPage}
      />
    </div>
  );
}
