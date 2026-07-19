import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface ScrapeProgressPayload {
  current_page: number;
  total_pages: number;
  message: string;
  status: string;
}

interface UseScrapeOptions {
  target?: "1337x" | "sr";
  mode?: "latest" | "leechers" | "seeders";
  onComplete?: (msg: string) => void;
}

export function useScrape({ target = "1337x", mode = "latest", onComplete }: UseScrapeOptions = {}) {
  const [isScraping, setIsScraping] = useState<boolean>(false);
  const [scrapeProgress, setScrapeProgress] = useState<number>(0);
  const [scrapeMessage, setScrapeMessage] = useState<string>("");

  const startScrape = useCallback(async () => {
    setIsScraping(true);
    setScrapeProgress(0);
    setScrapeMessage("初始化 1337x 数据同步任务...");

    let unlisten: (() => void) | null = null;
    try {
      let eventPrefix = target === "1337x" ? `scrape-progress-${mode}` : "scrape-progress-sr";
      unlisten = await listen<ScrapeProgressPayload>(eventPrefix, (event) => {
        const payload = event.payload;
        const progress = Math.round((payload.current_page / (payload.total_pages || 1)) * 100);
        setScrapeProgress(progress);
        setScrapeMessage(payload.message);
      });

      let cmd = target === "1337x" ? "scrape_1337x_command" : "scrape_sr_command";
      let args = target === "1337x" ? { mode } : {};
      
      const resultMsg = await invoke<string>(cmd, args);
      onComplete?.(resultMsg);
    } catch (e) {
      console.error("同步发生错误:", e);
      setScrapeMessage(`同步失败: ${e}`);
    } finally {
      setIsScraping(false);
      if (unlisten) {
        unlisten();
      }
    }
  }, [onComplete]);

  const cancelScrape = useCallback(async () => {
    try {
      await invoke("cancel_scrape_command");
    } catch (e) {
      console.error("取消同步失败:", e);
    }
  }, []);

  return {
    isScraping,
    scrapeProgress,
    scrapeMessage,
    startScrape,
    cancelScrape
  };
}
