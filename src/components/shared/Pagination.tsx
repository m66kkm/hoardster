import { useEffect, useCallback, useRef } from "react";
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

  // 记录每个页码离开时的最终滚动位置 { [page]: scrollTop }
  const pageScrollMapRef = useRef<Record<number, number>>({});
  // 待还原的目标滚动高度
  const targetScrollTopRef = useRef<number | null>(null);
  // 是否正在执行还原滚动，避免触发实时 scroll 记录覆盖
  const isRestoringScrollRef = useRef<boolean>(false);

  // 列表总量或每页大小变化（如重新搜索、切换过滤）时重置滚动位置记录
  useEffect(() => {
    pageScrollMapRef.current = {};
    targetScrollTopRef.current = null;
  }, [totalItems, pageSize]);

  // 实时捕获当前页的滚动位置，确保离开当前页时已记录最新浏览深度
  useEffect(() => {
    const scrollContainer = document.querySelector('.tab-content-scrollable');
    if (!scrollContainer) return;

    const handleScroll = () => {
      if (!isRestoringScrollRef.current) {
        pageScrollMapRef.current[currentPage] = scrollContainer.scrollTop;
      }
    };

    scrollContainer.addEventListener('scroll', handleScroll, { passive: true });
    return () => {
      scrollContainer.removeEventListener('scroll', handleScroll);
    };
  }, [currentPage]);

  // 处理页码变更并计算目标滚动位置
  const handlePageChange = useCallback((targetPage: number) => {
    if (targetPage === currentPage || targetPage < 1 || targetPage > totalPages) return;

    const scrollContainer = document.querySelector('.tab-content-scrollable');
    const currentScroll = scrollContainer ? scrollContainer.scrollTop : window.scrollY;

    // 1. 记下当前离开页的最终浏览位置
    pageScrollMapRef.current[currentPage] = currentScroll;

    const isGoingBack = targetPage < currentPage;
    const savedPosition = pageScrollMapRef.current[targetPage];

    if (isGoingBack) {
      // 回到上一页/更早页：优先还原上一页的最终浏览位置；未记录时默认展示至底部
      targetScrollTopRef.current = savedPosition !== undefined 
        ? savedPosition 
        : (scrollContainer ? scrollContainer.scrollHeight : 0);
    } else {
      // 前往下一页/新页：浏览新内容，默认从顶部开始
      targetScrollTopRef.current = 0;
    }

    onPageChange(targetPage);
  }, [currentPage, totalPages, onPageChange]);

  // 当 currentPage 切换后，DOM 挂载完成后精确还原目标滚动位置
  useEffect(() => {
    if (targetScrollTopRef.current !== null) {
      const targetTop = targetScrollTopRef.current;
      targetScrollTopRef.current = null;
      isRestoringScrollRef.current = true;

      // 使用 requestAnimationFrame 等待当前页的 DOM 节点计算完成
      requestAnimationFrame(() => {
        const scrollContainer = document.querySelector('.tab-content-scrollable');
        if (scrollContainer) {
          scrollContainer.scrollTo({ top: targetTop, behavior: targetTop === 0 ? 'smooth' : 'instant' });
        } else {
          window.scrollTo({ top: targetTop, behavior: targetTop === 0 ? 'smooth' : 'instant' });
        }

        // 双重校验：确保在图片尺寸或动态网格布局稳定后精确对齐
        setTimeout(() => {
          const c = document.querySelector('.tab-content-scrollable');
          if (c && targetTop > 0 && Math.abs(c.scrollTop - targetTop) > 10) {
            c.scrollTo({ top: targetTop, behavior: 'instant' });
          }
          isRestoringScrollRef.current = false;
        }, 50);
      });
    }
  }, [currentPage]);

  useEffect(() => {
    if (totalPages <= 1) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // 1. Windows 多媒体切曲键 (MediaTrackPrevious, MediaTrackNext)
      // 2. 浏览器导航/前进后退键 (BrowserBack, BrowserForward)
      const isPrev = 
        e.key === "MediaTrackPrevious" || 
        e.code === "MediaTrackPrevious" || 
        e.keyCode === 177 ||
        e.key === "BrowserBack" ||
        e.keyCode === 166;

      const isNext = 
        e.key === "MediaTrackNext" || 
        e.code === "MediaTrackNext" || 
        e.keyCode === 176 ||
        e.key === "BrowserForward" ||
        e.keyCode === 167;

      if (isPrev) {
        if (currentPage > 1) {
          e.preventDefault();
          e.stopPropagation();
          handlePageChange(currentPage - 1);
        }
      } else if (isNext) {
        if (currentPage < totalPages) {
          e.preventDefault();
          e.stopPropagation();
          handlePageChange(currentPage + 1);
        }
      }
    };

    const handleMouseUp = (e: MouseEvent) => {
      // 鼠标侧键 3 (Back) 与 4 (Forward)
      if (e.button === 3) {
        if (currentPage > 1) {
          e.preventDefault();
          handlePageChange(currentPage - 1);
        }
      } else if (e.button === 4) {
        if (currentPage < totalPages) {
          e.preventDefault();
          handlePageChange(currentPage + 1);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("mouseup", handleMouseUp);

    // 绑定 Chromium / WebView2 Media Session API 确保硬件多媒体键被直接捕获
    if ("mediaSession" in navigator) {
      try {
        navigator.mediaSession.setActionHandler("previoustrack", () => {
          if (currentPage > 1) {
            handlePageChange(currentPage - 1);
          }
        });
        navigator.mediaSession.setActionHandler("nexttrack", () => {
          if (currentPage < totalPages) {
            handlePageChange(currentPage + 1);
          }
        });
      } catch (err) {
        console.debug("MediaSession setActionHandler not supported:", err);
      }
    }

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("mouseup", handleMouseUp);
      if ("mediaSession" in navigator) {
        try {
          navigator.mediaSession.setActionHandler("previoustrack", null);
          navigator.mediaSession.setActionHandler("nexttrack", null);
        } catch (_) {}
      }
    };
  }, [currentPage, totalPages, handlePageChange]);

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
          onClick={() => handlePageChange(Math.max(currentPage - 1, 1))}
          disabled={currentPage === 1}
          title={`${t("btnPrevPage", "上一页")} (多媒体键 ⏪ / 侧键)`}
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
              onClick={() => handlePageChange(pageNum)}
            >
              {pageNum}
            </button>
          );
        })}

        <button
          className="page-btn"
          onClick={() => handlePageChange(Math.min(currentPage + 1, totalPages))}
          disabled={currentPage === totalPages}
          title={`${t("btnNextPage", "下一页")} (多媒体键 ⏩ / 侧键)`}
        >
          {t("btnNextPage", "下一页")}
        </button>
      </div>
    </div>
  );
}
