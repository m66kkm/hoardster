import { useTranslation } from "react-i18next";

interface PaginationProps {
  currentPage: number;
  totalPages: number;
  totalItems: number;
  pageSize: number;
  onPageChange: (page: number) => void;
}

export default function Pagination({
  currentPage,
  totalPages,
  totalItems,
  pageSize,
  onPageChange,
}: PaginationProps) {
  const { t } = useTranslation();

  if (totalItems <= 0 || totalPages <= 0) return null;

  const start = (currentPage - 1) * pageSize + 1;
  const end = Math.min(currentPage * pageSize, totalItems);

  return (
    <div
      className="pagination"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
      }}
    >
      <span
        style={{
          color: "var(--text-secondary)",
          fontSize: "0.85rem",
        }}
      >
        {t("paginationInfo", {
          start,
          end,
          total: totalItems,
          defaultValue: `显示 ${start}-${end}，共 ${totalItems} 条`,
        })}
      </span>

      <div style={{ display: "flex", alignItems: "center", gap: "0.35rem" }}>
        <button
          className="page-btn"
          onClick={() => onPageChange(Math.max(currentPage - 1, 1))}
          disabled={currentPage === 1}
        >
          {t("btnPrevPage", "上一页")}
        </button>

        {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
          let pageNum: number;
          if (currentPage <= 2) {
            pageNum = i + 1;
          } else if (currentPage >= totalPages - 1) {
            pageNum = totalPages - 4 + i;
          } else {
            pageNum = currentPage - 2 + i;
          }

          if (pageNum < 1 || pageNum > totalPages) return null;

          return (
            <button
              key={pageNum}
              className={`page-btn ${pageNum === currentPage ? "active" : ""}`}
              onClick={() => onPageChange(pageNum)}
            >
              {pageNum}
            </button>
          );
        })}

        <button
          className="page-btn"
          onClick={() => onPageChange(Math.min(currentPage + 1, totalPages))}
          disabled={currentPage === totalPages}
        >
          {t("btnNextPage", "下一页")}
        </button>
      </div>
    </div>
  );
}
