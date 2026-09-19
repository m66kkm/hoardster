import { useCallback } from "react";
import { useScrapeStore, getTaskKey } from "../stores/useScrapeStore";

interface UseScrapeOptions {
  target?: "1337x" | "sr";
  mode?: "latest" | "leechers" | "seeders";
  onComplete?: (msg: string) => void;
}

export function useScrape({ target = "1337x", mode = "latest", onComplete }: UseScrapeOptions = {}) {
  const taskKey = getTaskKey(target, mode);
  
  const task = useScrapeStore((s) => s.tasks[taskKey]) || {
    isScraping: false,
    scrapeProgress: 0,
    scrapeMessage: "",
  };

  const startScrapeAction = useScrapeStore((s) => s.startScrape);
  const cancelScrapeAction = useScrapeStore((s) => s.cancelScrape);

  const startScrape = useCallback(async () => {
    return startScrapeAction(target, mode, onComplete);
  }, [startScrapeAction, target, mode, onComplete]);

  return {
    isScraping: task.isScraping,
    scrapeProgress: task.scrapeProgress,
    scrapeMessage: task.scrapeMessage,
    startScrape,
    cancelScrape: cancelScrapeAction,
  };
}
