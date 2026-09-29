import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface SteamResyncPayload {
  is_running: boolean;
  current: number;
  total: number;
  message: string;
  status: string; // "running", "completed", "cancelled", "error"
}

export function useSteamResync() {
  const [isResyncing, setIsResyncing] = useState(false);
  const [resyncCurrent, setResyncCurrent] = useState(0);
  const [resyncTotal, setResyncTotal] = useState(0);
  const [resyncMessage, setResyncMessage] = useState("");
  const [resyncStatus, setResyncStatus] = useState("idle");

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let isMounted = true;

    listen<SteamResyncPayload>("steam-resync-progress", (event) => {
      if (!isMounted) return;
      const data = event.payload;
      setIsResyncing(data.is_running);
      setResyncCurrent(data.current);
      setResyncTotal(data.total);
      setResyncMessage(data.message);
      setResyncStatus(data.status);
    }).then((u) => {
      unlisten = u;
    });

    return () => {
      isMounted = false;
      if (unlisten) unlisten();
    };
  }, []);

  const startResync = useCallback(async () => {
    setIsResyncing(true);
    setResyncCurrent(0);
    setResyncTotal(0);
    setResyncMessage("正在初始化 Steam 元数据重新获取任务...");
    setResyncStatus("running");
    try {
      const res = await invoke<string>("resync_all_steam_metadata_command");
      setResyncMessage(res);
      setResyncStatus("completed");
    } catch (e: any) {
      console.error("Steam 元数据重新获取失败:", e);
      setResyncMessage(typeof e === "string" ? e : e?.message || "获取失败");
      setResyncStatus("error");
    } finally {
      setIsResyncing(false);
    }
  }, []);

  const cancelResync = useCallback(async () => {
    try {
      await invoke("cancel_steam_resync_command");
    } catch (e) {
      console.error("取消 Steam 元数据重新获取失败:", e);
    }
  }, []);

  const progressPercent = resyncTotal > 0 ? Math.min(100, Math.round((resyncCurrent / resyncTotal) * 100)) : 0;

  return {
    isResyncing,
    resyncCurrent,
    resyncTotal,
    resyncMessage,
    resyncStatus,
    progressPercent,
    startResync,
    cancelResync,
  };
}
