import { useState, useEffect, useMemo, useRef } from "react";

export interface UseDynamicGridOptions {
  containerRef?: React.RefObject<HTMLDivElement | null>;
  gridRef?: React.RefObject<HTMLDivElement | null>;
  targetPageSize?: number; // 目标每页数量（默认 30）
  minColWidth?: number;    // 列最小宽度（默认 200，对应 CSS minmax(200px, 1fr)）
  gap?: number;            // 网格间距（默认 24，对应 CSS 1.5rem）
  enabled?: boolean;       // 是否开启动态网格计算（默认 true，非网格模式可设为 false）
}

/**
 * 通用动态网格 Hook
 * 监听容器与网格真实宽度，测量当前 CSS Grid 列数 columns，
 * 并动态调整 effectivePageSize 恒为 columns 的整数倍，
 * 彻底消除最后一页或每页末行出现的孤立未排满卡片。
 */
export function useDynamicGrid({
  containerRef: externalContainerRef,
  gridRef: externalGridRef,
  targetPageSize = 30,
  minColWidth = 200,
  gap = 24,
  enabled = true,
}: UseDynamicGridOptions = {}) {
  const internalContainerRef = useRef<HTMLDivElement>(null);
  const internalGridRef = useRef<HTMLDivElement>(null);

  const containerRef = externalContainerRef || internalContainerRef;
  const gridRef = externalGridRef || internalGridRef;

  // 根据当前实际宽度与 CSS Grid 自动换行规则预估或计算初始列数
  const [columns, setColumns] = useState<number>(() => {
    if (typeof window !== "undefined") {
      const estimatedWidth = window.innerWidth - 80;
      return Math.max(1, Math.floor((estimatedWidth + gap) / (minColWidth + gap)));
    }
    return 6;
  });

  useEffect(() => {
    if (!enabled) return;
    const container = containerRef.current;
    if (!container) return;

    let rafId: number | null = null;
    let observedGrid: HTMLElement | null = null;

    const measure = () => {
      if (rafId) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        // 1. 若网格元素已挂载渲染，直接读取浏览器当前精确计算的网格列数
        if (gridRef.current) {
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

        // 2. 备选兜底：根据容器可用宽度与 CSS Grid (minmax + gap) 规则计算
        const currentContainer = containerRef.current;
        if (currentContainer) {
          const width = currentContainer.clientWidth;
          if (width > 0) {
            const cols = Math.max(1, Math.floor((width + gap) / (minColWidth + gap)));
            setColumns(cols);
          }
        }
      });
    };

    measure();

    const observer = new ResizeObserver(() => {
      if (gridRef.current && gridRef.current !== observedGrid) {
        observedGrid = gridRef.current;
        observer.observe(observedGrid);
      }
      measure();
    });

    observer.observe(container);
    if (gridRef.current) {
      observedGrid = gridRef.current;
      observer.observe(observedGrid);
    }
    window.addEventListener("resize", measure);

    return () => {
      if (rafId) cancelAnimationFrame(rafId);
      observer.disconnect();
      window.removeEventListener("resize", measure);
    };
  }, [enabled, minColWidth, gap, containerRef, gridRef]);

  // 动态计算每页数量：必须为 columns 的整数倍，确保每一页的所有行都 100% 排满，不出现单张孤立悬空卡片
  const effectivePageSize = useMemo(() => {
    if (!enabled) {
      return targetPageSize;
    }
    const targetRows = Math.max(2, Math.round(targetPageSize / columns));
    return columns * targetRows;
  }, [enabled, targetPageSize, columns]);

  return {
    containerRef,
    gridRef,
    columns,
    effectivePageSize,
  };
}
