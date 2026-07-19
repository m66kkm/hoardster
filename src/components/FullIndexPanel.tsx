import { useTranslation } from "react-i18next";
import { ExternalLink } from "lucide-react";
import type { Game } from "../types";
import { getRatingColorClass, getCoverUrl } from "../utils/helpers";
import Pagination from "./shared/Pagination";

interface FullIndexPanelProps {
  games: Game[];
  currentPage: number;
  setCurrentPage: (page: number | ((prev: number) => number)) => void;
  pageSize: number;
  openGameFolder: (path: string) => void;
}

export default function FullIndexPanel({ games, currentPage, setCurrentPage, pageSize, openGameFolder }: FullIndexPanelProps) {
  const { t } = useTranslation();

  const totalItems = games.length;
  const totalPages = Math.ceil(totalItems / pageSize) || 1;
  const paginatedGames = games.slice((currentPage - 1) * pageSize, currentPage * pageSize);

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
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.5rem" }}>
        <div>
          <h2 style={{ margin: 0 }}>{t("allTitle")}</h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            {t("allSubtitle")}
          </p>
        </div>
        {paginationControls}
      </div>

      <div className="table-container">
        <table>
          <thead>
            <tr>
              <th style={{ width: "70px" }}>{t("colIndex")}</th>
              <th>{t("colName")}</th>
              <th style={{ width: "160px" }}>{t("colSteamRating")}</th>
              <th style={{ width: "140px" }}>{t("colGenre")}</th>
              <th style={{ width: "80px" }}>{t("colFile")}</th>
              <th style={{ width: "150px" }}>{t("colStatus")}</th>
              <th style={{ width: "80px", textAlign: "center" }}>{t("colAction")}</th>
            </tr>
          </thead>
          <tbody>
            {paginatedGames.map((game, idx) => {
              const globalIdx = (currentPage - 1) * pageSize + idx + 1;
              return (
                <tr key={game.full_path}>
                  <td style={{ fontFamily: "'Outfit', sans-serif", fontWeight: 500 }}>{globalIdx}</td>
                  <td style={{ fontWeight: 600, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: "0.75rem" }}>
                    {game.local_cover ? (
                      <img src={getCoverUrl(game.local_cover) || ""} style={{ width: "32px", height: "48px", objectFit: "cover", borderRadius: "4px", border: "1px solid var(--panel-border)" }} alt="" />
                    ) : (
                      <span style={{ display: "inline-block", width: "32px", height: "48px", background: "rgba(255,255,255,0.05)", borderRadius: "4px", textAlign: "center", lineHeight: "48px", fontSize: "1.2rem" }}>🎮</span>
                    )}
                    {game.name || game.original_name}
                  </td>
                  <td>
                    {game.review_score_desc ? (
                      <span className={`rating-text ${getRatingColorClass(game.review_score_desc)}`}>
                        👍 {game.positive_percent}% ({game.review_score_desc})
                      </span>
                    ) : (
                      <span style={{ color: "var(--text-secondary)", opacity: 0.3 }}>-</span>
                    )}
                  </td>
                  <td>
                    {game.genres ? <span style={{ fontSize: "0.85rem", color: "var(--text-secondary)" }}>{game.genres}</span> : <span style={{ color: "var(--text-secondary)", opacity: 0.3 }}>-</span>}
                  </td>
                  <td>
                    <span className={`badge ${game.type === "Directory" ? "badge-dir" : "badge-iso"}`}>{game.type}</span>
                  </td>
                  <td>
                    {game.is_exact_dup && <span className="badge badge-dup" style={{ marginRight: "0.35rem" }}>{t("tagExactDup")}</span>}
                    {game.is_version_dup && <span className="badge badge-ver">{t("tagVersionDup")}</span>}
                    {!game.is_exact_dup && !game.is_version_dup && <span style={{ color: "var(--text-secondary)", opacity: 0.3 }}>-</span>}
                  </td>
                  <td style={{ textAlign: "center" }}>
                    <button className="view-btn" onClick={() => openGameFolder(game.full_path)} title={t("openInExplorer")} style={{ padding: "0.4rem", display: "inline-flex" }}>
                      <ExternalLink size={12} />
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Bottom Pagination Controls */}
      <Pagination 
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        pageSize={pageSize}
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
