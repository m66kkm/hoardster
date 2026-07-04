import { useEffect, useRef } from "react";
import { Sliders, FolderOpen, Info } from "lucide-react";

import TabNav, { type TabDef } from "../components/TabNav";
import SettingsPanel from "../components/SettingsPanel";

import { useSettingsStore } from "../stores/useSettingsStore";
import { useAppStore } from "../stores/useAppStore";
import { useSettings } from "../hooks/useSettings";
import { useScan } from "../hooks/useScan";

export default function SettingsPage() {

  const { showToast } = useAppStore();
  const { activeTab, setActiveTab } = useSettingsStore();

  const { 
    scanPaths, loadScanPaths, addScanPath, removeScanPath, 
    steamApiThreads, saveSteamApiThreads, 
    language, saveLanguage 
  } = useSettings();

  const { 
    isScanning, scanProgress, scanMessage, scanLogs, 
    startScan, cancelScan, clearLogs 
  } = useScan({ onComplete: () => {} });

  const loggerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadScanPaths();
  }, [loadScanPaths]);

  useEffect(() => {
    if (loggerRef.current) {
      loggerRef.current.scrollTop = loggerRef.current.scrollHeight;
    }
  }, [scanLogs]);

  const tabs: TabDef[] = [
    { id: "general", icon: Sliders, labelKey: "tabSettingsGeneral" },
    { id: "local", icon: FolderOpen, labelKey: "tabSettingsLocal" },
    { id: "intel", icon: Info, labelKey: "tabSettingsIntel" }
  ];

  return (
    <>
      <TabNav 
        tabs={tabs} 
        activeTab={activeTab} 
        onTabChange={(id) => setActiveTab(id)} 
      />

      <div className="tab-content-scrollable">
        <SettingsPanel
          activeTab={activeTab}
          scanPaths={scanPaths}
          addScanPath={() => addScanPath(showToast)}
          removeScanPath={(path) => removeScanPath(path, showToast)}
          startScan={startScan}
          cancelScan={cancelScan}
          isScanning={isScanning}
          scanProgress={scanProgress}
          scanMessage={scanMessage}
          scanLogs={scanLogs}
          loggerRef={loggerRef}
          steamApiThreads={steamApiThreads}
          saveSteamApiThreads={(threads) => saveSteamApiThreads(threads, showToast)}
          clearLogs={clearLogs}
          language={language}
          saveLanguage={(lang) => saveLanguage(lang, showToast)}
        />
      </div>
    </>
  );
}
