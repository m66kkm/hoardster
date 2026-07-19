import { useState, useEffect } from "react";
import { Terminal, Copy, RefreshCw, ExternalLink, Flame } from "lucide-react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useScrape } from "../hooks/useScrape";
import type { TorrentSR } from "../types";
import Pagination from "./shared/Pagination";
import SearchBox from "./shared/SearchBox";
import SortSelect from "./shared/SortSelect";

interface SRPanelProps {
  showToast: (msg: string) => void;
}

export default function SRPanel({ showToast }: SRPanelProps) {
  const { t } = useTranslation();
  const [torrents, setTorrents] = useState<TorrentSR[]>([]);
  const [searchVal, setSearchVal] = useState("");
  const [sortVal, setSortVal] = useState("");
  const [currentPage, setCurrentPage] = useState(1);
  const pageSize = 30;
  
  const sortOptions = [
    { value: "dateAsc", label: t("srSortDateAsc") || "发布时间从旧到新 (Date)" },
    { value: "heatDesc", label: t("srSortHeatDesc") || "热度从高到低 (Heat)" },
  ];
  
  const { 
    isScraping, scrapeProgress, scrapeMessage, 
    startScrape, cancelScrape 
  } = useScrape({ 
    target: "sr",
    onComplete: (msg) => {
      if (msg) showToast(msg);
    }
  });

  const loadData = () => {
    invoke<TorrentSR[]>("get_torrents_sr_command")
      .then((data) => {
        setTorrents(data || []);
      })
      .catch((err) => {
        console.error("加载 Skidrow/Reloaded 列表失败:", err);
        setTorrents([]);
      });
  };

  useEffect(() => {
    loadData();
  }, [isScraping, scrapeProgress]);

  let filteredTorrents = torrents.filter(t => 
    t.title.toLowerCase().includes(searchVal.toLowerCase()) ||
    t.category.toLowerCase().includes(searchVal.toLowerCase())
  );

  filteredTorrents.sort((a, b) => {
    if (sortVal === "dateAsc") {
      return a.published_ts - b.published_ts;
    } else if (sortVal === "heatDesc") {
      return (b.comments || 0) - (a.comments || 0);
    } else {
      // Default: dateDesc
      return b.published_ts - a.published_ts;
    }
  });

  useEffect(() => {
    setCurrentPage(1);
  }, [searchVal, sortVal]);

  const totalItems = filteredTorrents.length;
  const totalPages = Math.ceil(totalItems / pageSize) || 1;
  const paginatedTorrents = filteredTorrents.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  const handleOpenUrl = (url: string) => {
    invoke("open_url_command", { url })
      .catch(() => {
        window.open(url, "_blank");
      });
  };

  const handleCopyUrl = (url: string) => {
    navigator.clipboard.writeText(url).then(() => {
      showToast(t("srCopySuccess") || "已复制链接！");
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
              onClick={cancelScrape}
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
              {t("srBtnCancelSync") || "取消同步"}
            </button>
          </div>
        </div>
      )}

      <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "1.5rem", flexWrap: "wrap", gap: "1rem" }}>
        <div>
          <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <Terminal size={22} style={{ color: "var(--primary-accent)" }} />
            {t("srTitle")}
          </h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            {t("srDesc")}
          </p>
        </div>
        <button 
          className="action-btn" 
          onClick={startScrape} 
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
          {isScraping ? t("scraping") || "同步中..." : t("srBtnFetch") || "获取最新"}
        </button>
      </div>

      <section className="controls-row" style={{ display: "flex", gap: "1rem", alignItems: "center", marginBottom: "1.5rem" }}>
        <SearchBox value={searchVal} onChange={setSearchVal} />
        <SortSelect 
          value={sortVal} 
          onChange={setSortVal} 
          options={sortOptions} 
          defaultLabel={t("srSortDateDesc") || "发布时间从新到旧 (Date)"} 
        />
      </section>

      <div className="posters-grid">
        {paginatedTorrents.map((t) => (
          <div 
            key={t.id} 
            className="poster-card"
            onClick={() => handleOpenUrl(t.url)}
            onContextMenu={(e) => {
              e.preventDefault();
              handleCopyUrl(t.url);
            }}
            title={`${t.title}\n${t.date}\n${t.category}\n\n(右键复制链接)`}
          >
            {t.image_url ? (
              <img 
                className="poster-img"
                src={t.image_url} 
                alt={t.title}
                referrerPolicy="no-referrer"
              />
            ) : (
              <div className="poster-fallback" style={{ background: "linear-gradient(135deg, #1e293b, #0f172a)" }}>
                <div className="poster-fallback-icon"><Terminal /></div>
                <div className="poster-fallback-title" title={t.title}>{t.title}</div>
              </div>
            )}
            
            <div className="poster-info">
              <div className="poster-title" title={t.title}>
                {t.title}
              </div>
              
              <div className="poster-meta" style={{ marginBottom: "0.25rem", color: "rgba(255,255,255,0.7)", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span>{t.date}</span>
                {t.comments > 0 && (
                  <span style={{ display: "flex", alignItems: "center", gap: "0.2rem", color: "#ef4444" }}>
                    <Flame size={12} />
                    {t.comments}
                  </span>
                )}
              </div>
              
              <div className="poster-meta" style={{ display: "flex", flexWrap: "wrap", gap: "0.3rem", justifyContent: "flex-start" }}>
                {t.category.split(",")
                  .map(c => c.trim())
                  .filter(c => {
                    const lowerC = c.toLowerCase();
                    return lowerC && !lowerC.includes("request accepted") && !lowerC.includes("pc games");
                  })
                  .slice(0, 2)
                  .map((c, i) => {
                  let badgeClass = "badge";
                  if (c.toUpperCase().includes("GAME")) badgeClass += " badge-dir";
                  else if (c.toUpperCase().includes("UPDATE")) badgeClass += " badge-ver";
                  return (
                    <span key={i} className={badgeClass} style={{ fontSize: "0.65rem", padding: "0.15rem 0.35rem", background: "rgba(255,255,255,0.2)", border: "none", color: "#fff" }}>
                      {c}
                    </span>
                  );
                })}
              </div>
            </div>
          </div>
        ))}
      </div>
      
      {filteredTorrents.length === 0 && !isScraping && (
        <div style={{ textAlign: "center", padding: "3rem", color: "var(--text-secondary)" }}>
          暂无数据，请点击右上角获取。
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
