import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { StatsSummary } from "./types";
import { useGames } from "./hooks/useGames";
import { useScan } from "./hooks/useScan";
import { useSettings } from "./hooks/useSettings";
import Header from "./components/Header";
import StatsGrid from "./components/StatsGrid";
import TabNav from "./components/TabNav";
import FilterBar from "./components/FilterBar";
import Dashboard from "./components/Dashboard";
import PosterWall from "./components/PosterWall";
import DuplicatesPanel from "./components/DuplicatesPanel";
import FranchisesPanel from "./components/FranchisesPanel";
import FullIndexPanel from "./components/FullIndexPanel";
import SettingsPanel from "./components/SettingsPanel";
import Toast from "./components/Toast";
import { useTranslation } from "react-i18next";

export default function App() {
  const { t } = useTranslation();

  // Navigation & View states
  const [activeTab, setActiveTab] = useState<string>("dashboard");
  const [viewMode, setViewMode] = useState<"tile" | "detail">("tile");
  const [currentPage, setCurrentPage] = useState<number>(1);
  const pageSize = 36;

  // Filter states
  const [searchVal, setSearchVal] = useState<string>("");
  const [driveVal, setDriveVal] = useState<string>("");
  const [typeVal, setTypeVal] = useState<string>("");
  const [ratingVal, setRatingVal] = useState<string>("");
  const [sortVal, setSortVal] = useState<string>("");

  // Data states
  const [stats, setStats] = useState<StatsSummary>({
    total_scan: 0,
    unique_games: 0,
    franchise_count: 0,
    exact_dups: 0,
    version_dups: 0
  });

  // Feedback states
  const [toastMessage, setToastMessage] = useState<string>("");
  const [openAccordions, setOpenAccordions] = useState<{ [key: string]: boolean }>({});
  
  const loggerRef = useRef<HTMLDivElement>(null);

  const [genres, setGenres] = useState<string[]>([]);
  const [ratings, setRatings] = useState<string[]>([]);

  useEffect(() => {
    invoke<string[]>("get_all_genres_command").then(setGenres).catch(console.error);
    invoke<{name: string, count: number}[]>("get_rating_stats_command")
      .then(stats => setRatings(stats.map(s => s.name)))
      .catch(console.error);
  }, []);

  const showToast = useCallback((msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(""), 2500);
  }, []);

  // Custom hooks
  const { gamesList, exactDuplicates, versionDuplicates, franchises, loadGames, loadDuplicates, loadFranchises } = useGames({
    searchVal, driveVal, typeVal, ratingVal, sortVal
  });

  const { scanPaths, loadScanPaths, addScanPath, removeScanPath, steamApiThreads, saveSteamApiThreads, language, saveLanguage } = useSettings();

  // Load stats summary from DB
  const loadStats = useCallback(async () => {
    try {
      const summary = await invoke<StatsSummary>("get_games_stats_command");
      setStats(summary);
    } catch (e) {
      console.error("加载统计数据失败:", e);
    }
  }, []);

  // Trigger loading for tab specific data
  const loadTabData = useCallback(() => {
    if (activeTab === "posters") {
      loadGames(true, false);
    } else if (activeTab === "installed") {
      loadGames(true, true);
    } else if (activeTab === "all") {
      loadGames(false, false);
    } else if (activeTab === "duplicates") {
      loadDuplicates();
    } else if (activeTab === "franchise") {
      loadFranchises();
    } else if (activeTab === "dashboard") {
      loadStats();
    }
  }, [activeTab, loadGames, loadDuplicates, loadFranchises, loadStats]);

  const onScanComplete = useCallback(() => {
    showToast(t("scanCompleteToast") || "Data sync complete!");
    loadStats();
    loadTabData();
  }, [showToast, loadStats, loadTabData, t]);

  const { isScanning, scanProgress, scanMessage, scanLogs, startScan, cancelScan, clearLogs } = useScan({
    onComplete: onScanComplete
  });

  // Initialize
  useEffect(() => {
    loadStats();
    loadScanPaths();
  }, [loadStats, loadScanPaths]);

  // Fetch items based on active tab and filters
  useEffect(() => {
    setCurrentPage(1);
    loadTabData();
  }, [activeTab, searchVal, driveVal, typeVal, ratingVal, sortVal, loadTabData]);

  // Scroll to bottom of logger
  useEffect(() => {
    if (loggerRef.current) {
      loggerRef.current.scrollTop = loggerRef.current.scrollHeight;
    }
  }, [scanLogs]);

  // Open directory in File Explorer
  const openGameFolder = useCallback(async (path: string) => {
    try {
      await invoke("open_game_folder_command", { path });
      showToast(t("toastFolderOpened") || "Opened in File Explorer");
    } catch (e) {
      console.error(e);
      showToast(t("toastFolderFailed") || "Failed to open folder");
    }
  }, [showToast, t]);

  // Copy path to clipboard
  const copyPath = useCallback((path: string, gameName: string) => {
    navigator.clipboard.writeText(path).then(() => {
      showToast(`${t("toastCopied") || "Copied path:"} ${gameName}`);
    }).catch(() => {
      showToast(t("toastCopyFailed") || "Failed to copy path");
    });
  }, [showToast, t]);

  const toggleAccordion = useCallback((key: string) => {
    setOpenAccordions(prev => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const handleAddScanPath = useCallback(() => {
    addScanPath(showToast);
  }, [addScanPath, showToast]);

  const handleRemoveScanPath = useCallback((path: string) => {
    removeScanPath(path, showToast);
  }, [removeScanPath, showToast]);

  const handleSaveSteamApiThreads = useCallback((threads: number) => {
    saveSteamApiThreads(threads, showToast);
  }, [saveSteamApiThreads, showToast]);

  const handleSaveLanguage = useCallback((lang: string) => {
    saveLanguage(lang, showToast);
  }, [saveLanguage, showToast]);

  return (
    <div className="container">
      {/* Header */}
      <Header
        isScanning={isScanning}
        onStartScan={startScan}
        onSettingsClick={() => setActiveTab("settings")}
        isSettingsActive={activeTab === "settings"}
        onIndexClick={() => setActiveTab("all")}
        isIndexActive={activeTab === "all"}
      />

      {/* Tab Nav */}
      <TabNav activeTab={activeTab} onTabChange={setActiveTab} />

      {/* Filter Row */}
      <FilterBar
        activeTab={activeTab}
        searchVal={searchVal}
        setSearchVal={setSearchVal}
        driveVal={driveVal}
        setDriveVal={setDriveVal}
        typeVal={typeVal}
        setTypeVal={setTypeVal}
        ratingVal={ratingVal}
        setRatingVal={setRatingVal}
        sortVal={sortVal}
        setSortVal={setSortVal}
        viewMode={viewMode}
        setViewMode={setViewMode}
        scanPaths={scanPaths}
        genres={genres}
        ratings={ratings}
      />

      {/* Scrollable Tab Content Area */}
      <div className="tab-content-scrollable">
        {/* DASHBOARD PANEL */}
        {activeTab === "dashboard" && (
          <>
            <StatsGrid stats={stats} onCardClick={setActiveTab} />
            <Dashboard 
              scanPaths={scanPaths} 
              onGenreClick={(genre) => {
                setTypeVal(genre);
                setActiveTab("posters");
              }}
              onRatingClick={(rating) => {
                setRatingVal(rating);
                setActiveTab("posters");
              }}
            />
          </>
        )}

        {/* POSTERS / INSTALLED PANEL */}
        {(activeTab === "posters" || activeTab === "installed") && (
          <PosterWall
            games={gamesList}
            viewMode={viewMode}
            currentPage={currentPage}
            setCurrentPage={setCurrentPage}
            pageSize={pageSize}
            title={activeTab === "posters" ? t("wallTitlePosters") : t("wallTitleInstalled")}
            subtitle={t("wallSubtitle")}
            copyPath={copyPath}
            openGameFolder={openGameFolder}
          />
        )}

        {/* DUPLICATES PANEL */}
        {activeTab === "duplicates" && (
          <DuplicatesPanel
            exactDuplicates={exactDuplicates}
            versionDuplicates={versionDuplicates}
            copyPath={copyPath}
            openGameFolder={openGameFolder}
          />
        )}

        {/* FRANCHISES PANEL */}
        {activeTab === "franchise" && (
          <FranchisesPanel
            franchises={franchises}
            openAccordions={openAccordions}
            toggleAccordion={toggleAccordion}
            copyPath={copyPath}
            openGameFolder={openGameFolder}
          />
        )}

        {/* ALL GAMES INDEX PANEL */}
        {activeTab === "all" && (
          <FullIndexPanel
            games={gamesList}
            currentPage={currentPage}
            setCurrentPage={setCurrentPage}
            pageSize={pageSize}
            copyPath={copyPath}
            openGameFolder={openGameFolder}
          />
        )}

        {/* SETTINGS PANEL */}
        {activeTab === "settings" && (
          <SettingsPanel
            scanPaths={scanPaths}
            addScanPath={handleAddScanPath}
            removeScanPath={handleRemoveScanPath}
            startScan={startScan}
            cancelScan={cancelScan}
            isScanning={isScanning}
            scanProgress={scanProgress}
            scanMessage={scanMessage}
            scanLogs={scanLogs}
            loggerRef={loggerRef}
            steamApiThreads={steamApiThreads}
            saveSteamApiThreads={handleSaveSteamApiThreads}
            clearLogs={clearLogs}
            language={language}
            saveLanguage={handleSaveLanguage}
          />
        )}
      </div>

      {/* Toast popup */}
      <Toast message={toastMessage} />
    </div>
  );
}
