import { useState, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Trash2, Link2, X, RefreshCw } from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import type { Game } from "../types";

interface UseGameContextMenuOptions {
  onDeleteGame?: (game: Game) => void;
  onRefresh?: () => void;
}

export function useGameContextMenu({ onDeleteGame, onRefresh }: UseGameContextMenuOptions) {
  const { t } = useTranslation();
  const { showToast } = useAppStore();
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; game: Game } | null>(null);

  // 手工映射弹窗状态
  const [manualMapGame, setManualMapGame] = useState<Game | null>(null);
  const [manualMapInput, setManualMapInput] = useState("");
  const [isMapping, setIsMapping] = useState(false);
  const [mapError, setMapError] = useState("");

  const menuRef = useRef<HTMLDivElement>(null);

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

  const contextMenuElement = (
    <>
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
    </>
  );

  return {
    handleContextMenu,
    contextMenuElement,
  };
}
