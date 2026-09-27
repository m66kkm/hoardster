import { useState, useEffect, useRef, useMemo } from "react";
import { createPortal } from "react-dom";
import type { Game } from "../types";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Trash2, Link2, X, RefreshCw } from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import GameCard from "./GameCard";
import GameDetailRow from "./GameDetailRow";
import Pagination from "./shared/Pagination";

interface PosterWallProps {
  games: Game[];
  viewMode: "tile" | "detail";
  currentPage: number;
  setCurrentPage: (page: number | ((prev: number) => number)) => void;
  pageSize?: number;
  title: string;
  subtitle: string;
  copyPath: (path: string, gameName: string) => void;
  openGameFolder: (path: string) => void;
  onDeleteGame?: (game: Game) => void;
  onRefresh?: () => void;
}

export default function PosterWall({
  games,
  viewMode,
  currentPage,
  setCurrentPage,
  pageSize = 35,
  title,
  subtitle,
  copyPath,
  openGameFolder,
  onDeleteGame,
  onRefresh
}: PosterWallProps) {
  const { t } = useTranslation();
  const { showToast } = useAppStore();
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; game: Game } | null>(null);

  // 手工映射弹窗状态
  const [manualMapGame, setManualMapGame] = useState<Game | null>(null);
  const [manualMapInput, setManualMapInput] = useState("");
  const [isMapping, setIsMapping] = useState(false);
  const [mapError, setMapError] = useState("");

  const containerRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);

  // 根据当前实际宽度与 CSS Grid 自动换行规则动态计算列数
  const [columns, setColumns] = useState<number>(() => {
    if (typeof window !== "undefined") {
      const estimatedWidth = window.innerWidth - 80;
      const gap = 24; // 1.5rem
      const minColWidth = 200;
      return Math.max(1, Math.floor((estimatedWidth + gap) / (minColWidth + gap)));
    }
    return 6;
  });

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    let rafId: number | null = null;

    const measure = () => {
      if (rafId) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        // 1. 若平铺模式下 gridRef 已渲染，直接读取浏览器当前精确计算的网格列数
        if (viewMode === "tile" && gridRef.current) {
          const computed = window.getComputedStyle(gridRef.current);
          const templateCols = computed.getPropertyValue("grid-template-columns");
          if (templateCols && templateCols !== "none") {
            const cols = templateCols.trim().split(/\s+/).filter(Boolean).length;
            if (cols > 0) {
              setColumns(cols);
              return;
            }
          }
        }

        // 2. 备选兜底：根据容器可用宽度与 CSS Grid (minmax 200px + gap 24px) 规则计算
        const width = container.clientWidth;
        if (width > 0) {
          const gap = 24;
          const minColWidth = 200;
          const cols = Math.max(1, Math.floor((width + gap) / (minColWidth + gap)));
          setColumns(cols);
        }
      });
    };

    measure();

    const observer = new ResizeObserver(() => {
      measure();
    });

    observer.observe(container);
    if (gridRef.current) {
      observer.observe(gridRef.current);
    }
    window.addEventListener("resize", measure);

    return () => {
      if (rafId) cancelAnimationFrame(rafId);
      observer.disconnect();
      window.removeEventListener("resize", measure);
    };
  }, [viewMode]);

  // 动态计算每页数量：必须为 columns 的整数倍，确保每一页的所有行都 100% 排满，不出现单张孤立悬空卡片
  const effectivePageSize = useMemo(() => {
    if (viewMode !== "tile") {
      return pageSize || 25;
    }
    const targetCount = pageSize || 35;
    const targetRows = Math.max(2, Math.round(targetCount / columns));
    return columns * targetRows;
  }, [viewMode, pageSize, columns]);

  const handleContextMenu = (e: React.MouseEvent, game: Game) => {
    e.preventDefault();
    e.stopPropagation();
    const menuWidth = 160;
    const menuHeight = onDeleteGame ? 85 : 45;
    const x = e.clientX + menuWidth > window.innerWidth 
      ? Math.max(10, window.innerWidth - menuWidth - 10) 
      : Math.max(10, e.clientX);
    const y = e.clientY + menuHeight > window.innerHeight 
      ? Math.max(10, window.innerHeight - menuHeight - 10) 
      : Math.max(10, e.clientY);
    setContextMenu({ x, y, game });
  };

  const handleConfirmManualMap = async () => {
    if (!manualMapGame) return;
    const trimmed = manualMapInput.trim();
    if (!trimmed) {
      setMapError(t("manualMapInvalidInput"));
      return;
    }

    setIsMapping(true);
    setMapError("");
    try {
      const res = await invoke<{
        appid?: number;
        name?: string;
        local_cover?: string;
      }>("manual_map_steam_game_command", {
        baseName: manualMapGame.base_name,
        appidInput: trimmed,
      });

      showToast(t("manualMapSuccess", { name: res.name || manualMapGame.base_name }));
      setManualMapGame(null);
      if (onRefresh) {
        onRefresh();
      }
    } catch (err: any) {
      console.error("Manual map error:", err);
      setMapError(typeof err === "string" ? err : err.message || String(err));
    } finally {
      setIsMapping(false);
    }
  };

  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!contextMenu) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && menuRef.current.contains(e.target as Node)) {
        return;
      }
      setContextMenu(null);
    };

    const handleClose = () => setContextMenu(null);

    const timer = setTimeout(() => {
      document.addEventListener("mousedown", handleClickOutside);
      document.addEventListener("contextmenu", handleClickOutside);
      window.addEventListener("scroll", handleClose, true);
      window.addEventListener("resize", handleClose);
    }, 50);

    return () => {
      clearTimeout(timer);
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("contextmenu", handleClickOutside);
      window.removeEventListener("scroll", handleClose, true);
      window.removeEventListener("resize", handleClose);
    };
  }, [contextMenu]);

  const totalItems = games.length;
  const totalPages = Math.ceil(totalItems / effectivePageSize) || 1;
  const paginatedGames = games.slice((currentPage - 1) * effectivePageSize, currentPage * effectivePageSize);

  useEffect(() => {
    if (currentPage > totalPages && totalPages > 0) {
      setCurrentPage(totalPages);
    }
  }, [currentPage, totalPages, setCurrentPage]);

  const paginationControls = totalItems > 0 && (
    <div className="pagination" style={{ display: "flex", alignItems: "center", gap: "0.35rem" }}>
      <span style={{ color: "var(--text-secondary)", fontSize: "0.85rem", marginRight: "0.5rem" }}>
        {t("paginationTotal", { total: totalItems })}
      </span>
      <button className="page-btn" onClick={() => setCurrentPage((prev: number) => Math.max(prev - 1, 1))} disabled={currentPage === 1}>
        {t("btnPrevPage")}
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
      <button className="page-btn" onClick={() => setCurrentPage((prev: number) => Math.min(prev + 1, totalPages))} disabled={currentPage === totalPages}>
        {t("btnNextPage")}
      </button>
    </div>
  );

  return (
    <div className="panel" ref={containerRef} style={{ display: "block" }}>
      <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.5rem" }}>
        <div>
          <h2 style={{ margin: 0 }}>{title}</h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            {subtitle}
          </p>
        </div>
        {paginationControls}
      </div>

      {viewMode === "tile" ? (
        <div className="posters-grid" ref={gridRef}>
          {paginatedGames.map((game) => (
            <GameCard
              key={game.full_path}
              game={game}
              onOpenFolder={openGameFolder}
              onContextMenu={handleContextMenu}
            />
          ))}
        </div>
      ) : (
        <div className="posters-list">
          {paginatedGames.map((game) => (
            <GameDetailRow
              key={game.full_path}
              game={game}
              onCopyPath={copyPath}
              onOpenFolder={openGameFolder}
              onContextMenu={handleContextMenu}
            />
          ))}
        </div>
      )}

      {/* Custom Context Menu via Portal */}
      {contextMenu && typeof document !== "undefined" && createPortal(
        <div 
          ref={menuRef}
          className="custom-context-menu" 
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onMouseDown={(e) => e.stopPropagation()}
          onClick={(e) => e.stopPropagation()}
        >
          <div 
            className="context-menu-item" 
            onMouseDown={(e) => e.stopPropagation()}
            onClick={(e) => {
              e.stopPropagation();
              const game = contextMenu.game;
              setContextMenu(null);
              setManualMapGame(game);
              setManualMapInput(game.appid ? String(game.appid) : "");
              setMapError("");
            }}
          >
            <Link2 size={15} />
            <span>{t("manualMapSteam") || "手工映射"}</span>
          </div>

          {onDeleteGame && (
            <>
              <div className="context-menu-divider" />
              <div 
                className="context-menu-item danger" 
                onMouseDown={(e) => e.stopPropagation()}
                onClick={(e) => {
                  e.stopPropagation();
                  const game = contextMenu.game;
                  setContextMenu(null);
                  onDeleteGame(game);
                }}
              >
                <Trash2 size={15} />
                <span>{t("deleteCurrentGame") || "删除当前游戏"}</span>
              </div>
            </>
          )}
        </div>,
        document.body
      )}

      {/* 手工映射弹窗 */}
      {manualMapGame && typeof document !== "undefined" && createPortal(
        <div 
          className="modal-overlay" 
          onClick={() => !isMapping && setManualMapGame(null)}
        >
          <div 
            className="modal-content" 
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <h3>
                <Link2 size={18} style={{ color: "var(--primary-accent)" }} />
                <span>{t("manualMapTitle")}</span>
              </h3>
              <button 
                className="modal-close-btn" 
                onClick={() => setManualMapGame(null)}
                disabled={isMapping}
                title={t("titlebarClose") || "关闭"}
              >
                <X size={18} />
              </button>
            </div>

            <div className="modal-body">
              <p className="modal-desc">{t("manualMapDesc")}</p>

              <div className="modal-info-box">
                <div className="modal-info-row">
                  <span className="modal-info-label">{t("manualMapCurrentGame")}:</span>
                  <span className="modal-info-value" title={manualMapGame.original_name}>
                    {manualMapGame.original_name}
                  </span>
                </div>
                <div className="modal-info-row">
                  <span className="modal-info-label">{t("manualMapCurrentMapping")}:</span>
                  <span className="modal-info-value">
                    {manualMapGame.appid ? `${manualMapGame.appid} (${manualMapGame.name || "Steam"})` : t("manualMapNotMapped")}
                  </span>
                </div>
              </div>

              <div className="modal-field">
                <label htmlFor="steam-appid-input">{t("manualMapInputLabel")}</label>
                <input
                  id="steam-appid-input"
                  className="modal-input"
                  type="text"
                  autoFocus
                  placeholder={t("manualMapInputPlaceholder")}
                  value={manualMapInput}
                  onChange={(e) => {
                    setManualMapInput(e.target.value);
                    if (mapError) setMapError("");
                  }}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && !isMapping) {
                      handleConfirmManualMap();
                    } else if (e.key === "Escape" && !isMapping) {
                      setManualMapGame(null);
                    }
                  }}
                  disabled={isMapping}
                />
              </div>

              {mapError && (
                <div className="modal-error">{mapError}</div>
              )}
            </div>

            <div className="modal-actions">
              <button 
                className="modal-btn-cancel" 
                onClick={() => setManualMapGame(null)}
                disabled={isMapping}
              >
                {t("manualMapBtnCancel")}
              </button>
              <button 
                className="action-btn" 
                onClick={handleConfirmManualMap}
                disabled={isMapping || !manualMapInput.trim()}
              >
                {isMapping ? (
                  <>
                    <RefreshCw size={14} className="spin" />
                    <span>{t("manualMapFetching")}</span>
                  </>
                ) : (
                  <span>{t("manualMapBtnConfirm")}</span>
                )}
              </button>
            </div>
          </div>
        </div>,
        document.body
      )}

      {/* Bottom Pagination Controls */}
      <Pagination 
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        pageSize={effectivePageSize}
        onPageChange={setCurrentPage as (page: number) => void}
      />

      {totalItems === 0 && (
        <div style={{ textAlign: "center", padding: "4rem", color: "var(--text-secondary)", border: "1px dashed var(--panel-border)", borderRadius: "12px" }}>
          {t("noResults")}
        </div>
      )}
    </div>
  );
}
