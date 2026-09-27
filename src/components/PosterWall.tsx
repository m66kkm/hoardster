import { useEffect } from "react";
import type { Game } from "../types";
import { useTranslation } from "react-i18next";
import GameCard from "./GameCard";
import GameDetailRow from "./GameDetailRow";
import Pagination from "./shared/Pagination";
import { useDynamicGrid } from "../hooks/useDynamicGrid";
import { useGameContextMenu } from "../hooks/useGameContextMenu";

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

  const { handleContextMenu, contextMenuElement } = useGameContextMenu({
    onDeleteGame,
    onRefresh,
  });

  const { containerRef, gridRef, effectivePageSize } = useDynamicGrid({
    targetPageSize: pageSize || (viewMode === "tile" ? 35 : 25),
    enabled: viewMode === "tile",
  });

  const totalItems = games.length;
  const totalPages = Math.ceil(totalItems / effectivePageSize) || 1;
  const paginatedGames = games.slice((currentPage - 1) * effectivePageSize, currentPage * effectivePageSize);

  useEffect(() => {
    if (currentPage > totalPages && totalPages > 0) {
      setCurrentPage(totalPages);
    }
  }, [currentPage, totalPages, setCurrentPage]);

  return (
    <div className="panel" ref={containerRef} style={{ display: "block" }}>
      <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.5rem" }}>
        <div>
          <h2 style={{ margin: 0 }}>{title}</h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            {subtitle}
          </p>
        </div>
      </div>

      {viewMode === "tile" ? (
        <div ref={gridRef} className="posters-grid">
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
        <div className="game-detail-list">
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

      {contextMenuElement}

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
