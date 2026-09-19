import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ScrapeTaskState {
  isScraping: boolean;
  scrapeProgress: number;
  scrapeMessage: string;
}

interface ScrapeProgressPayload {
  current_page: number;
  total_pages: number;
  message: string;
  status: string;
}

export const getTaskKey = (
  target: "1337x" | "sr",
  mode: "latest" | "leechers" | "seeders" = "latest"
): string => {
  return target === "sr" ? "sr" : `1337x-${mode}`;
};

interface ScrapeStore {
  tasks: Record<string, ScrapeTaskState>;
  startScrape: (
    target: "1337x" | "sr",
    mode?: "latest" | "leechers" | "seeders",
    onComplete?: (msg: string) => void
  ) => Promise<void>;
  cancelScrape: () => Promise<void>;
}

// Module-level unlisteners & callbacks map so they persist across component unmounts
const activeUnlisteners = new Map<string, UnlistenFn>();
const activeCallbacks = new Map<string, (msg: string) => void>();

export const useScrapeStore = create<ScrapeStore>((set, get) => ({
  tasks: {},

  startScrape: async (target, mode = "latest", onComplete) => {
    const taskKey = getTaskKey(target, mode);
    
    // If this task is already running, avoid duplicate invocation
    if (get().tasks[taskKey]?.isScraping) {
      return;
    }

    if (onComplete) {
      activeCallbacks.set(taskKey, onComplete);
    }

    const initialMsg = target === "sr" 
      ? "初始化 Skidrow/Reloaded 数据同步任务..." 
      : "初始化 1337x 数据同步任务...";

    set((state) => ({
      tasks: {
        ...state.tasks,
        [taskKey]: {
          isScraping: true,
          scrapeProgress: 0,
          scrapeMessage: initialMsg,
        },
      },
    }));

    // Clean up any stale unlistener
    if (activeUnlisteners.has(taskKey)) {
      activeUnlisteners.get(taskKey)?.();
      activeUnlisteners.delete(taskKey);
    }

    const eventPrefix = target === "1337x" ? `scrape-progress-${mode}` : "scrape-progress-sr";

    try {
      const unlisten = await listen<ScrapeProgressPayload>(eventPrefix, (event) => {
        const payload = event.payload;
        const progress = Math.round((payload.current_page / (payload.total_pages || 1)) * 100);
        set((state) => ({
          tasks: {
            ...state.tasks,
            [taskKey]: {
              isScraping: true,
              scrapeProgress: progress,
              scrapeMessage: payload.message,
            },
          },
        }));
      });

      activeUnlisteners.set(taskKey, unlisten);

      const cmd = target === "1337x" ? "scrape_1337x_command" : "scrape_sr_command";
      const args = target === "1337x" ? { mode } : {};

      const resultMsg = await invoke<string>(cmd, args);

      const cb = activeCallbacks.get(taskKey);
      activeCallbacks.delete(taskKey);
      cb?.(resultMsg);

      set((state) => ({
        tasks: {
          ...state.tasks,
          [taskKey]: {
            isScraping: false,
            scrapeProgress: 100,
            scrapeMessage: resultMsg || "同步完成",
          },
        },
      }));
    } catch (e) {
      console.error("同步发生错误:", e);
      set((state) => ({
        tasks: {
          ...state.tasks,
          [taskKey]: {
            isScraping: false,
            scrapeProgress: 0,
            scrapeMessage: `同步失败: ${e}`,
          },
        },
      }));
    } finally {
      if (activeUnlisteners.has(taskKey)) {
        activeUnlisteners.get(taskKey)?.();
        activeUnlisteners.delete(taskKey);
      }
      set((state) => {
        const t = state.tasks[taskKey];
        if (!t || !t.isScraping) return state;
        return {
          tasks: {
            ...state.tasks,
            [taskKey]: {
              ...t,
              isScraping: false,
            },
          },
        };
      });
    }
  },

  cancelScrape: async () => {
    set((state) => {
      const updatedTasks = { ...state.tasks };
      for (const k in updatedTasks) {
        if (updatedTasks[k]?.isScraping) {
          updatedTasks[k] = {
            ...updatedTasks[k],
            scrapeMessage: "正在取消同步...",
          };
        }
      }
      return { tasks: updatedTasks };
    });

    try {
      await invoke("cancel_scrape_command");
    } catch (e) {
      console.error("取消同步失败:", e);
    }
  },
}));
