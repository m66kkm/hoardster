import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface DataCorrectionProgress {
  is_running: boolean;
  phase: string;
  current: number;
  total: number;
  current_item: string;
  message: string;
}

export interface DataCorrectionStatus {
  is_running: boolean;
  phase: string;
  current: number;
  total: number;
  last_run?: string;
}

export function useDataCorrection() {
  const [isRunning, setIsRunning] = useState(false);
  const [phase, setPhase] = useState("idle");
  const [current, setCurrent] = useState(0);
  const [total, setTotal] = useState(0);
  const [message, setMessage] = useState("");
  const [lastRun, setLastRun] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      const status: DataCorrectionStatus = await invoke("get_data_correction_status_command");
      setIsRunning(status.is_running);
      setPhase(status.phase);
      if (status.last_run) {
        setLastRun(status.last_run);
      }
    } catch (e) {
      console.error("Failed to get data correction status:", e);
    }
  }, []);

  useEffect(() => {
    refreshStatus();

    let unlisten: UnlistenFn | undefined;
    let isMounted = true;

    listen<DataCorrectionProgress>("data_correction_progress", (event) => {
      if (!isMounted) return;
      const data = event.payload;
      setIsRunning(data.is_running);
      setPhase(data.phase);
      setCurrent(data.current);
      setTotal(data.total);
      setMessage(data.message);

      if (data.phase === "completed") {
        refreshStatus();
      }
    }).then((unsub) => {
      unlisten = unsub;
    });

    return () => {
      isMounted = false;
      if (unlisten) {
        unlisten();
      }
    };
  }, [refreshStatus]);

  const startCorrection = useCallback(async () => {
    try {
      setMessage("正在启动数据校准任务...");
      setIsRunning(true);
      await invoke("trigger_data_correction_command");
    } catch (e) {
      console.error("Failed to start data correction:", e);
      setIsRunning(false);
      setMessage(String(e));
    }
  }, []);

  const cancelCorrection = useCallback(async () => {
    try {
      await invoke("cancel_data_correction_command");
    } catch (e) {
      console.error("Failed to cancel data correction:", e);
    }
  }, []);

  const progressPercent = total > 0 ? Math.round((current / total) * 100) : (phase === "completed" ? 100 : 0);

  return {
    isRunning,
    phase,
    current,
    total,
    progressPercent,
    message,
    lastRun,
    startCorrection,
    cancelCorrection,
    refreshStatus,
  };
}
