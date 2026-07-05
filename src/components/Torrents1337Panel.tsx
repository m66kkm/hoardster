import { useState, useEffect } from "react";
import { Download, Check, Copy, ExternalLink, RefreshCw } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import type { Torrent1337x } from "../types";
import SearchBox from "./shared/SearchBox";
import SortSelect from "./shared/SortSelect";

import { MOCK_TORRENTS } from "../mocks/mockTorrents";

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
  setSortVal
}: Torrents1337PanelProps) {
  const { t } = useTranslation();
  const [torrents, setTorrents] = useState<Torrent1337x[]>([]);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  
  // Pagination State
  const [currentPage, setCurrentPage] = useState<number>(1);
  const pageSize = 50;

  useEffect(() => {
    if (!isScraping) {
      invoke<Torrent1337x[]>("get_torrents_1337x_command")
        .then((data) => {
          if (data && data.length > 0) {
            setTorrents(data);
          } else {
            setTorrents(MOCK_TORRENTS);
          }
        })
        .catch((err) => {
          console.error("加载种子列表失败, 使用 Mock 数据:", err);
          setTorrents(MOCK_TORRENTS);
        });
    }
  }, [isScraping]);

  // Reset page when filters change
  useEffect(() => {
    setCurrentPage(1);
  }, [searchVal, sortVal]);

  // Apply search query filter
  const filteredTorrents = torrents.filter(t => 
    t.name.toLowerCase().includes(searchVal.toLowerCase()) ||
    t.uploader.toLowerCase().includes(searchVal.toLowerCase())
  );

  // Apply sorting options
  if (sortVal === "seeds-desc") {
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
    filteredTorrents.reverse();
  }
  // If sortVal is "date-desc" or empty, we do NOT sort and just use the array order from DB

  // Paginate filtered torrents
  const totalItems = filteredTorrents.length;
  const totalPages = Math.ceil(totalItems / pageSize) || 1;
  const paginatedTorrents = filteredTorrents.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  const sortOptions = [
    { value: "date-asc", label: "发布时间从旧到新 (Date)" },
    { value: "seeds-desc", label: "种子数从多到少 (Seeds)" },
    { value: "seeds-asc", label: "种子数从少到多 (Seeds)" },
    { value: "leeches-desc", label: "下载数从多到少 (Leechers)" },
    { value: "leeches-asc", label: "下载数从少到多 (Leechers)" },
    { value: "size-desc", label: "文件从大到小 (Size)" },
    { value: "size-asc", label: "文件从小到大 (Size)" }
  ];

  const handleOpenUrl = (url: string) => {
    invoke("open_url_command", { url })
      .catch((err) => {
        console.error("无法打开链接:", err);
        window.open(url, "_blank");
      });
  };

  const handleCopyUrl = (url: string, id: string, name: string) => {
    navigator.clipboard.writeText(url).then(() => {
      setCopiedId(id);
      showToast(`已成功复制种子网页链接: ${name}`);
      setTimeout(() => setCopiedId(null), 2000);
    }).catch(() => {
      showToast("链接复制失败！");
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
              取消同步
            </button>
          </div>
        </div>
      )}

      <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "1.5rem", flexWrap: "wrap", gap: "1rem" }}>
        <div>
          <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <Download size={22} style={{ color: "var(--primary-accent)" }} />
            1337x 游戏资源索引
          </h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            搜索与获取最新的数字版镜像、Repack 与 Scene 种子（数据经本地过滤整理）。
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
          {isScraping ? t("scraping") || "正在更新..." : "获取1337信息"}
        </button>
      </div>

      <section className="controls-row" style={{ display: "flex", gap: "1rem", alignItems: "center", marginBottom: "1.5rem" }}>
        <SearchBox value={searchVal} onChange={setSearchVal} />
        <SortSelect 
          value={sortVal} 
          onChange={setSortVal} 
          options={sortOptions} 
          defaultLabel="发布时间从新到旧 (Date)" 
        />
      </section>

      <div className="table-container">
        <table>
          <thead>
            <tr>
              <th>种子发布名称</th>
              <th style={{ width: "100px" }}>大小</th>
              <th style={{ width: "90px", color: "#10b981" }}>做种 (S)</th>
              <th style={{ width: "90px", color: "#ef4444" }}>下载 (L)</th>
              <th style={{ width: "120px" }}>发布时间</th>
              <th style={{ width: "100px" }}>发布者</th>
              <th style={{ width: "120px", textAlign: "center" }}>操作</th>
            </tr>
          </thead>
          <tbody>
            {paginatedTorrents.map((t) => {
              const uniqueKey = t.torrent_id || t.name;
              return (
                <tr key={uniqueKey}>
                  <td style={{ fontWeight: 600 }}>
                    <a 
                      href="#" 
                      onClick={(e) => { e.preventDefault(); handleOpenUrl(t.url); }}
                      style={{ color: "var(--text-primary)", textDecoration: "none", borderBottom: "1px dashed var(--primary-accent)" }}
                      title="点击在浏览器中打开种子网页"
                    >
                      {t.name}
                    </a>
                  </td>
                  <td style={{ fontFamily: "'Outfit', sans-serif" }}>{formatSizeGB(t.size)}</td>
                  <td style={{ color: "#10b981", fontWeight: 600 }}>{t.seeds.toLocaleString()}</td>
                  <td style={{ color: "#ef4444", fontWeight: 600 }}>{t.leeches.toLocaleString()}</td>
                  <td>{t.date}</td>
                  <td>
                    <a 
                      href="#" 
                      onClick={(e) => { e.preventDefault(); handleOpenUrl(t.uploader_url); }}
                      className="badge badge-dir"
                      style={{ textDecoration: "none", cursor: "pointer" }}
                      title="查看发布者主页"
                    >
                      {t.uploader}
                    </a>
                  </td>
                  <td style={{ textAlign: "center" }}>
                    <div style={{ display: "flex", gap: "0.35rem", justifyContent: "center" }}>
                      <button 
                        className="view-btn"
                        onClick={() => handleOpenUrl(t.url)}
                        title="在浏览器中打开页面"
                        style={{ padding: "0.4rem", display: "inline-flex" }}
                      >
                        <ExternalLink size={12} />
                      </button>
                      <button 
                        className={`view-btn ${copiedId === uniqueKey ? "active" : ""}`}
                        onClick={() => handleCopyUrl(t.url, uniqueKey, t.name)}
                        title="复制种子页面链接"
                        style={{ padding: "0.4rem", display: "inline-flex" }}
                      >
                        {copiedId === uniqueKey ? <Check size={12} style={{ color: "#10b981" }} /> : <Copy size={12} />}
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
            {filteredTorrents.length === 0 && (
              <tr>
                <td colSpan={7} style={{ textAlign: "center", padding: "3rem", color: "var(--text-secondary)" }}>
                  未检索到任何符合条件的种子。
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: "1.5rem" }}>
          <span style={{ color: "var(--text-secondary)", fontSize: "0.85rem" }}>
            {t("paginationInfo", { 
              start: (currentPage - 1) * pageSize + 1, 
              end: Math.min(currentPage * pageSize, totalItems), 
              total: totalItems 
            }) || `显示第 ${(currentPage - 1) * pageSize + 1} 至 ${Math.min(currentPage * pageSize, totalItems)} 条，共 ${totalItems} 条`}
          </span>
          <div style={{ display: "flex", gap: "0.35rem" }}>
            <button 
              className="page-btn" 
              onClick={() => setCurrentPage((prev) => Math.max(prev - 1, 1))} 
              disabled={currentPage === 1}
            >
              {t("btnPrevPage") || "上一页"}
            </button>
            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
              let pageNum = currentPage - 2 + i;
              if (currentPage <= 2) pageNum = i + 1;
              else if (currentPage >= totalPages - 1) pageNum = totalPages - 4 + i;
              if (pageNum < 1 || pageNum > totalPages) return null;
              return (
                <button
                  key={pageNum}
                  className={`page-btn ${pageNum === currentPage ? "active" : ""}`}
                  onClick={() => setCurrentPage(pageNum)}
                >
                  {pageNum}
                </button>
              );
            })}
            <button 
              className="page-btn" 
              onClick={() => setCurrentPage((prev) => Math.min(prev + 1, totalPages))} 
              disabled={currentPage === totalPages}
            >
              {t("btnNextPage") || "下一页"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
