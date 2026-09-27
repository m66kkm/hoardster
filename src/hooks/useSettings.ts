import { useState, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface ScanPathEntry {
  path: string;
  scan_type: "installed" | "archived";
}

export function useSettings() {
  const [scanPathEntries, setScanPathEntries] = useState<ScanPathEntry[]>([]);
  const [steamApiThreads, setSteamApiThreads] = useState<number>(10);
  const [language, setLanguage] = useState<string>("schinese");

  const scanPaths = useMemo(() => scanPathEntries.map((e) => e.path), [scanPathEntries]);
  const installedPaths = useMemo(
    () => scanPathEntries.filter((e) => (e.scan_type || (e as any).scanType) !== "archived").map((e) => e.path),
    [scanPathEntries]
  );
  const archivedPaths = useMemo(
    () => scanPathEntries.filter((e) => (e.scan_type || (e as any).scanType) === "archived").map((e) => e.path),
    [scanPathEntries]
  );

  // Load scan paths
  const loadScanPaths = useCallback(async () => {
    try {
      const paths = await invoke<ScanPathEntry[]>("get_scan_paths_command");
      setScanPathEntries(paths || []);
      const config = await invoke<Record<string, string>>("get_all_config_command");
      if (config.steam_api_threads) {
        setSteamApiThreads(parseInt(config.steam_api_threads) || 10);
      }
      if (config.language) {
        setLanguage(config.language);
        import("../i18n").then(({ default: i18n }) => i18n.changeLanguage(config.language));
      }
    } catch (e) {
      console.error("加载配置失败:", e);
    }
  }, []);

  const saveSteamApiThreads = useCallback(async (threads: number, showToast: (msg: string) => void) => {
    try {
      await invoke("set_config_command", { key: "steam_api_threads", value: threads.toString() });
      setSteamApiThreads(threads);
      showToast(`已保存多线程配置 (当前: ${threads} 线程)`);
    } catch (e) {
      console.error(e);
      showToast("保存多线程配置失败");
    }
  }, []);

  const saveLanguage = useCallback(async (lang: string, showToast: (msg: string) => void) => {
    try {
      await invoke("set_config_command", { key: "language", value: lang });
      setLanguage(lang);
      import("../i18n").then(({ default: i18n }) => i18n.changeLanguage(lang));
      showToast(`已切换语言 / Language switched`);
    } catch (e) {
      console.error(e);
      showToast("切换语言失败 / Failed to switch language");
    }
  }, []);

  // Add scan path
  const addScanPath = useCallback(
    async (
      scanTypeOrShowToast?: "installed" | "archived" | ((msg: string) => void),
      maybeShowToast?: (msg: string) => void
    ) => {
      let scanType: "installed" | "archived" = "installed";
      let showToast: ((msg: string) => void) | undefined;

      if (typeof scanTypeOrShowToast === "function") {
        showToast = scanTypeOrShowToast;
        scanType = "installed";
      } else if (scanTypeOrShowToast === "installed" || scanTypeOrShowToast === "archived") {
        scanType = scanTypeOrShowToast;
        showToast = maybeShowToast;
      }

      try {
        const selected = await open({
          directory: true,
          multiple: false,
        });
        if (selected === null) return; // User cancelled
        
        const path = selected as string;
        await invoke("add_scan_path_command", { path, scanType });
        loadScanPaths();
        if (showToast) {
          showToast(scanType === "archived" ? "成功添加归档路径" : "成功添加安装目录");
        }
      } catch (e) {
        console.error(e);
        if (showToast) {
          showToast("添加路径失败");
        }
      }
    },
    [loadScanPaths]
  );

  // Remove scan path
  const removeScanPath = useCallback(
    async (path: string, showToast?: (msg: string) => void) => {
      try {
        await invoke("remove_scan_path_command", { path });
        loadScanPaths();
        if (showToast) {
          showToast("已移除该扫描路径");
        }
      } catch (e) {
        console.error(e);
      }
    },
    [loadScanPaths]
  );

  return {
    scanPaths,
    scanPathEntries,
    installedPaths,
    archivedPaths,
    loadScanPaths,
    addScanPath,
    removeScanPath,
    steamApiThreads,
    saveSteamApiThreads,
    language,
    saveLanguage
  };
}
