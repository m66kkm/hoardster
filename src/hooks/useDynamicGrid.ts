import { useState, useEffect, useMemo, useRef, useCallback } from "react";

export interface UseDynamicGridOptions {
  containerRef?: React.RefObject<HTMLDivElement | null>;
  gridRef?: React.RefObject<HTMLDivElement | null>;
  targetPageSize?: number; // 目标每页数量（默认 35）
  minColWidth?: number;    // 列最小宽度（默认 200，对应 CSS minmax(200px, 1fr)）
  gap?: number;            // 网格间距（默认 24，对应 CSS 1.5rem）
  enabled?: boolean;       // 是否开启动态网格计算（默认 true，非网格模式可设为 false）
}

/**
 * 通用动态网格 Hook
 * 监听容器与网格真实宽度，测量当前 CSS Grid 真实渲染列数 columns，
 * 并动态调整 effectivePageSize 恒为 columns 的整数倍，
 * 彻底消除多页展示时最底下一行未排满、存在空缺卡片槽位的问题。
 */
export function useDynamicGrid({
  containerRef: externalContainerRef,
  gridRef: externalGridRef,
  targetPageSize = 35,
  minColWidth = 200,
  gap = 24,
  enabled = true,
}: UseDynamicGridOptions = {}) {
  const internalContainerRef = useRef<HTMLDivElement>(null);
  const internalGridRef = useRef<HTMLDivElement>(null);

  const containerRef = externalContainerRef || internalContainerRef;
  const gridRef = externalGridRef || internalGridRef;

  // 根据当前实际可用宽度与 CSS Grid 自动换行规则预估初始列数
  const [columns, setColumns] = useState<number>(() => {
    if (typeof window !== "undefined") {
      // .container max-width: 95%, .panel padding: 1.75rem (56px) + 滚动条占位
      const estimatedWidth = Math.max(300, window.innerWidth * 0.95 - 70);
      return Math.max(1, Math.floor((estimatedWidth + gap) / (minColWidth + gap)));
    }
    return 6;
  });

  // 测量真实渲染列数
  const measureColumns = useCallback((): number | null => {
    const gridEl = gridRef.current;

    // 1. 实测已渲染子元素：若已有卡片渲染，检查第一行实际排布的卡片数量
    // 这是真实 DOM 中已经发生的物理排布，100% 真实准确
    if (gridEl && gridEl.children.length >= 2) {
      const children = Array.from(gridEl.children) as HTMLElement[];
      const firstTop = children[0].offsetTop;
      let countInRow1 = 0;
      for (const child of children) {
        if (Math.abs(child.offsetTop - firstTop) < 8) {
          countInRow1++;
        } else {
          break;
        }
      }
      if (countInRow1 > 0 && countInRow1 < children.length) {
        return countInRow1;
      }
    }

    // 2. 读取浏览器 Chromium 引擎实时解析计算出的 CSS Grid 模板列数
    if (gridEl) {
      const computed = window.getComputedStyle(gridEl);
      const templateCols = computed.getPropertyValue("grid-template-columns");
      if (templateCols && templateCols !== "none") {
        const cols = templateCols.trim().split(/\s+/).filter((s) => s && s !== "none").length;
        if (cols > 0) {
          return cols;
        }
      }

      // 3. 根据 gridEl 自身 clientWidth 与计算得出的 actualGap 计算
      const width = gridEl.clientWidth;
      if (width > 0) {
        const actualGap = parseFloat(computed.columnGap || computed.gap) || gap;
        return Math.max(1, Math.floor((width + actualGap) / (minColWidth + actualGap)));
      }
    }

    // 4. 根据容器元素（如 .panel）可用内容宽度（减去左右内边距）兜底计算
    const containerEl = containerRef.current || gridEl?.parentElement;
    if (containerEl) {
      const computed = window.getComputedStyle(containerEl);
      const padLeft = parseFloat(computed.paddingLeft) || 0;
      const padRight = parseFloat(computed.paddingRight) || 0;
      const width = Math.max(0, containerEl.clientWidth - padLeft - padRight);
      if (width > 0) {
        return Math.max(1, Math.floor((width + gap) / (minColWidth + gap)));
      }
    }

    return null;
  }, [gridRef, containerRef, minColWidth, gap]);

  useEffect(() => {
    if (!enabled) return;

    let rafId: number | null = null;

    const update = () => {
      const measured = measureColumns();
      if (measured !== null && measured > 0) {
        setColumns((prev) => (prev !== measured ? measured : prev));
      }
    };

    const scheduleUpdate = () => {
      if (rafId) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(update);
    };

    // 立即执行一次测量，无需等待下一帧
    update();

    // 安排下一帧再次校验（应对初次加载时图片排版或滚动条介入后的微调）
    scheduleUpdate();

    const observer = new ResizeObserver(() => {
      scheduleUpdate();
    });

    if (gridRef.current) {
      observer.observe(gridRef.current);
    }
    if (containerRef.current) {
      observer.observe(containerRef.current);
    }
    if (gridRef.current?.parentElement && gridRef.current.parentElement !== containerRef.current) {
      observer.observe(gridRef.current.parentElement);
    }

    window.addEventListener("resize", scheduleUpdate);

    return () => {
      if (rafId) cancelAnimationFrame(rafId);
      observer.disconnect();
      window.removeEventListener("resize", scheduleUpdate);
    };
  }, [enabled, measureColumns, containerRef, gridRef]);

  // 动态计算每页数量：恒为 columns 的整数倍，确保每一页的所有行都 100% 排满
  const effectivePageSize = useMemo(() => {
    if (!enabled) {
      return targetPageSize;
    }
    const safeColumns = Math.max(1, columns || 1);
    const targetRows = Math.max(2, Math.round(targetPageSize / safeColumns));
    return safeColumns * targetRows;
  }, [enabled, targetPageSize, columns]);

  return {
    containerRef,
    gridRef,
    columns,
    effectivePageSize,
  };
}
